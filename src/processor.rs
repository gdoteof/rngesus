use solana_program::{
    account_info::{next_account_info,AccountInfo},
    entrypoint::ProgramResult,
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
                Self::process_init_rngesus(accounts, &initial_key, program_id)
            },
            RngesusInstruction::IncrementPass { new_key, secret } => {
                Self::process_increment_pass(accounts, &new_key, &secret, program_id)
            },
        }
    }

    fn process_increment_pass(
        accounts: &[AccountInfo],
        new_key: &Pubkey,
        secret: &[u8; 32],
        program_id: &Pubkey, 
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        let rngesus_data_account = next_account_info(account_info_iter)?;

        let mut rngesus_data = Rngesus::unpack_unchecked(&rngesus_data_account.try_borrow_data()?)?;

        if !rngesus_data.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }


        if rngesus_data_account.owner != program_id {
            return Err(ProgramError::InvalidAccountData);
        }
        
        


        
        if !piapprec::verify(
            &rngesus_data.prev_hash.to_bytes(), 
            &new_key.to_bytes(), 
            secret
        ) { 
            return Err(RngesusError::IncorrectSecretOrHash.into())
        }
        
        rngesus_data.prev_hash = *new_key;
        rngesus_data.ptr += 1;
        Rngesus::pack(rngesus_data, &mut rngesus_data_account.try_borrow_mut_data()?)?;

        Ok(())

    }

    fn process_init_rngesus(
        accounts: &[AccountInfo],
        initial_key: &Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        let rngesus_data_account = next_account_info(account_info_iter)?;

        let rent_account = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(rent_account)?;

        if !rent.is_exempt(rngesus_data_account.lamports(), rngesus_data_account.data_len()) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        let mut rngesus_data = Rngesus::unpack_unchecked(&rngesus_data_account.try_borrow_data()?)?;
        if rngesus_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        if rngesus_data_account.owner != program_id {
            return Err(ProgramError::InvalidAccountData);
        }
        

        rngesus_data.is_initialized= true;
        rngesus_data.prev_hash = *initial_key;
        rngesus_data.ptr= 1;
        rngesus_data.num_callbacks= 0;
        rngesus_data.callbacks= vec![];

        
        Rngesus::pack(rngesus_data, &mut rngesus_data_account.try_borrow_mut_data()?)?;

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
