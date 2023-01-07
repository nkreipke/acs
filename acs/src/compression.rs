use crate::{AcsError, AcsResult};
use crate::bit_reader::BitReader;

const MAX_UNCOMPRESSED_LEN: usize = 10_000_000;

/// Implements the ACS compression algorithm
pub fn decompress(data: &[u8], target: &mut Vec<u8>) -> AcsResult<()> {
    let mut reader = BitReader::new(data)?;

    // First byte is 0
    if reader.read_bits(8)? != 0 {
        return Err(AcsError::InvalidCompressedData("invalid header"));
    }

    loop {
        if target.len() > MAX_UNCOMPRESSED_LEN {
            return Err(AcsError::InvalidCompressedData("data length overflow"));
        }

        let is_compressed = reader.read_bit()?;

        if !is_compressed {
            let uncompressed_byte = reader.read_bits(8)? as u8;
            //println!("write uncompressed byte {} ({:#x})", target.len(), uncompressed_byte);
            target.push(uncompressed_byte);
        } else {
            let mut decode_bytes = 2_u32;

            // Count sequential bits
            let mut bit_count = 0_u8;
            while bit_count < 3 && reader.read_bit()? {
                bit_count += 1;
            }

            //println!("bit count for BOS is {}", bit_count);

            // Read the amount of bits specified and add the fixed offset as per specification
            let buffer_offset_subtractor = match bit_count {
                0 => reader.read_bits(6)? + 0x1,
                1 => reader.read_bits(9)? + 0x41,
                2 => reader.read_bits(12)? + 0x241,
                3 => {
                    let raw_value = reader.read_bits(20)?;
                    if raw_value == 0x000FFFFF {
                        // End of bit stream
                        break;
                    }

                    decode_bytes += 1;

                    raw_value + 0x1241
                },
                _ => unreachable!()
            };

            if buffer_offset_subtractor as usize > target.len() {
                //println!("e: subtractor is {}, length is {}", buffer_offset_subtractor, target.len());
                return Err(AcsError::InvalidCompressedData("invalid offset"));
            }

            let buffer_offset = target.len() - (buffer_offset_subtractor as usize);

            // Count sequential bits
            let mut bit_count = 0_u8;
            while reader.read_bit()? {
                bit_count += 1;

                if bit_count == 12 {
                    return Err(AcsError::InvalidCompressedData("invalid decode length"));
                }
            }

            // Add the value of the bits consumed to the amount to decode
            decode_bytes += (0b1 << bit_count) - 1;

            if bit_count > 0 {
                let increment = reader.read_bits(bit_count)?;

                // Read the specified amount of bits and add it to the amount to decode
                decode_bytes += increment;
            }

            for byte_i in 0..decode_bytes {
                let src_offset = buffer_offset + byte_i as usize;
                let copy_value = target[src_offset];

                //println!("copying from offset {} to {} ({:#x})", src_offset, target.len(), copy_value);

                target.push(copy_value);
            }
        }
    }

    target.shrink_to_fit();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decompress() {
        let compressed = [0x00, 0x40, 0x00, 0x04, 0x10, 0xD0, 0x90, 0x80,
            0x42, 0xED, 0x98, 0x01, 0xB7, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF];

        let mut decompressed = vec![];
        decompress(&compressed, &mut decompressed).unwrap();

        let expected = [0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xA8, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert_eq!(expected, &decompressed[..]);
    }
}