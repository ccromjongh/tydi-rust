use std::fmt;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TydiEl<T> {
    pub data: Option<T>,
    pub last: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TydiVec<T> {
    pub data: Vec<TydiEl<T>>,
    n: i8,  // Number of lanes (for throughput)
    d: i8,  // Dimensionality
}

impl<T> TydiVec<T> {
    pub fn new(d: i8) -> Self {
        TydiVec {
            data: Vec::new(),
            n: 1,  // Single lane for simplicity
            d,
        }
    }

    pub fn push(&mut self, data: Option<T>, last: Vec<bool>) {
        self.data.push(TydiEl { data, last });
    }
}

impl From<&str> for TydiVec<u8> {
    /// Creates a TydiVec from a string.
    fn from(value: &str) -> Self {
        let bytes: &[u8] = value.as_bytes();
        let mut result: Vec<TydiEl<u8>> = Vec::new();

        // Handle empty strings
        if bytes.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiEl {
                        data: None,
                        last: vec![true],  // Empty string marker
                    }
                ),
                d: 0,
                n: 0
            }
        }

        for (i, &byte) in bytes.iter().enumerate() {
            let is_last_char = i == bytes.len() - 1;

            result.push(TydiEl {
                data: Some(byte),
                last: vec![is_last_char],
            });
        }

        TydiVec {
            data: result,
            n: 0,
            d: 0,
        }
    }
}

impl<T: Clone> From<Vec<T>> for TydiVec<T> {
    /// Creates a TydiVec from any vector.
    fn from(value: Vec<T>) -> Self {
        let mut result: Vec<TydiEl<T>> = Vec::new();

        // Handle empty sequences
        if value.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiEl {
                        data: None,
                        last: vec![true],  // Empty sequence marker
                    }
                ),
                d: 0,
                n: 0
            }
        }

        for (i, el) in value.iter().enumerate() {
            let is_last_el = i == value.len() - 1;

            result.push(TydiEl {
                data: Some((*el).clone()),
                last: vec![is_last_el],
            });
        }

        TydiVec {
            data: result,
            n: 0,
            d: 0,
        }
    }
}

impl<T: Clone> From<Vec<TydiVec<T>>> for TydiVec<T> {
    /// Creates a TydiVec from any vector.
    fn from(value: Vec<TydiVec<T>>) -> Self {
        let mut result: Vec<TydiEl<T>> = Vec::new();

        // Handle empty sequences
        if value.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiEl {
                        data: None,
                        last: vec![true, true],  // Fixme how do we know what dimension we should be at here?
                    }
                ),
                d: 0,
                n: 0
            }
        }

        for (i, seq) in value.iter().enumerate() {
            let is_last_seq = i == value.len() - 1;

            for (j, el) in seq.data.iter().enumerate() {
                result.push(TydiEl {
                    data: el.data.clone(),
                    last: [el.last.clone(), vec![is_last_seq]].concat(),
                });
            }
        }

        TydiVec {
            data: result,
            n: 0,
            d: 0,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TydiBinary {
    pub data: Vec<u8>,
    pub len: usize,
}


impl TydiBinary {
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

    /// Splits this TydiBinary into two new TydiBinary instances at the specified lengths.
    /// Returns a tuple of (TydiBinary, TydiBinary).
    pub fn split(&self, len1: usize, len2: usize) -> (Self, Self) {
        assert_eq!(self.len, len1 + len2, "The sum of the lengths must equal the original length.");

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

            let new_byte = (current_byte_original << start_bit_offset) | (next_byte_original >> (8 - start_bit_offset));
            data2.push(new_byte);
        }

        // Handle the final partial byte of the second part.
        let bin2 = TydiBinary::new(data2, len2);

        (bin1, bin2)
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

        // Format the full bytes
        for i in 0..full_bytes {
            write!(f, "{:08b}", self.data[i])?;
        }

        // Format the last byte with the remaining bits
        if remaining_bits > 0 {
            let mask = (8 - remaining_bits);
            let last_byte = self.data[full_bytes];
            let masked_bits = last_byte >> mask;
            write!(f, "{:0w$b}", masked_bits, w = remaining_bits)?;
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

            // Process full bytes
            for i in 0..full_bytes {
                binary_string.push_str(&format!("{:08b} ", self.data[i]));
                hex_string.push_str(&format!("{:02x} ", self.data[i]));
            }

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
