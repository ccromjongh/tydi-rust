use std::mem;
use std::fmt;
use std::fmt::{Debug, Display};
use bytemuck::{bytes_of, cast, cast_slice, from_bytes_mut, NoUninit, Pod};

pub mod drilling;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TydiPacket<T> {
    pub data: Option<T>,
    pub last: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TydiVec<T> {
    pub data: Vec<TydiPacket<T>>,
    d: i8,  // Dimensionality
}

impl<T> TydiVec<T> {
    pub fn new(d: i8) -> Self {
        TydiVec {
            data: Vec::new(),
            d,
        }
    }

    pub fn push(&mut self, data: Option<T>, last: Vec<bool>) {
        self.data.push(TydiPacket { data, last });
    }
}

impl From<&str> for TydiVec<u8> {
    /// Creates a TydiVec from a string.
    fn from(value: &str) -> Self {
        let bytes: &[u8] = value.as_bytes();
        let mut result: Vec<TydiPacket<u8>> = Vec::new();

        // Handle empty strings
        if bytes.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiPacket {
                        data: None,
                        last: vec![true],  // Empty string marker
                    }
                ),
                d: 0
            }
        }

        for (i, &byte) in bytes.iter().enumerate() {
            let is_last_char = i == bytes.len() - 1;

            result.push(TydiPacket {
                data: Some(byte),
                last: vec![is_last_char],
            });
        }

        TydiVec {
            data: result,
            d: 0,
        }
    }
}

impl<T: Clone> From<Vec<T>> for TydiVec<T> {
    /// Creates a TydiVec from any vector.
    fn from(value: Vec<T>) -> Self {
        let mut result: Vec<TydiPacket<T>> = Vec::new();

        // Handle empty sequences
        if value.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiPacket {
                        data: None,
                        last: vec![true],  // Empty sequence marker
                    }
                ),
                d: 0
            }
        }

        for (i, el) in value.iter().enumerate() {
            let is_last_el = i == value.len() - 1;

            result.push(TydiPacket {
                data: Some((*el).clone()),
                last: vec![is_last_el],
            });
        }

        TydiVec {
            data: result,
            d: 0
        }
    }
}

impl<T: Clone> From<Vec<TydiVec<T>>> for TydiVec<T> {
    /// Creates a TydiVec from any vector.
    fn from(value: Vec<TydiVec<T>>) -> Self {
        let mut result: Vec<TydiPacket<T>> = Vec::new();

        // Handle empty sequences
        if value.is_empty() {
            return TydiVec {
                data: vec!(
                    TydiPacket {
                        data: None,
                        last: vec![true, true],  // Fixme how do we know what dimension we should be at here?
                    }
                ),
                d: 0
            }
        }

        for (i, seq) in value.iter().enumerate() {
            let is_last_seq = i == value.len() - 1;

            for (j, el) in seq.data.iter().enumerate() {
                result.push(TydiPacket {
                    data: el.data.clone(),
                    last: [el.last.clone(), vec![is_last_seq]].concat(),
                });
            }
        }

        TydiVec {
            data: result,
            d: 0
        }
    }
}

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

    fn split_for<T: Pod>(&self) -> (T, TydiBinary) {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_from_u32() {
        // let value: u32 = 0x12345678;
        let value = 12345678u64;
        let binary = TydiBinary::from(value);

        assert_eq!(binary.len, 64);
        assert_eq!(binary.data, value.to_ne_bytes().to_vec());

        let binary: TydiBinary = value.into();

        assert_eq!(binary.len, 64);
        assert_eq!(binary.data, value.to_ne_bytes().to_vec());
    }

    #[test]
    fn test_binary_from_f64() {
        let value: f64 = 3.14159;
        let binary = TydiBinary::from(value);

        let val2 = true;

        assert_eq!(binary.len, 64);
        assert_eq!(binary.data, value.to_ne_bytes().to_vec());
    }

    #[test]
    fn test_binary_from_string() {
        let value = 'm';
        let binary = TydiBinary::from(value);

        assert_eq!(binary.len, 8);
        // assert_eq!(binary.data, value.to_string().as_bytes().to_vec());
    }

    #[test]
    fn test_packing() {
        #[derive(Debug, PartialEq, Eq, Clone)]
        struct Comment {
            comment_id: u32,
            author: Author,
            content: String,
            created_at: String,
            likes: u32,
            in_reply_to_comment_id: Option<u32>,
        }

        #[derive(Debug, PartialEq, Eq, Clone)]
        struct Author {
            user_id: u32,
            username: String,
        }

        let data = Comment {
            comment_id: 1,
            author: Author {
                user_id: 789,
                username: "CultureVulture".into()
            },
            content: "Oh, Andalusia is truly magical! Did you get a chance to see any flamenco shows in Seville?".into(),
            created_at: "2025-06-15T12:05:00Z".into(),
            likes: 10,
            in_reply_to_comment_id: None,
        };

        impl From<Author> for TydiBinary {
            fn from(author: Author) -> TydiBinary {
                author.user_id.into()
            }
        }

        impl From<Comment> for TydiBinary {
            fn from(comment: Comment) -> TydiBinary {
                let binaries: Vec<TydiBinary> = vec![
                    comment.comment_id.into(),
                    comment.author.into(),
                    comment.likes.into(),
                ];
                binaries.iter().fold(TydiBinary::empty(), |acc, e| acc.concatenate(e)).clone()
            }
        }

        impl From<TydiBinary> for Comment {
            fn from(bin: TydiBinary) -> Self {
                let (comment_id, remainder) = bin.split_for();
                let (author_id, remainder) = remainder.split_for();
                let (likes, remainder) = remainder.split_for();

                Comment {
                    comment_id,
                    author: Author {
                        user_id: author_id,
                        username: "".to_string(),
                    },
                    content: "".to_string(),
                    created_at: "".to_string(),
                    likes,
                    in_reply_to_comment_id: None,
                }
            }
        }

        let le_value = 1u32.to_le_bytes();
        let be_value = 2u32.to_be_bytes();

        let bin = TydiBinary::from(data);
        println!("{}", bin);
        println!("{:?}", bin);
        let reconstructed: Comment = bin.into();
        println!("{:?}", reconstructed);
        println!("done");
    }
}
