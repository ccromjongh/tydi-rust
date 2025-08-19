use serde::Deserialize;
use std::fs;
use std::error::Error;
use std::fmt::{self, Display, Debug};

// Define the data structures based on the JSON schema.
// We use `serde::Deserialize` to automatically derive the deserialization logic.

// Represents a single comment.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
struct Comment {
    comment_id: u32,
    author: Author,
    content: String,
    created_at: String,
    likes: u32,
    // The `in_reply_to_comment_id` field is optional, so we use `Option<u32>`.
    in_reply_to_comment_id: Option<u32>,
}

// Represents the author of a post or comment.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
struct Author {
    user_id: u32,
    username: String,
}

// Represents a single post.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
struct Post {
    post_id: u32,
    title: String,
    content: String,
    author: Author,
    created_at: String,
    updated_at: String,
    tags: Vec<String>,
    likes: u32,
    shares: u32,
    comments: Vec<Comment>,
}

// The root data structure, which is a vector of posts.
#[derive(Debug, Deserialize)]
struct Posts(Vec<Post>);

// Tydi data-structures
// Each "exploded" version contains all non-sequence data, the constant length ground types such as numbers and booleans
#[derive(Debug, Clone)]
struct AuthorExploded {
    user_id: u32,
}

#[derive(Debug, Clone)]
struct PostExploded {
    post_id: u32,
    author: AuthorExploded,
    likes: u32,
    shares: u32,
}

#[derive(Debug, Clone)]
struct CommentExploded {
    comment_id: u32,
    author: AuthorExploded,
    likes: u32,
    // The `in_reply_to_comment_id` field is optional, so we use `Option<u32>`.
    in_reply_to_comment_id: Option<u32>,
}

// trait TydiConv {
//
// }

// use Into<String> as TydiConv;
// type TydiConv = Into<String>;

