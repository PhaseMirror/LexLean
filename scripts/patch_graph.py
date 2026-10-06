import sys

content = open("packages/compression-main/crates/uorc-core/src/graph.rs").read()

content = content.replace("""pub enum Opcode {
    /// A literal byte sequence.
    Literal = 0x00,
    /// A reference to a previously constructed node.
    Reference = 0x01,
    /// Concatenation of two previously constructed nodes.
    Concat = 0x02,
}""", """pub enum Opcode {
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
}""")

content = content.replace("""        match value {
            0x00 => Ok(Opcode::Literal),
            0x01 => Ok(Opcode::Reference),
            0x02 => Ok(Opcode::Concat),
            _ => Err(io::Error::new(""", """        match value {
            0x00 => Ok(Opcode::Literal),
            0x01 => Ok(Opcode::Reference),
            0x02 => Ok(Opcode::Concat),
            0x03 => Ok(Opcode::StoreNew),
            0x04 => Ok(Opcode::StoreRead),
            0x05 => Ok(Opcode::StoreWrite),
            0x06 => Ok(Opcode::Scan),
            _ => Err(io::Error::new(""")


content = content.replace("""pub enum Instruction {
    /// `Literal(payload)`
    Literal(Vec<u8>),
    /// `Reference(index)`
    Reference(usize),
    /// `Concat(left_index, right_index)`
    Concat(usize, usize),
}""", """pub enum Instruction {
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
}""")

parse_str = """                Opcode::Concat => {
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
                }"""

new_parse = parse_str + """
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
                }"""

content = content.replace(parse_str, new_parse)

serialize_str = """                Instruction::Concat(left, right) => {
                    writer.write_all(&[Opcode::Concat as u8])?;
                    encode_uleb128(&mut writer, *left as u64)?;
                    encode_uleb128(&mut writer, *right as u64)?;
                }"""

new_serialize = serialize_str + """
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
                }"""

content = content.replace(serialize_str, new_serialize)

with open("packages/compression-main/crates/uorc-core/src/graph.rs", "w") as f:
    f.write(content)

