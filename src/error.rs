use std::ops::{Range, RangeFrom};

use serde::de;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    TryFromIntError(#[from] core::num::TryFromIntError),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    // #[error("Deserialization error: {0}")]
    // SerdeDeserializationError(String),

    // #[error(transparent)]
    // SerdeDeError(#[from] serde::de::Error),
    #[error("BinaryIndexOutOfBounds: index: {}, binary: {:?}", index, binary)]
    BinaryIndexOutOfBounds { index: usize, binary: Vec<u8> },

    #[error("BinaryRangeFromOutOfBounds: index: {:?}, binary: {:?}", range, binary)]
    BinaryRangeFromOutOfBounds {
        range: RangeFrom<usize>,
        binary: Vec<u8>,
    },

    #[error("BinaryRangeOutOfBounds: index: {:?}, binary: {:?}", range, binary)]
    BinaryRangeOutOfBounds {
        range: Range<usize>,
        binary: Vec<u8>,
    },
}

// impl From<de::Error> for Error {
//     fn from(err: de::Error) -> Self {
//         Error::SerdeDeserializationError(err.to_string())
//     }
// }

pub type Result<T> = core::result::Result<T, Error>;
