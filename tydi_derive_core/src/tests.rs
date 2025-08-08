#![cfg(test)]

use crate::{tydi_derive_impl};
use proc_macro2::TokenStream;
use quote::quote;

#[test]
fn first() {
    let input = quote! {
        struct TestPost {
            post_id: u32,
            title: String,
            content: String,
            author: TestAuthor,
            created_at: String,
            updated_at: String,
            tags: Vec<String>,
            likes: u32,
            shares: u32,
            comments: Vec<TestComment>,
        }
    };

    let after = tydi_derive_impl(input);
    let after_str = after.to_string();
    println!("{}", after_str);
    println!("done");
}
