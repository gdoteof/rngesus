use std::convert::TryInto;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::RngesusError::InvalidInstruction;

pub enum RngesusInstruction {
    /// Starts the contract
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the client invoking the function
    /// 1. `[]` The account of the Rngesus Program 
    InitRngesus {
        /// The amount party A expects to receive of token Y
        initial_key: Pubkey
    }
}
impl RngesusInstruction {
    /// Unpacks a byte buffer into a [RngesusInstruction](enum.RngesusInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitRngesus {
                initial_key: Self::unpack_key(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_key(input: &[u8]) -> Result<Pubkey, ProgramError> {
        let amount = input
            .get(..32)
            .and_then(|slice| slice.try_into().ok())
            .map(Pubkey::new)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
