//! Archive framing for `uorc/archive/1`.
//!
//! Provides strict parsing and serialization of UORC archives, enforcing
//! magic bytes, versioning, and length-prefixed blocks using ULEB128.

use std::io::{self, Read, Write};
use crate::uleb128::{decode_uleb128, encode_uleb128};

/// The standard magic bytes identifying a UORC archive.
pub const UORC_MAGIC: &[u8; 4] = b"UORC";

/// The version byte for `archive/1`.
pub const UORC_VERSION_1: u8 = 0x01;

/// A UORC archive frame consisting of a sequence of data blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveFrame {
    /// The blocks contained within this archive.
    pub blocks: Vec<Block>,
}

/// A contiguous block of data within an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The raw payload of the block.
    pub payload: Vec<u8>,
}

impl ArchiveFrame {
    /// Parses an `ArchiveFrame` from a reader.
    ///
    /// # Errors
    /// Returns an error if the magic bytes mismatch, the version is unsupported,
    /// or if the ULEB128 framing boundaries are malformed (e.g., EOF before block ends).
    pub fn parse<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != UORC_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid UORC magic bytes",
            ));
        }

        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != UORC_VERSION_1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported UORC version: {}", version[0]),
            ));
        }

        // Read the number of blocks as a ULEB128 integer.
        let block_count = decode_uleb128(&mut reader)?;
        let mut blocks = Vec::with_capacity(block_count.try_into().unwrap_or(0));

        for _ in 0..block_count {
            let length = decode_uleb128(&mut reader)?;
            let length_usize: usize = length.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "block length exceeds memory capacity")
            })?;
            
            let mut payload = vec![0u8; length_usize];
            reader.read_exact(&mut payload)?;
            blocks.push(Block { payload });
        }

        // Enforce exact EOF rules: trailing bytes are forbidden.
        let mut trailing = [0u8; 1];
        if reader.read(&mut trailing)? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing bytes detected after archive frame",
            ));
        }

        Ok(Self { blocks })
    }

    /// Serializes the `ArchiveFrame` to a writer.
    pub fn serialize<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(UORC_MAGIC)?;
        writer.write_all(&[UORC_VERSION_1])?;

        // Write the number of blocks
        encode_uleb128(&mut writer, self.blocks.len() as u64)?;

        // Write each block
        for block in &self.blocks {
            encode_uleb128(&mut writer, block.payload.len() as u64)?;
            writer.write_all(&block.payload)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_archive_roundtrip() {
        let frame = ArchiveFrame {
            blocks: vec![
                Block { payload: b"hello".to_vec() },
                Block { payload: b"uorc".to_vec() },
            ],
        };

        let mut buf = Vec::new();
        frame.serialize(&mut buf).unwrap();

        let parsed = ArchiveFrame::parse(Cursor::new(buf)).unwrap();
        assert_eq!(frame, parsed);
    }

    #[test]
    fn test_invalid_magic() {
        let buf = b"BAD 1\x00".to_vec();
        let err = ArchiveFrame::parse(Cursor::new(buf)).unwrap_err();
        assert_eq!(err.to_string(), "invalid UORC magic bytes");
    }

    #[test]
    fn test_trailing_bytes() {
        let frame = ArchiveFrame { blocks: vec![] };
        let mut buf = Vec::new();
        frame.serialize(&mut buf).unwrap();
        buf.push(0xFF); // Trailing garbage

        let err = ArchiveFrame::parse(Cursor::new(buf)).unwrap_err();
        assert_eq!(err.to_string(), "trailing bytes detected after archive frame");
    }
}
