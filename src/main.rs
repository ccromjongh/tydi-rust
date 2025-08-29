use serde::Deserialize;
use std::fs;
use std::error::Error;
use rust_tydi_packages::{binary::TydiBinary, TydiPacket, TydiVec, drilling::*};
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
    let (recovered3, recovered4) = result2.split(12);
    println!("recovered3: {:?} (recovered4: {:?})\n", recovered3, recovered4);

    let number = 123456789u64;
    let tydi_number: TydiBinary = number.into();
    println!("number: {}, tydi: {:?}", number, tydi_number);



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
    let comments_tydi = posts_tydi.drill(|e| e.comments.clone());
    let comment_author = comments_tydi.drill(|e| e.author.username.as_bytes().to_vec());
    let my_var = 5;

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
