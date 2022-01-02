use std::convert::TryInto;
use arrayref::{mut_array_refs};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::RngesusError::InvalidInstruction;

pub enum RngesusInstruction {
    /// Starts the contract
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the client invoking the function
    /// 1. `[writeable]` The data account which should be created before, and passed in.
    /// 2. `[]` The rent account, so we can make sure our data is rent-free
    InitRngesus {
        /// the first "prev_key"
        initial_key: Pubkey
    },

    /// Move to the next epoch without satisfying any bets.
    /// Primarily used for testing
    ///
    /// Accounts expected:
    ///
    /// 0. `[]` The account of the client invoking the function,
    ///         it doesn't need to be signed, because the secret
    ///         proves control.
    /// 1. `[]` The account of the Rngesus Program 
    IncrementPass {
        // the new prev_key
        new_key: Pubkey,
        // the secret which proves it's from the same derived chain
        secret: [u8; 32],
    },
    RegisterCallback {
        // the program's pubkey that will receive the callback
        program_address: Pubkey,
    },
}
impl RngesusInstruction {
    /// Unpacks a byte buffer into a [RngesusInstruction](enum.RngesusInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitRngesus {
                initial_key: Self::unpack_first_key(rest)?,
            },
            1 => Self::IncrementPass {
                new_key: Self::unpack_first_key(rest)?,
                secret: rest[32..64].try_into().unwrap()
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_first_key(input: &[u8]) -> Result<Pubkey, ProgramError> {
        let key = input
            .get(..32)
            .and_then(|slice| slice.try_into().ok())
            .map(Pubkey::new)
            .ok_or(InvalidInstruction)?;
        Ok(key)
    }

    pub fn pack(&self) -> Result<Vec<u8>, ProgramError>{
        match self {
            RngesusInstruction::InitRngesus { initial_key} => {
                let mut ret = [0; 33];
                let (instruction_dst, key_dst) = mut_array_refs![& mut ret, 1, 32];
                instruction_dst[0] = 1;
                *key_dst = initial_key.to_bytes();
                return Ok(ret.to_vec());
            },
            _ => Err(ProgramError::Custom(420))
        }

    }
}
