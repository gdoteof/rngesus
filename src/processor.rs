use solana_program::{
    account_info::{next_account_info,AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
};

use crate::{instruction::RngesusInstruction, state::Rngesus};


pub struct Processor;

impl Processor {
    pub fn process( program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = RngesusInstruction::unpack(instruction_data)?;

        match instruction {
            RngesusInstruction::InitRngesus { initial_key } => {
                msg!("Instruction: InitRngesus");
                Self::process_init_rngesus(accounts, &initial_key, program_id)
            }
        }
    }

    fn process_init_rngesus(
        accounts: &[AccountInfo],
        initial_key: &Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let rngesus_account = next_account_info(account_info_iter)?;

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
