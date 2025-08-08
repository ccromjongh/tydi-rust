extern crate proc_macro;

mod tests;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse2, ItemStruct, Data, DeriveInput, Fields, Ident, Type};

pub fn tydi_derive_impl(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    // let input = parse_macro_input!(input as DeriveInput);

    // proc_marco2 version of "parse_macro_input!(input as ItemFn)"
    let input = match parse2::<ItemStruct>(input) {
        Ok(syntax_tree) => syntax_tree,
        Err(error) => return error.to_compile_error(),
    };

    let struct_name = &input.ident; // e.g., User

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect fields for the non-Vec struct and Vec-only struct
    let mut non_vec_fields = Vec::new();
    let mut vec_fields = Vec::new();

    if let Fields::Named(fields_named) = &input.fields {
        for field in &fields_named.named {
            let field_name = field.ident.as_ref().expect("Expected named field");
            let field_type = &field.ty;

            // Check if the type is a Vec
            let is_vec = if let Type::Path(type_path) = field_type {
                type_path.path.segments.last().map_or(false, |segment| {
                    segment.ident == "Vec"
                })
            } else {
                false
            };

            if is_vec {
                vec_fields.push(quote! { pub #field_name: #field_type, });
            } else {
                non_vec_fields.push(quote! { pub #field_name: #field_type, });
            }
        }
    }

    // Generate the non-Vec struct
    let non_vec_struct_name = Ident::new(&format!("{}NonVecs", struct_name), struct_name.span());
    let generated_non_vec_struct = quote! {
        #[derive(Debug, PartialEq, Eq, Clone)] // Add common derives
        pub struct #non_vec_struct_name #ty_generics #where_clause {
            #(#non_vec_fields)*
        }
    };

    // Generate the Vec-only struct
    let vec_struct_name = Ident::new(&format!("{}Vecs", struct_name), struct_name.span());
    let generated_vec_struct = quote! {
        #[derive(Debug, PartialEq, Eq, Clone)] // Add common derives
        pub struct #vec_struct_name #ty_generics #where_clause {
            #(#vec_fields)*
        }
    };

    /*// Generate an impl From for original -> non_vec
    let original_to_non_vec_impl = {
        let field_assignments: Vec<_> = non_vec_fields.iter().filter_map(|field| {
            // Extract the field name from the quote::Tokens
            // This is a bit of a hack and might break with complex field definitions.
            // A more robust solution would involve parsing the field again or storing the ident.
            let field_str = field.to_string();
            let parts: Vec<&str> = field_str.split(':').collect();
            if parts.len() > 0 {
                let name = parts[0].trim_end_matches("pub").trim();
                let ident = Ident::new(name, struct_name.span());
                Some(quote! { #ident: value.#ident })
            } else {
                None
            }
        }).collect();

        if field_assignments.is_empty() {
            quote! {} // No non-vec fields, no impl needed
        } else {
            quote! {
                impl #impl_generics From<#struct_name #ty_generics> for #non_vec_struct_name #ty_generics #where_clause {
                    fn from(value: #struct_name #ty_generics) -> Self {
                        Self {
                            #(#field_assignments),*
                        }
                    }
                }
            }
        }
    };

    // Generate an impl From for original -> vec
    let original_to_vec_impl = {
        let field_assignments: Vec<_> = vec_fields.iter().filter_map(|field| {
            let field_str = field.to_string();
            let parts: Vec<&str> = field_str.split(':').collect();
            if parts.len() > 0 {
                let name = parts[0].trim_end_matches("pub").trim();
                let ident = Ident::new(name, struct_name.span());
                Some(quote! { #ident: value.#ident })
            } else {
                None
            }
        }).collect();

        if field_assignments.is_empty() {
            quote! {} // No vec fields, no impl needed
        } else {
            quote! {
                impl #impl_generics From<#struct_name #ty_generics> for #vec_struct_name #ty_generics #where_clause {
                    fn from(value: #struct_name #ty_generics) -> Self {
                        Self {
                            #(#field_assignments),*
                        }
                    }
                }
            }
        }
    };*/


    // Combine all generated tokens
    let expanded = quote! {
        #generated_non_vec_struct
        #generated_vec_struct
        // #original_to_non_vec_impl
        // #original_to_vec_impl
    };

    expanded.into()
}
