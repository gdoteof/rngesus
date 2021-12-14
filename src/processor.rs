use std::str::FromStr;

use solana_program::{
    account_info::{next_account_info,AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized}, rent::Rent, sysvar::Sysvar
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
            },
            RngesusInstruction::IncrementPtr => Self::process_increment_ptr(accounts, program_id)
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
        msg!("before anything??");
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        msg!("before sign check");

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        let rngesus_data_account = next_account_info(account_info_iter)?;
        msg!("rngesus_data_account: {:?}", rngesus_data_account);

        let rent_account = next_account_info(account_info_iter)?;
        msg!("rent_account: {:?}", rent_account);

        let rent = &Rent::from_account_info(rent_account)?;

        msg!("before rent check");
        if !rent.is_exempt(rngesus_data_account.lamports(), rngesus_data_account.data_len()) {
            return Err(ProgramError::AccountNotRentExempt);
        }
        msg!("after rent check");

        let mut rngesus_data = Rngesus::unpack_unchecked(&rngesus_data_account.try_borrow_data()?)?;
        msg!("after first unpack");
        if rngesus_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        msg!("rngesus_data_account: {:?}", rngesus_data_account);
        msg!("program_id: {:?}", program_id);
        if rngesus_data_account.owner != program_id {
            return Err(ProgramError::InvalidAccountData);
        }
        

        rngesus_data.is_initialized= true;
        rngesus_data.prev_hash = *initial_key;
        rngesus_data.ptr= 1;
        rngesus_data.num_callbacks= 0;
        rngesus_data.callbacks= vec![];

        
        Rngesus::pack(rngesus_data, &mut rngesus_data_account.try_borrow_mut_data()?)?;
        msg!("packed this time, i hope?");

        Ok(())
    }

    fn process_increment_ptr(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {

        msg!("in ptr bump");
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let rngesus_account = next_account_info(account_info_iter)?;
        msg!("after two accounts consumed");

        // derived_bpf_address contains the pubkey for the account which holds
        // the data for the currently running program.
        let (derived_bpf_address, _) =     Pubkey::find_program_address(
            &[
                &program_id.to_bytes()
            ],
            &Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111").ok().unwrap()
        );


        // Here we want to check the upgrade authority to ensure the initializaiton is done by
        // the contract creator.  Awkward because in order to check, we need to pass the account in,
        // which we do.  But since we have it, we can just derive what it is supposed to be, and check.
        
        let actual_bpf_account = next_account_info(account_info_iter)?;
        msg!("after three accounts consumed");
        

        // We need to ensure that the initialization is done by the contract owner.
        if !initializer.is_signer || *actual_bpf_account.key != derived_bpf_address{
            return Err(ProgramError::MissingRequiredSignature);
        }
        msg!("after sig check");

        let raw_data = &rngesus_account.try_borrow_mut_data()?;

        msg!("after borrow raw data_len: {}", raw_data.len());

        let mut rng_info = Rngesus::unpack(&raw_data)?;
        msg!("after unpack check");

        rng_info.ptr += 1;
        msg!("after ptr bump");
        
        let mut data = [0; Rngesus::LEN];
        Rngesus::pack(rng_info, &mut data)?;
        msg!("after ptr bump");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn generate_derived_bpf() {

        // derived_bpf_address contains the pubkey for the account which holds
        // the data for the currently running program.
        let (derived_bpf_address, _) =     Pubkey::find_program_address(
            &[
                &Pubkey::from_str("64JwRVSfuDvp2jo5MMYJ993FSAdq2gtede3ToCK2JUwN").ok().unwrap().to_bytes()
            ],
            &Pubkey::from_str("BPFLoaderUpgradeab1e11111111111111111111111").ok().unwrap(),
        );

        println!("{}", derived_bpf_address.to_string());

    }
}
