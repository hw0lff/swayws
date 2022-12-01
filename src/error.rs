use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwayWsError {
    #[error("Swayipc: {0}")]
    SwayIpc(#[from] swayipc::Error),

    #[error("Cannot parse {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("No output can be matched against the specified parameters")]
    NoOutputMatched,
}
