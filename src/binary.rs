use std::mem;
use std::fmt;
use std::fmt::{Debug, Display};
use bytemuck::Pod;

#[derive(Clone, PartialEq, Eq)]
pub struct TydiBinary {
    pub data: Vec<u8>,
    pub len: usize,
}


impl TydiBinary {
    pub fn empty() -> Self {
        Self { data: Vec::new(), len: 0 }
    }

    /// Creates a new TydiBinary struct from a vector of bytes and a bit length.
    pub fn new(data: Vec<u8>, len: usize) -> Self {
        // Simple sanity check to ensure the length is not greater than
        // the capacity of the data vector.
        assert!(len <= data.len() * 8, "Length cannot exceed data capacity");
        Self { data, len }
    }

    /// Concatenates this TydiBinary with another one, returning a new TydiBinary.
    pub fn concatenate(&self, other: &Self) -> Self {
        // If this TydiBinary is empty, the result is simply a clone of the other.
        if self.len == 0 {
            return other.clone();
        }

        // Calculate the total length of the new binary string.
        let new_len = self.len + other.len;

        // Calculate the number of bits already in the last byte of `self`.
        let self_tail_bits = self.len % 8;

        // If `self` is byte-aligned, we can simply extend its data with `other`'s data.
        if self_tail_bits == 0 {
            let mut new_data = self.data.clone();
            new_data.extend_from_slice(&other.data);
            return Self::new(new_data, new_len);
        }

        // The number of bits needed to complete the last byte of `self`.
        let tail_space = 8 - self_tail_bits;

        // Clone the data from the first binary to start building the new vector.
        let mut new_data = self.data.clone();

        // Handle the last byte of `self` and its combination with the first bytes of `other`.
        // This is the core of the non-byte-aligned concatenation.
        for (i, &other_byte) in other.data.iter().enumerate() {
            // Get a mutable reference to the last byte of `new_data`.
            let last_byte = new_data.last_mut().unwrap();

            // Fill the remaining space in the last byte of `self` with bits from `other_byte`.
            let bits_from_other = other_byte >> (8 - tail_space);
            *last_byte |= bits_from_other;

            // If we're not at the end of the `other` data, push the carry-over bits
            // as a new byte. The carry-over bits are the lower `tail_space` bits
            // of the current `other` byte, shifted into a new byte.
            if i < other.data.len() - 1 || (other.len - (i * 8) > tail_space) {
                let carry_over = (other_byte << tail_space);
                new_data.push(carry_over);
            }
        }

        Self::new(new_data, new_len)
    }

    /// Splits this TydiBinary into two new TydiBinary instances at the specified length.
    /// Returns a tuple of (TydiBinary, TydiBinary).
    pub fn split(&self, len1: usize) -> (Self, Self) {
        let len2 = self.len - len1;

        // Part 1: First TydiBinary
        let mut data1 = Vec::new();
        let full_bytes1 = len1 / 8;
        let rem_bits1 = len1 % 8;
        for i in 0..full_bytes1 {
            data1.push(self.data[i]);
        }
        if rem_bits1 > 0 {
            let byte_to_push = self.data[full_bytes1] & (!0u8 << (8 - rem_bits1));
            data1.push(byte_to_push);
        }
        let bin1 = TydiBinary::new(data1, len1);

        // Part 2: Second TydiBinary
        let mut data2 = Vec::new();
        let full_bytes2 = len2 / 8;
        let rem_bits2 = len2 % 8;
        let start_byte_index = len1 / 8;
        let start_bit_offset = len1 % 8;

        // Handle the first, potentially partial, byte
        let mut current_byte = 0;
        if start_bit_offset > 0 {
            let next_byte_index = start_byte_index + 1;
            let current_byte_original = self.data[start_byte_index];
            let next_byte_original = if next_byte_index < self.data.len() {
                self.data[next_byte_index]
            } else {
                0
            };

            let remaining_bits_in_byte = 8 - start_bit_offset;
            current_byte = current_byte_original << start_bit_offset;
            current_byte |= next_byte_original >> remaining_bits_in_byte;
            data2.push(current_byte);
        } else {
            // Start on a byte boundary, so the first byte is just the first byte of the second part.
            if len2 > 0 {
                data2.push(self.data[start_byte_index]);
            }
        }

        // Handle all full bytes after the first partial byte.
        let bytes_to_add = (len2 - (8 - start_bit_offset) % 8 + 7) / 8;
        for i in 0..bytes_to_add {
            let original_byte_index = start_byte_index + if start_bit_offset > 0 { 1 } else { 0 } + i;
            let next_byte_index = original_byte_index + 1;

            let current_byte_original = self.data[original_byte_index];
            let next_byte_original = if next_byte_index < self.data.len() {
                self.data[next_byte_index]
            } else {
                0
            };

            let new_byte = if (start_bit_offset == 0)
            { next_byte_original }
            else
            { (current_byte_original << start_bit_offset) | (next_byte_original >> (8 - start_bit_offset)) };
            data2.push(new_byte);
        }

        // Handle the final partial byte of the second part.
        let bin2 = TydiBinary::new(data2, len2);

        (bin1, bin2)
    }

