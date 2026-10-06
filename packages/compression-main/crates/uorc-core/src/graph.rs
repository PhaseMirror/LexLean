//! Graph body encoding, decoding, and static validation for UORC.
//!
//! Provides the instruction set for the reference machine and validates
//! all back-references, arities, and opcode bounds statically before execution.

use std::io::{self, Read, Write};
use crate::uleb128::{decode_uleb128, encode_uleb128};

/// Opcodes for the UORC Graph Machine.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// A literal byte sequence.
    Literal = 0x00,
    /// A reference to a previously constructed node.
    Reference = 0x01,
    /// Concatenation of two previously constructed nodes.
    Concat = 0x02,
    /// Create a new empty persistent store.
    StoreNew = 0x03,
    /// Read a value from a store by key.
    StoreRead = 0x04,
    /// Write a key-value pair to a store.
    StoreWrite = 0x05,
    /// Sequential emission of the store contents.
    Scan = 0x06,
}

impl TryFrom<u8> for Opcode {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::Literal),
            0x01 => Ok(Opcode::Reference),
            0x02 => Ok(Opcode::Concat),
            0x03 => Ok(Opcode::StoreNew),
            0x04 => Ok(Opcode::StoreRead),
            0x05 => Ok(Opcode::StoreWrite),
            0x06 => Ok(Opcode::Scan),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown opcode tag: 0x{:02X}", value),
            )),
        }
    }
}

/// An instruction within a UORC graph body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// `Literal(payload)`
    Literal(Vec<u8>),
    /// `Reference(index)`
    Reference(usize),
    /// `Concat(left_index, right_index)`
    Concat(usize, usize),
    /// `StoreNew` creates an empty finite map.
    StoreNew,
    /// `StoreRead(store_idx, key_idx)`
    StoreRead(usize, usize),
    /// `StoreWrite(store_idx, key_idx, val_idx)`
    StoreWrite(usize, usize, usize),
    /// `Scan(store_idx)` emits a stable serialization of the map.
    Scan(usize),
}

/// A graph body consists of a sequence of instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphBody {
    /// The statically validated sequence of instructions.
    pub instructions: Vec<Instruction>,
}

