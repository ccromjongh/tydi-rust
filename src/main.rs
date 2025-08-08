use serde::Deserialize;
use std::fs;
use std::error::Error;

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

#[derive(Debug, Clone)]
struct TydiEl<T> {
    data: Option<T>,
    last: Vec<bool>,
}

#[derive(Debug)]
struct TydiVec<T> {
    data: Vec<TydiEl<T>>,
    n: i8,  // Number of lanes (for throughput)
    d: i8,  // Dimensionality
}

// Complete Tydi representation of our data
#[derive(Debug)]
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

    // Transform to Tydi representation
    let tydi_data = transform_to_tydi(&posts);

    // Print Tydi transformation summary
    print_tydi_summary(&tydi_data);

    Ok(())
}
