use serde::Deserialize;
use std::fs;
use std::error::Error;
use chrono::{DateTime, Utc};
use rust_tydi_packages::{binary::TydiBinary, TydiPacket, TydiVec, drilling::*};
// Define the data structures based on the JSON schema.
// We use `serde::Deserialize` to automatically derive the deserialization logic.

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
struct MyDate(DateTime<Utc>);

// Represents a single comment.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
struct Comment {
    comment_id: u32,
    author: Author,
    content: String,
    created_at: MyDate,
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
    created_at: MyDate,
    updated_at: MyDate,
    tags: Vec<String>,
    likes: u32,
    shares: u32,
    comments: Vec<Comment>,
}

impl From<MyDate> for TydiBinary {
    fn from(value: MyDate) -> Self {
        let temp: u64 = value.0.timestamp() as u64;
        temp.into()
    }
}

// The root data structure, which is a vector of posts.
#[derive(Debug, Deserialize)]
struct Posts(Vec<Post>);

struct MyTypeStreams {
    subStream: Vec<TydiBinary>,
    subStream2: Vec<TydiBinary>
}

struct MyTypeProcessed {
    someProp: TydiBinary, // from bool
    someOtherProp: TydiBinary, // from u8

    streams: MyTypeStreams,
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

    let posts_tydi = posts.convert();
    let posts_binary = posts_tydi.finish(256);
    let tags_tydi = posts_tydi.drill(|e| e.tags.clone()).drill(|e| e.as_bytes().to_vec());
    let comments_tydi = posts_tydi.drill(|e| e.comments.clone());
    let comments_binary = comments_tydi.finish(160);
    let comment_author_tydi = comments_tydi.drill(|e| e.author.username.as_bytes().to_vec());
    let comment_author_binary = comment_author_tydi.finish(8);
    let my_var = 5;

    println!("author stream binary: {:?}", comment_author_binary.iter().map(|e| e.to_string()).collect::<Vec<String>>());
    println!("author stream native: {:?}", posts.iter().flat_map(|e| e.comments.clone()).flat_map(|e| e.author.username.as_bytes().iter().map(|e| format!("{:08b}", e)).collect::<Vec<_>>()).collect::<Vec<_>>());

    /*let exploded_posts: Vec<PostNonVecs> = posts.iter().map(|p| PostNonVecs::from(p.clone())).collect();
    let posts_tydi: TydiVec<PostNonVecs> = exploded_posts.into();
    let comments_tydi: Vec<TydiVec<Comment>> = posts.iter().map(|p| TydiVec::from(p.comments.clone())).collect();
    let comments_tydi2: TydiVec<Comment> = comments_tydi.into();
    let tags_tydi: Vec<TydiVec<u8>> = posts.iter().map(|p| {
        TydiVec::from(
            p.tags.iter().map(|t| TydiVec::from(t.as_str())).collect::<Vec<_>>()
        )
    }).collect();
    let tags_tydi2: TydiVec<u8> = tags_tydi.into();*/

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostNonVecs {
    pub post_id: u32,
    pub author: AuthorNonVecs,
    pub created_at: MyDate,
    pub updated_at: MyDate,
    pub likes: u32,
    pub shares: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostVecs {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub comments: Vec<Comment>,
}

impl From<Post> for TydiBinary {
    fn from(value: Post) -> Self {
        let post_id: TydiBinary = value.post_id.into();
        let author: TydiBinary = value.author.into();
        let created_at: TydiBinary = value.created_at.into();
        let updated_at: TydiBinary = value.updated_at.into();
        let likes: TydiBinary = value.likes.into();
        let shares: TydiBinary = value.shares.into();
        post_id.concatenate(&author).concatenate(&created_at).concatenate(&updated_at).concatenate(&likes).concatenate(&shares)
    }
}

impl From<Author> for TydiBinary {
    fn from(value: Author) -> Self {
        let author_id: TydiBinary = value.user_id.into();
        author_id
    }
}

impl From<Comment> for TydiBinary {
    fn from(value: Comment) -> Self {
        let comment_id: TydiBinary = value.comment_id.into();
        let author: TydiBinary = value.author.into();
        let created_at: TydiBinary = value.created_at.into();
        let likes: TydiBinary = value.likes.into();
        comment_id.concatenate(&author).concatenate(&created_at).concatenate(&likes)
    }
}

impl From<Post> for PostNonVecs { fn from(value: Post) -> Self { Self { post_id: value.post_id, author: value.author.into(), created_at: value.created_at, updated_at: value.updated_at, likes: value.likes, shares: value.shares } } }

impl From<Post> for PostVecs { fn from(value: Post) -> Self { Self { title: value.title, content: value.content, tags: value.tags, comments: value.comments } } }


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
    pub created_at: MyDate,
    pub likes: u32,
    pub in_reply_to_comment_id: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommentVecs {
    pub content: String,
}

impl From<Comment> for CommentNonVecs { fn from(value: Comment) -> Self { Self { comment_id: value.comment_id, author: value.author, created_at: value.created_at, likes: value.likes, in_reply_to_comment_id: value.in_reply_to_comment_id } } }

impl From<Comment> for CommentVecs { fn from(value: Comment) -> Self { Self { content: value.content } } }
