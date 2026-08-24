//! Bincode 2 settings matching bincode 1.3+ `DefaultOptions` on-disk format.

use std::io::{Read, Write};

use bincode::config::{self, Config};
use serde::Serialize;
use serde::de::DeserializeOwned;

// Match the previous bincode 1 cache limit on 64-bit systems.
#[cfg(target_pointer_width = "64")]
pub(crate) const BINCODE_MEMORY_LIMIT: usize = 8 * 1024 * 1024 * 1024;
#[cfg(target_pointer_width = "32")]
pub(crate) const BINCODE_MEMORY_LIMIT: usize = 2 * 1024 * 1024 * 1024;

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

#[cfg(test)]
mod tests {
    use super::{decode_from_reader, encode_into_writer, legacy_with_memory_limit};

    #[test]
    fn legacy_config_matches_bincode_one_fixture() {
        const BINCODE_ONE_VEC_U64: &[u8] = &[2, 1, 251, 44, 1];

        let mut fixture = BINCODE_ONE_VEC_U64;
        let decoded: Vec<u64> = decode_from_reader(&mut fixture, legacy_with_memory_limit()).unwrap();
        assert_eq!(decoded, [1, 300]);

        let mut encoded = Vec::new();
        encode_into_writer(&decoded, &mut encoded, legacy_with_memory_limit()).unwrap();
        assert_eq!(encoded, BINCODE_ONE_VEC_U64);
    }
}
