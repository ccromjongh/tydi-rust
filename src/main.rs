use serde::Deserialize;
use std::fs;
use std::error::Error;

// Define the data structures based on the JSON schema.
// We use `serde::Deserialize` to automatically derive the deserialization logic.

// Represents a single comment.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    user_id: u32,
    username: String,
}

// Represents a single post.
#[derive(Debug, Deserialize)]
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

#[derive(Debug)]
struct PostsTydi {
    posts: TydiVec<PostExploded>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // This assumes the JSON file is named 'posts.json' and is in the same directory.
    let json_file_path = "posts.json";

    // Read the contents of the JSON file into a string.
    let json_data = fs::read_to_string(json_file_path)
        .expect("Should have been able to read the file");

    // Deserialize the JSON string into our `Posts` data structure.
    let posts: Vec<Post> = serde_json::from_str(&json_data)?;

    // Iterate through the posts and print their titles and authors.
    for post in &posts {
        println!("Title: {}", post.title);
        println!("Author: {}", post.author.username);
        println!("Likes: {}", post.likes);
        println!("Number of Comments: {}\n", post.comments.len());
    }

    Ok(())
}
