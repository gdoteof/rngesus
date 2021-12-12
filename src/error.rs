use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum RngesusError {

    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("More Callbacks than space allocated")]
    TooManyCallbacks,
    #[error("Incorrect Secret or Hash")]
    IncorrectSecretOrHash,
}

impl From<RngesusError> for ProgramError {
    fn from (e: RngesusError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
