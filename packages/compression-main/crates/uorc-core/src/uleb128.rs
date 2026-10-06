//! ULEB128 encoding and decoding for UORC reference machine.
//!
//! Enforces exact shortest-form rejection rules as required by Profile 02.

use std::io::{self, Read, Write};

/// Decode a shortest-form ULEB128 integer.
///
/// # Errors
/// Returns an error if the encoding is not the shortest possible (e.g., trailing zeros).
pub fn decode_uleb128<R: Read>(mut reader: R) -> io::Result<u64> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut byte = [0u8; 1];

    loop {
        reader.read_exact(&mut byte)?;
        let b = byte[0];
        let val = (b & 0x7f) as u64;

        if shift >= 64 || (shift == 63 && val > 1) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "ULEB128 overflow"));
        }
        
        result |= val << shift;

        // Profile 02 requires shortest-form. A non-terminating zero byte (0x80)
        // that contributes nothing (i.e., at the very end of the integer) 
        // is forbidden unless it's the very first byte (which is 0x00 and terminates).
        // A terminating zero byte (0x00) is forbidden if shift > 0 and it contributes nothing.
        if (b & 0x80) == 0 {
            if b == 0 && shift > 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "ULEB128 not shortest form"));
            }
            break;
        }
        shift += 7;
    }

    Ok(result)
}

/// Encode a u64 into shortest-form ULEB128.
pub fn encode_uleb128<W: Write>(mut writer: W, mut value: u64) -> io::Result<usize> {
    let mut written = 0;
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        written += 1;
        if value == 0 {
            break;
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode() {
        let mut buf = Vec::new();
        encode_uleb128(&mut buf, 624485).unwrap();
        assert_eq!(buf, vec![0xE5, 0x8E, 0x26]);

        let mut reader = Cursor::new(buf);
        let val = decode_uleb128(&mut reader).unwrap();
        assert_eq!(val, 624485);
    }

    #[test]
    fn test_not_shortest_form() {
        // 0 encoded as 0x80, 0x00 instead of 0x00
        let buf = vec![0x80, 0x00];
        let mut reader = Cursor::new(buf);
        assert!(decode_uleb128(&mut reader).is_err());
    }
}
