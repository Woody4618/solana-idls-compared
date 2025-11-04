use codama::CodamaErrors;
use thiserror::Error;

#[derive(CodamaErrors, Error, Debug)]
pub enum CounterError {
    #[error("Invalid instruction data provided")]
    InvalidInstructionData,

    #[error("Counter overflow occurred")]
    CounterOverflow,

    #[error("Incorrect program ID")]
    IncorrectProgramId,
}
