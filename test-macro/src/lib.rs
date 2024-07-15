use proc_macro2::TokenStream;
use quote::quote;
use std::fmt;
use syn::ext::IdentExt;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, ExprLit, Ident, Lit, LitStr, Macro, Token};

// use proc_macro2::TokenStream;
// use syn::{
//     parse::{ParseStream, Parser},
//     Error, Result,
// };
//
// use std::{
//     env,
//     path::{Path, PathBuf},
// };
//
// fn get_tmp_dir() -> PathBuf {
//     let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be defined"));
//     out_dir.join("rust-ts")
// }

fn ts_macro_parse(input: ParseStream) -> Result<TokenStream> {
    todo!()
}

fn ts_macro_impl(tokens: TokenStream) -> TokenStream {
    println!("#### impl ts_macro_impl");
    //
    //     ts_macro_parse
    //         .parse2(tokens)
    //         .unwrap_or_else(Error::into_compile_error)

    // let name = get_tmp_dir();

    quote! {
        println("output path: {}", name);
    }
}

#[proc_macro]
pub fn ts_macro(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ts_macro_impl(tokens.into()).into()
}

#[macro_export]
macro_rules! ts2 {
      // inline closure
      ([$($captures:tt)*] $($rest:tt)*) => {
        {
            #[allow(unused)]
            #[derive($crate::__cpp_internal_closure)]
            enum TsClosureInput {
                Input = (stringify!([$($captures)*] $($rest)*), 0).1
            }
            __cpp_closure_impl![$($captures)*]
        }
    };
}

//
// use quote::quote;
//
// #[test]
// fn test() {
//     assert_eq! {
//         ts_macro_impl(quote!{
//             わたくし std::env::args 様を使わせていただきますわ.
//         }).to_string(),
//         quote!{
//             use std::env::args;
//         }.to_string()
//     };
// }

// The arguments expected by libcore's format_args macro, and as a
// result most other formatting and printing macros like println.
//
//     println!("{} is {number:.prec$}", "x", prec=5, number=0.01)
struct FormatArgs {
    format_string: Expr,
    positional_args: Vec<Expr>,
    named_args: Vec<(Ident, Expr)>,
}

impl Parse for FormatArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let format_string: Expr;
        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();

        format_string = input.parse()?;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            if input.peek(Ident::peek_any) && input.peek2(Token![=]) {
                while !input.is_empty() {
                    let name: Ident = input.call(Ident::parse_any)?;
                    input.parse::<Token![=]>()?;
                    let value: Expr = input.parse()?;
                    named_args.push((name, value));
                    if input.is_empty() {
                        break;
                    }
                    input.parse::<Token![,]>()?;
                }
                break;
            }
            positional_args.push(input.parse()?);
        }

        Ok(FormatArgs {
            format_string,
            positional_args,
            named_args,
        })
    }
}

// Extract the first argument, the format string literal, from an
// invocation of a formatting or printing macro.
fn get_format_string(m: &Macro) -> Result<LitStr> {
    let seg = m.path.segments.first().unwrap();
    eprintln!("##### macro path: {:?}", seg);
    let args: FormatArgs = m.parse_body()?;
    match args.format_string {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => Ok(lit),
        other => {
            // First argument was not a string literal expression.
            // Maybe something like: println!(concat!(...), ...)
            Err(Error::new_spanned(
                other,
                "format string must be a string literal",
            ))
        }
    }
}

#[test]
fn my_test() {
    // let invocation = parse_quote! {
    //     println!("{:?}", Instant::now())
    // };

    let invocation = parse_quote! {
        ts_macro!("{:?}", Instant::now())
    };

    let lit = get_format_string(&invocation).unwrap();
    println!("#### my-testa");
    assert_eq!(lit.value(), "{:!}");
}
