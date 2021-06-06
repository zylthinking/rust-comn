use chrono::prelude::*;
use proc_macro::TokenStream;
extern crate chrono;

#[proc_macro]
pub fn compile_time(_: TokenStream) -> TokenStream {
    let dt = Local::now();

    let current = format!(
        "{}{:02}{:02}.{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    );

    let expanded = quote::quote! {
         #current
    };
    expanded.into()
}
