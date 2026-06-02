//! Bincode 2 settings matching bincode 1.3+ `DefaultOptions` on-disk format.

use std::io::{Read, Write};

use bincode::config::{self, Config};
use serde::Serialize;
use serde::de::DeserializeOwned;

// Use 14GB limit on 64-bit systems, 2GB limit on 32-bit systems to avoid overflow
#[cfg(target_pointer_width = "64")]
pub(crate) const BINCODE_MEMORY_LIMIT: usize = 14 * 1024 * 1024 * 1024;
#[cfg(target_pointer_width = "32")]
pub(crate) const BINCODE_MEMORY_LIMIT: usize = 2 * 1024 * 1024 * 1024;

#[inline]
pub(crate) fn legacy_no_limit() -> impl Config {
    config::legacy().with_variable_int_encoding()
}

#[inline]
pub(crate) fn legacy_with_memory_limit() -> impl Config {
    config::legacy().with_variable_int_encoding().with_limit::<{ BINCODE_MEMORY_LIMIT }>()
}

pub(crate) fn encode_into_writer<W: Write, T: Serialize + ?Sized>(value: &T, writer: &mut W, config: impl Config) -> Result<(), bincode::error::EncodeError> {
    bincode::serde::encode_into_std_write(value, writer, config).map(|_| ())
}

pub(crate) fn decode_from_reader<R: Read, T: DeserializeOwned>(reader: &mut R, config: impl Config) -> Result<T, bincode::error::DecodeError> {
    bincode::serde::decode_from_std_read(reader, config)
}
