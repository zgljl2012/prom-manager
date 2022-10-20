extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn make_hello(_item: TokenStream) -> TokenStream {
    "fn hello() -> u32 { 5 }".parse().unwrap()
}

#[proc_macro_derive(AnswerFn1)]
pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
    "fn hello2() -> u32 { 88 }".parse().unwrap()
}

#[proc_macro_attribute]
pub fn show_streams(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}
