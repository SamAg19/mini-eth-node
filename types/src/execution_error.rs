use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Unexpected prefix found: {0}")]
    UnexpectedPrefix(u8),
    #[error("Expected input length is {expected}, but found {actual}")]
    InputTooShort {
        expected: usize,
        actual: usize
    },
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>)
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance{
        available: u128,
        required: u128
    },
    #[error("Out of gas: limit {limit}, used {used}")]
    OutOfGas {
        limit: u64,
        used: u64
    },
    #[error("Invalid nonce: expected {expected}, found {actual}")]
    InvalidNonce {
        expected: u64,
        actual: u64
    },
    #[error("Overflow error")]
    Overflow,
    #[error("Insufficient max fee: base {base_fee}, max {max_fee}")]
    InsufficientMaxFee {
        base_fee: u128,
        max_fee: u128
    },
    // `#[from]` generates `impl From<DecodeError> for ExecutionError`, which lets                                                      
    // the `?` operator automatically convert `DecodeError` into `ExecutionError`                                                       
    // in any function returning Result<_, ExecutionError>.
    #[error(transparent)]
    Decode(#[from] DecodeError)
}