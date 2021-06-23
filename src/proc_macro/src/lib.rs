use chrono::prelude::*;
use proc_macro::TokenStream;
extern crate chrono;
extern crate proc_macro2;
use syn::{
    parse::{self, Parse, ParseStream},
    Block, Ident, LitInt, Token,
};

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

struct IdentParser(Ident);
impl Parse for IdentParser {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut symbol = String::new();
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let n: LitInt = input.parse()?;

        symbol.push_str(ident.to_string().trim_start_matches("r#"));
        symbol.push('_');
        symbol.push_str(n.to_string().as_str());
        Ok(Self(Ident::new(symbol.as_str(), ident.span())))
    }
}

struct BlockParser(Block);
impl Parse for BlockParser {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

#[proc_macro]
pub fn ident(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as BlockParser);
    let b = input.0;
    let statements = b.stmts;

    mod x {
        use std::sync::atomic::AtomicI32;
        static N: AtomicI32 = AtomicI32::new(0);
        pub fn inc() -> i32 {
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }
    }
    let seq = x::inc();

    let seq = LitInt::new(seq.to_string().as_str(), proc_macro2::Span::call_site());
    let expanded = quote::quote! {
        macro_rules! n {
            ($iden:ident) => {
                ident_num!($iden, #seq)
            }
        }
        #( #statements )*
    };
    expanded.into()
}

#[proc_macro]
pub fn ident_num(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as IdentParser);
    let input = input.0;
    let expanded = quote::quote! {
         #input
    };
    expanded.into()
}