impl GraphBody {
    /// Parses and statically validates a graph body from a reader.
    ///
    /// # Errors
    /// Returns an error on malformed input, unknown opcodes, or invalid back-references.
    pub fn parse<R: Read>(mut reader: R) -> io::Result<Self> {
        let instruction_count = decode_uleb128(&mut reader)?;
        let mut instructions = Vec::with_capacity(instruction_count as usize);

        for current_index in 0..instruction_count {
            let mut opcode_byte = [0u8; 1];
            reader.read_exact(&mut opcode_byte)?;
            let opcode = Opcode::try_from(opcode_byte[0])?;

            let instruction = match opcode {
                Opcode::Literal => {
                    let len = decode_uleb128(&mut reader)? as usize;
                    let mut payload = vec![0u8; len];
                    reader.read_exact(&mut payload)?;
                    Instruction::Literal(payload)
                }
                Opcode::Reference => {
                    let ref_idx = decode_uleb128(&mut reader)? as u64;
                    if ref_idx >= current_index {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("invalid Reference index {ref_idx} at instruction {current_index}"),
                        ));
                    }
                    Instruction::Reference(ref_idx as usize)
                }
                Opcode::Concat => {
                    let left_idx = decode_uleb128(&mut reader)? as u64;
                    let right_idx = decode_uleb128(&mut reader)? as u64;

                    if left_idx >= current_index {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("invalid Concat left index {left_idx} at instruction {current_index}"),
                        ));
                    }
                    if right_idx >= current_index {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("invalid Concat right index {right_idx} at instruction {current_index}"),
                        ));
                    }

                    Instruction::Concat(left_idx as usize, right_idx as usize)
                }
                Opcode::StoreNew => Instruction::StoreNew,
                Opcode::StoreRead => {
                    let store_idx = decode_uleb128(&mut reader)? as u64;
                    let key_idx = decode_uleb128(&mut reader)? as u64;
                    if store_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid StoreRead store index"));
                    }
                    if key_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid StoreRead key index"));
                    }
                    Instruction::StoreRead(store_idx as usize, key_idx as usize)
                }
                Opcode::StoreWrite => {
                    let store_idx = decode_uleb128(&mut reader)? as u64;
                    let key_idx = decode_uleb128(&mut reader)? as u64;
                    let val_idx = decode_uleb128(&mut reader)? as u64;
                    if store_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid StoreWrite store index"));
                    }
                    if key_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid StoreWrite key index"));
                    }
                    if val_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid StoreWrite val index"));
                    }
                    Instruction::StoreWrite(store_idx as usize, key_idx as usize, val_idx as usize)
                }
                Opcode::Scan => {
                    let store_idx = decode_uleb128(&mut reader)? as u64;
                    if store_idx >= current_index {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Scan store index"));
                    }
                    Instruction::Scan(store_idx as usize)
                }
            };
            instructions.push(instruction);
        }

        Ok(Self { instructions })
    }

    /// Serializes the graph body.
    pub fn serialize<W: Write>(&self, mut writer: W) -> io::Result<()> {
        encode_uleb128(&mut writer, self.instructions.len() as u64)?;

        for inst in &self.instructions {
            match inst {
                Instruction::Literal(payload) => {
                    writer.write_all(&[Opcode::Literal as u8])?;
                    encode_uleb128(&mut writer, payload.len() as u64)?;
                    writer.write_all(payload)?;
                }
                Instruction::Reference(idx) => {
                    writer.write_all(&[Opcode::Reference as u8])?;
                    encode_uleb128(&mut writer, *idx as u64)?;
                }
                Instruction::Concat(left, right) => {
                    writer.write_all(&[Opcode::Concat as u8])?;
                    encode_uleb128(&mut writer, *left as u64)?;
                    encode_uleb128(&mut writer, *right as u64)?;
                }
                Instruction::StoreNew => {
                    writer.write_all(&[Opcode::StoreNew as u8])?;
                }
                Instruction::StoreRead(store_idx, key_idx) => {
                    writer.write_all(&[Opcode::StoreRead as u8])?;
                    encode_uleb128(&mut writer, *store_idx as u64)?;
                    encode_uleb128(&mut writer, *key_idx as u64)?;
                }
                Instruction::StoreWrite(store_idx, key_idx, val_idx) => {
                    writer.write_all(&[Opcode::StoreWrite as u8])?;
                    encode_uleb128(&mut writer, *store_idx as u64)?;
                    encode_uleb128(&mut writer, *key_idx as u64)?;
                    encode_uleb128(&mut writer, *val_idx as u64)?;
                }
                Instruction::Scan(store_idx) => {
                    writer.write_all(&[Opcode::Scan as u8])?;
                    encode_uleb128(&mut writer, *store_idx as u64)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_valid_graph_body() {
        let body = GraphBody {
            instructions: vec![
                Instruction::Literal(b"abc".to_vec()),       // Index 0
                Instruction::Literal(b"def".to_vec()),       // Index 1
                Instruction::Concat(0, 1),                   // Index 2
                Instruction::Reference(2),                   // Index 3
            ],
        };

        let mut buf = Vec::new();
        body.serialize(&mut buf).unwrap();

        let parsed = GraphBody::parse(Cursor::new(buf)).unwrap();
        assert_eq!(body, parsed);
    }

    #[test]
    fn test_invalid_backreference() {
        let body = GraphBody {
            instructions: vec![
                Instruction::Literal(b"abc".to_vec()),       // Index 0
                Instruction::Reference(1),                   // Index 1 (forward ref)
            ],
        };

        let mut buf = Vec::new();
        body.serialize(&mut buf).unwrap();

        let err = GraphBody::parse(Cursor::new(buf)).unwrap_err();
        assert!(err.to_string().contains("invalid Reference index 1 at instruction 1"));
    }

    #[test]
    fn test_unknown_opcode() {
        let buf = vec![
            0x01, // 1 instruction
            0xFF, // unknown opcode
        ];
        let err = GraphBody::parse(Cursor::new(buf)).unwrap_err();
        assert!(err.to_string().contains("unknown opcode tag: 0xFF"));
    }
}
