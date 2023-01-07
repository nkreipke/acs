use std::io::{ErrorKind, Read};

/// Bitwise reader
pub struct BitReader<R: Read> {
    reader: R,
    current: u64,
    offset: u8
}

impl<R: Read> BitReader<R> {
    pub fn new(reader: R) -> std::io::Result<Self> {
        let mut reader = BitReader {
            reader,
            current: 0,
            offset: 0
        };

        // initialize the first byte
        reader.current = reader.read_next()?;

        Ok(reader)
    }

    /// Read the next bit
    pub fn read_bit(&mut self) -> std::io::Result<bool> {
        self.read_bits(1).map(|x| x == 1)
    }

    /// Read n bits
    pub fn read_bits(&mut self, bits: u8) -> std::io::Result<u32> {
        debug_assert!(bits > 0 && bits < 32);

        let mut current = self.current;
        let offset = self.offset;

        let mut value = (current >> offset) as u64;

        let read_area_width = offset + bits;
        let (crosses_border, next_offset) = (read_area_width >= 64, read_area_width % 64);

        if crosses_border {
            current = self.read_next()?;

            value |= current << (64 - offset);
        }

        self.current = current;
        self.offset = next_offset;

        Ok((value & BIT_WINDOWS[bits as usize]) as u32)
    }

    /// Get the next u64 from the reader
    fn read_next(&mut self) -> std::io::Result<u64> {
        let mut buf_storage = [0_u8; 8];
        let mut buf = &mut buf_storage[..];

        // like read_exact, but without the check if the buffer could be filled completely.
        // leaving zeroes at the end is fine
        while !buf.is_empty() {
            match self.reader.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        Ok(u64::from_le_bytes(buf_storage))
    }
}

macro_rules! def_bitwindow {
    ($id:ident -> $t:ty = $($i:literal),*) => {
        const $id: &[$t] = &[
            $((1 << $i) - 1, )*
        ];
    };
}

// Define an array where the element at N has N bits set
def_bitwindow!(BIT_WINDOWS -> u64 = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn test_read_bits() {
        let input = [0b10110111, 0b01111011, 0b11101111, 0b11011111, 0b11011111, 0b11100000];

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            assert_eq!(true, reader.read_bit().unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            assert_eq!(0b110111, reader.read_bits(6).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            assert_eq!(0b110110111, reader.read_bits(9).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            assert_eq!(0b101110110111, reader.read_bits(12).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            assert_eq!(0b11110111101110110111, reader.read_bits(20).unwrap());
        }
    }

    #[test]
    fn test_read_bits2() {
        let input = [0b01101110, 0b11110111, 0b11011110, 0b10111111, 0b10111111, 0b11000001];

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            reader.read_bit().unwrap();
            assert_eq!(true, reader.read_bit().unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            reader.read_bit().unwrap();
            assert_eq!(0b110111, reader.read_bits(6).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            reader.read_bit().unwrap();
            assert_eq!(0b110110111, reader.read_bits(9).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            reader.read_bit().unwrap();
            assert_eq!(0b101110110111, reader.read_bits(12).unwrap());
        }

        {
            let mut reader = BitReader::new(Cursor::new(input)).unwrap();
            reader.read_bit().unwrap();
            assert_eq!(0b11110111101110110111, reader.read_bits(20).unwrap());
        }
    }
}