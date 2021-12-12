use solana_program::{
    account_info::{next_account_info,AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
};

use crate::{instruction::RngesusInstruction, state::Rngesus, error::RngesusError};

pub struct Processor;

impl Processor {
    pub fn process( program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = RngesusInstruction::unpack(instruction_data)?;

        match instruction {
            RngesusInstruction::InitRngesus { initial_key } => {
                msg!("Instruction: InitRngesus");
                Self::process_init_rngesus(accounts, &initial_key, program_id)
            },
            RngesusInstruction::IncrementPass { new_key, secret } => {
                msg!("Instruction: IncrementPass");
                Self::process_increment_pass(accounts, &new_key, &secret, program_id)
            }
        }
    }

    fn process_increment_pass(
        accounts: &[AccountInfo],
        new_key: &Pubkey,
        secret: &[u8; 32],
        program_id: &Pubkey, 
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        // We don't care who the initializer is, if they have the secret it's alll gravy
        let _ = next_account_info(account_info_iter)?;

        let rngesus_account = next_account_info(account_info_iter)?;

        // We need to ensure that the passed in rng account is the right one
        if rngesus_account.owner != program_id{
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut rng_info = Rngesus::unpack_unchecked(&rngesus_account.try_borrow_data()?)?;
        if !rng_info.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        
        if !piapprec::verify(
            &rng_info.prev_hash.to_bytes(), 
            &new_key.to_bytes(), 
            secret
        ) { 
            return Err(RngesusError::IncorrectSecretOrHash.into())
        }
        
        rng_info.prev_hash = *new_key;
        Rngesus::pack(rng_info, &mut rngesus_account.try_borrow_mut_data()?)?;

        Ok(())

    }

    fn process_init_rngesus(
        accounts: &[AccountInfo],
        initial_key: &Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let rngesus_account = next_account_info(account_info_iter)?;

        // We need to ensure that the initialization is done by the contract owner.
        if !initializer.is_signer || rngesus_account.owner != initializer.key{
            return Err(ProgramError::MissingRequiredSignature);
        }

        if rngesus_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut rng_info = Rngesus::unpack_unchecked(&rngesus_account.try_borrow_data()?)?;
        if rng_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        
        rng_info.prev_hash = *initial_key;
        Rngesus::pack(rng_info, &mut rngesus_account.try_borrow_mut_data()?)?;

        Ok(())
    }
}
