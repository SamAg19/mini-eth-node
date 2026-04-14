use thiserror::Error;

#[derive(Debug, Error)]
pub enum RlpError {
    #[error("RLP encoding overflow")]
    Overflow,
}
