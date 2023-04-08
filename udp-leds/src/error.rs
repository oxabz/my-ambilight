use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Malformed message : invalid message length")]
    InvalidMessageLength,
    #[error("Malformed message : invalid flag")]
    InvalidFlag,
}