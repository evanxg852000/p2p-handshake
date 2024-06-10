use std::{io, string::FromUtf8Error};

use thiserror::Error;

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("An io error occurred")]
    Io(#[from] io::Error),
    #[error("A string conversion error occurred")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("A variable length integer conversion error occurred")]
    LEB128Error(#[from] leb128::read::Error),
    #[error("unknown error")]
    Unknown(String),
}