    pub(crate) fn split_for<T: Pod>(&self) -> (T, TydiBinary) {
        let len = size_of::<T>() * 8;
        let (split1, split2) = self.split(len);
        let val: T = *bytemuck::from_bytes(split1.data.as_slice());
        (val, split2)
    }
}

impl Display for TydiBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Handle empty binary string
        if self.len == 0 {
            return write!(f, "0b");
        }

        // Calculate the number of full bytes and remaining bits
        let full_bytes = self.len / 8;
        let remaining_bits = self.len % 8;

        // Start with the "0b" prefix
        write!(f, "0b")?;

        // We print from last byte to first byte because of the little endian memory layout

        // Format the last byte with the remaining bits
        if remaining_bits > 0 {
            let mask = (8 - remaining_bits);
            let last_byte = self.data[full_bytes];
            let masked_bits = last_byte >> mask;
            write!(f, "{:0w$b}", masked_bits, w = remaining_bits)?;
        }

        // Format the full bytes
        for i in (0..full_bytes).rev() {
            write!(f, "{:08b}", self.data[i])?;
        }

        Ok(())
    }
}

impl Debug for TydiBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create formatted strings for binary and hexadecimal representations
        let mut binary_string = String::new();
        let mut hex_string = String::new();

        // Handle the case where len is 0 to avoid panicking on an empty vector.
        if self.len > 0 {
            // Full bytes
            let full_bytes = self.len / 8;
            let remaining_bits = self.len % 8;

            // We print from last byte to first byte because of the little endian memory layout

            // Process the last, potentially partial, byte
            if remaining_bits > 0 {
                // The number of bits to extract from the last byte is `remaining_bits`.
                // We need to shift the data to the right to get rid of the leading zeros
                // that are not part of the binary string.
                let shift_amount = 8 - remaining_bits;
                let masked_bits = self.data[full_bytes] >> shift_amount;

                // For the hexadecimal representation, we just take the full byte
                // because the hexadecimal string represents the underlying `Vec<u8>`.
                binary_string.push_str(&format!("{:0w$b}", masked_bits, w = remaining_bits));
                hex_string.push_str(&format!("{:02x}", self.data[full_bytes]));
            }

            // Process full bytes
            for i in (0..full_bytes).rev() {
                binary_string.push_str(&format!("{:08b} ", self.data[i]));
                hex_string.push_str(&format!("{:02x} ", self.data[i]));
            }
        }

        // Build the Debug struct representation
        f.debug_struct("TydiBinary")
            .field("len", &self.len)
            .field("data", &self.data)
            .field("binary", &binary_string.trim())
            .field("hex", &hex_string.trim())
            .finish()
    }
}

macro_rules! impl_from_primitive {
    ($($t:ty),*) => {
        $(
            impl From<$t> for TydiBinary {
                fn from(value: $t) -> Self {
                    TydiBinary {
                        data: value.to_ne_bytes().to_vec(),
                        len: mem::size_of::<$t>() * 8,
                    }
                }
            }
        )*
    };
}

impl_from_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

impl From<char> for TydiBinary {
    fn from(value: char) -> Self {
        let val: Vec<u8> = value.to_string().as_bytes().into();
        Self { data: val, len: 8*4 }
    }
}

impl From<bool> for TydiBinary {
    fn from(value: bool) -> Self {
        Self { data: vec![value.into()], len: 1 }
    }
}

impl From<Vec<bool>> for TydiBinary {
    fn from(value: Vec<bool>) -> Self {
        let bit_count = value.len();
        let byte_count = bit_count.div_ceil(8);

        let mut packed_bytes = Vec::with_capacity(byte_count);

        // Iterate over the boolean vector in chunks of 8.
        for chunk in value.chunks(8) {
            let mut byte: u8 = 0;
            let mut bit_position: u8 = 0;

            // Iterate through each boolean in the current chunk.
            for &value in chunk {
                if value {
                    // If the boolean is `true`, set the corresponding bit in the byte.
                    // We use a bitwise OR (`|=`) and left-shift a `1` to the
                    // correct position. `1 << bit_position` creates a byte with
                    // only a single bit set at the correct index.
                    byte |= 1 << bit_position;
                }
                // Move to the next bit position.
                bit_position += 1;
            }

            // Push the completed byte to the result vector.
            packed_bytes.push(byte);
        }
        TydiBinary { data: packed_bytes, len: bit_count }
    }
}
