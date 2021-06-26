use chrono::prelude::*;
use proc_macro::TokenStream;
use proc_macro2::Span;
use std::sync::atomic::AtomicI32;
use syn::{
    parse::{Parse, ParseStream},
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

enum Parser {
    _Block(Block),
    _Ident(Ident),
}

impl Parser {
    fn parse_ident(input: ParseStream) -> syn::Result<Ident> {
        let mut symbol = String::new();
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let n: LitInt = input.parse()?;

        symbol.push_str(ident.to_string().trim_start_matches("r#"));
        symbol.push('_');
        symbol.push_str(n.to_string().as_str());
        Ok(Ident::new(symbol.as_str(), ident.span()))
    }
}

impl Parse for Parser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(Ident) {
            return Ok(Parser::_Block(input.parse()?));
        }
        Ok(Parser::_Ident(Parser::parse_ident(input)?))
    }
}

fn id32() -> i32 {
    static N: AtomicI32 = AtomicI32::new(0);
    N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[proc_macro]
pub fn _ident(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Parser);
    match input {
        Parser::_Block(b) => {
            let stmts = b.stmts;
            let seq = LitInt::new(&id32().to_string(), Span::call_site());
            let expanded = quote::quote! {
                macro_rules! n {
                    ($iden:ident) => {
                        $crate::_ident!($iden, #seq)
                    }
                }
                #(#stmts)*
            };
            expanded.into()
        }

        Parser::_Ident(i) => {
            let expanded = quote::quote! {
                 #i
            };
            expanded.into()
        }
    }
}