// impl Into<Vec<char>> for &str {
//     fn into(self) -> Vec<char> {
//         todo!()
//     }
// }

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TydiEl<T> {
    data: Option<T>,
    last: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TydiVec<T> {
    data: Vec<TydiEl<T>>,
    n: i8,  // Number of lanes (for throughput)
    d: i8,  // Dimensionality
}

#[derive(Clone, PartialEq, Eq)]
struct TydiBinary {
    data: Vec<u8>,
    len: usize,
}


impl TydiBinary {
    /// Creates a new TydiBinary struct from a vector of bytes and a bit length.
    fn new(data: Vec<u8>, len: usize) -> Self {
        // Simple sanity check to ensure the length is not greater than
        // the capacity of the data vector.
        assert!(len <= data.len() * 8, "Length cannot exceed data capacity");
        Self { data, len }
    }

    /// Concatenates this TydiBinary with another one, returning a new TydiBinary.
    fn concatenate(&self, other: &Self) -> Self {
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
    fn split(&self, len1: usize, len2: usize) -> (Self, Self) {
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

// Complete Tydi representation of our data
#[derive(Debug, Clone)]
struct PostsTydi {
    posts: TydiVec<PostExploded>,
    // String streams for variable-length strings
    post_titles: TydiVec<char>,
    post_contents: TydiVec<char>,
    post_created_ats: TydiVec<char>,
    post_updated_ats: TydiVec<char>,
    post_author_usernames: TydiVec<char>,

    // Tag streams
    tags: TydiVec<char>,

    // Comment streams
    comments: TydiVec<CommentExploded>,
    comment_contents: TydiVec<char>,
    comment_created_ats: TydiVec<char>,
    comment_author_usernames: TydiVec<char>,
}

impl<T> TydiVec<T> {
    fn new(d: i8) -> Self {
        TydiVec {
            data: Vec::new(),
            n: 1,  // Single lane for simplicity
            d,
        }
    }

    fn push(&mut self, data: Option<T>, last: Vec<bool>) {
        self.data.push(TydiEl { data, last });
    }
}

fn string_to_tydi_chars(s: &str, is_last_string: bool) -> Vec<TydiEl<char>> {
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();

    for (i, &ch) in chars.iter().enumerate() {
        let is_last_char = i == chars.len() - 1;
        let last = if is_last_char && is_last_string {
            vec![true]  // Close the string dimension
        } else if is_last_char {
            vec![true]  // Close the string dimension, but not the parent
        } else {
            vec![false]
        };

        result.push(TydiEl {
            data: Some(ch),
            last,
        });
    }

    // Handle empty strings
    if chars.is_empty() {
        result.push(TydiEl {
            data: None,
            last: vec![true],  // Empty string marker
        });
    }

    result
}

fn transform_to_tydi(posts: &[Post]) -> PostsTydi {
    let mut tydi_data = PostsTydi {
        posts: TydiVec::new(1),
        post_titles: TydiVec::new(2),
        post_contents: TydiVec::new(2),
        post_created_ats: TydiVec::new(2),
        post_updated_ats: TydiVec::new(2),
        post_author_usernames: TydiVec::new(2),
        tags: TydiVec::new(2),
        comments: TydiVec::new(2),
        comment_contents: TydiVec::new(3),
        comment_created_ats: TydiVec::new(3),
        comment_author_usernames: TydiVec::new(3),
    };

    // Process posts
    for (post_idx, post) in posts.iter().enumerate() {
        let is_last_post = post_idx == posts.len() - 1;

        // Create exploded post data
        let post_exploded = PostExploded {
            post_id: post.post_id,
            author: AuthorExploded {
                user_id: post.author.user_id,
            },
            likes: post.likes,
            shares: post.shares,
        };

        // Add post to posts stream
        tydi_data.posts.push(
            Some(post_exploded),
            vec![is_last_post]  // Last bit for posts dimension
        );

        // Add strings for this post
        let title_chars = string_to_tydi_chars(&post.title, is_last_post);
        tydi_data.post_titles.data.extend(title_chars);

        let content_chars = string_to_tydi_chars(&post.content, is_last_post);
        tydi_data.post_contents.data.extend(content_chars);

        let created_at_chars = string_to_tydi_chars(&post.created_at, is_last_post);
        tydi_data.post_created_ats.data.extend(created_at_chars);

        let updated_at_chars = string_to_tydi_chars(&post.updated_at, is_last_post);
        tydi_data.post_updated_ats.data.extend(updated_at_chars);

        let username_chars = string_to_tydi_chars(&post.author.username, is_last_post);
        tydi_data.post_author_usernames.data.extend(username_chars);

        // Process tags for this post
        if post.tags.is_empty() {
            // Empty tag sequence
            tydi_data.tags.push(None, vec![true, is_last_post]);
        } else {
            for (tag_idx, tag) in post.tags.iter().enumerate() {
                let is_last_tag = tag_idx == post.tags.len() - 1;
                let tag_chars = string_to_tydi_chars(tag, is_last_tag && is_last_post);

                // Modify the last char of the tag to close the tag dimension
                if let Some(last_char) = tydi_data.tags.data.last_mut() {
                    // This is from the previous tag/post, we need to handle properly
                }

                for (char_idx, mut char_el) in tag_chars.into_iter().enumerate() {
                    if char_idx == 0 && tag_idx > 0 {
                        // Not the first tag, don't close higher dimensions yet
                        char_el.last = vec![false, false];
                    }
                    tydi_data.tags.data.push(char_el);
                }

                // Close tag dimension on last character of each tag
                if let Some(last_char) = tydi_data.tags.data.last_mut() {
                    if last_char.last.len() < 2 {
                        last_char.last.push(is_last_tag && is_last_post);
                    } else {
                        last_char.last[1] = is_last_tag && is_last_post;
                    }
                }
            }
        }

        // Process comments for this post
        if post.comments.is_empty() {
            // Empty comment sequence
            tydi_data.comments.push(None, vec![true, is_last_post]);
        } else {
            for (comment_idx, comment) in post.comments.iter().enumerate() {
                let is_last_comment = comment_idx == post.comments.len() - 1;

                // Create exploded comment data
                let comment_exploded = CommentExploded {
                    comment_id: comment.comment_id,
                    author: AuthorExploded {
                        user_id: comment.author.user_id,
                    },
                    likes: comment.likes,
                    in_reply_to_comment_id: comment.in_reply_to_comment_id,
                };

                // Add comment to comments stream
                tydi_data.comments.push(
                    Some(comment_exploded),
                    vec![is_last_comment, is_last_post]
                );

                // Add strings for this comment
                let comment_content_chars = string_to_tydi_chars(&comment.content, is_last_comment && is_last_post);
                tydi_data.comment_contents.data.extend(comment_content_chars);

                let comment_created_at_chars = string_to_tydi_chars(&comment.created_at, is_last_comment && is_last_post);
                tydi_data.comment_created_ats.data.extend(comment_created_at_chars);

                let comment_username_chars = string_to_tydi_chars(&comment.author.username, is_last_comment && is_last_post);
                tydi_data.comment_author_usernames.data.extend(comment_username_chars);
            }
        }
    }

    tydi_data
}

fn print_tydi_summary(tydi_data: &PostsTydi) {
    println!("=== Tydi Transformation Summary ===");
    println!("Posts stream: {} elements", tydi_data.posts.data.len());
    println!("Post titles stream: {} chars", tydi_data.post_titles.data.len());
    println!("Post contents stream: {} chars", tydi_data.post_contents.data.len());
    println!("Tags stream: {} chars", tydi_data.tags.data.len());
    println!("Comments stream: {} elements", tydi_data.comments.data.len());
    println!("Comment contents stream: {} chars", tydi_data.comment_contents.data.len());

    println!("\n=== Posts Stream Details ===");
    for (i, post_el) in tydi_data.posts.data.iter().enumerate() {
        if let Some(ref post_data) = post_el.data {
            println!("Post {}: ID={}, Last={:?}", i, post_data.post_id, post_el.last);
        } else {
            println!("Post {}: Empty, Last={:?}", i, post_el.last);
        }
    }

    println!("\n=== Comments Stream Details ===");
    for (i, comment_el) in tydi_data.comments.data.iter().enumerate() {
        if let Some(ref comment_data) = comment_el.data {
            println!("Comment {}: ID={}, Last={:?}", i, comment_data.comment_id, comment_el.last);
        } else {
            println!("Comment {}: Empty, Last={:?}", i, comment_el.last);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let bin2 = TydiBinary {
        data: vec![0b10101010, 0b11110000],
        len: 12,
    };
    let bin3 = TydiBinary::new(vec![0xAB, 0xC0], 12);
    let bin4 = TydiBinary::new(vec![0xDE, 0xF0], 16);
    println!("\nDisplay: {}", bin2);
    println!("Debug: {:?}", bin2);
    let result2 = bin3.concatenate(&bin4);
    println!("result2: {:?} (Display: {})\n", result2, result2);
    let (recovered3, recovered4) = result2.split(12, 16);
    println!("recovered3: {:?} (recovered4: {:?})\n", recovered3, recovered4);
    
    // This assumes the JSON file is named 'posts.json' and is in the same directory.
    let json_file_path = "posts.json";

    // Read the contents of the JSON file into a string.
    let json_data = fs::read_to_string(json_file_path)
        .expect("Should have been able to read the file");

    // Deserialize the JSON string into our `Posts` data structure.
    let posts: Vec<Post> = serde_json::from_str(&json_data)?;

    // Print original data summary
    println!("=== Original Data Summary ===");
    for post in &posts {
        println!("Title: {}", post.title);
        println!("Author: {}", post.author.username);
        println!("Likes: {}", post.likes);
        println!("Tags: {:?}", post.tags);
        println!("Number of Comments: {}\n", post.comments.len());
    }

    let exploded_posts: Vec<PostNonVecs> = posts.iter().map(|p| PostNonVecs::from(p.clone())).collect();
    let posts_tydi: TydiVec<PostNonVecs> = exploded_posts.into();
    let comments_tydi: Vec<TydiVec<Comment>> = posts.iter().map(|p| TydiVec::from(p.comments.clone())).collect();
    let comments_tydi2: TydiVec<Comment> = comments_tydi.into();
    let tags_tydi: Vec<TydiVec<u8>> = posts.iter().map(|p| {
        TydiVec::from(
            p.tags.iter().map(|t| TydiVec::from(t.as_str())).collect::<Vec<_>>()
        )
    }).collect();
    let tags_tydi2: TydiVec<u8> = tags_tydi.into();

    // Transform to Tydi representation
    let tydi_data = transform_to_tydi(&posts);

    // Print Tydi transformation summary
    print_tydi_summary(&tydi_data);

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostNonVecs {
    pub post_id: u32,
    pub author: AuthorNonVecs,
    pub likes: u32,
    pub shares: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostVecs {
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub comments: Vec<Comment>,
}

impl From<Post> for PostNonVecs { fn from(value: Post) -> Self { Self { post_id: value.post_id, author: value.author.into(), likes: value.likes, shares: value.shares } } }

impl From<Post> for PostVecs { fn from(value: Post) -> Self { Self { title: value.title, content: value.content, created_at: value.created_at, updated_at: value.updated_at, tags: value.tags, comments: value.comments } } }


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthorNonVecs {
    pub user_id: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthorVecs {
    pub username: String,
}

impl From<Author> for AuthorNonVecs { fn from(value: Author) -> Self { Self { user_id: value.user_id } } }

impl From<Author> for AuthorVecs { fn from(value: Author) -> Self { Self { username: value.username } } }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommentNonVecs {
    pub comment_id: u32,
    pub author: Author,
    pub likes: u32,
    pub in_reply_to_comment_id: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommentVecs {
    pub content: String,
    pub created_at: String,
}

impl From<Comment> for CommentNonVecs { fn from(value: Comment) -> Self { Self { comment_id: value.comment_id, author: value.author, likes: value.likes, in_reply_to_comment_id: value.in_reply_to_comment_id } } }

impl From<Comment> for CommentVecs { fn from(value: Comment) -> Self { Self { content: value.content, created_at: value.created_at } } }
