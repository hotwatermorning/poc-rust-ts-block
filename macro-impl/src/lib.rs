use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Expr, ExprLit, Ident, Lit, LitStr, Macro, Token};

/// Find the occurrence of the `stringify!` macro within the macro derive
fn extract_original_macro(input: &syn::DeriveInput) -> Option<proc_macro2::TokenStream> {
    #[derive(Default)]
    struct Finder(Option<proc_macro2::TokenStream>);
    impl<'ast> syn::visit::Visit<'ast> for Finder {
        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            if mac.path.segments.len() == 1 && mac.path.segments[0].ident == "stringify" {
                self.0 = Some(mac.tokens.clone());
            }
        }
    }
    let mut f = Finder::default();
    syn::visit::visit_derive_input(&mut f, input);
    f.0
}

#[proc_macro_derive(__ts_block_internal_closure)]
#[allow(clippy::cognitive_complexity)]
pub fn expand_internal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let result = quote! {
        macro_rules! __ts_block_closure_impl {
            () => (println!("hello world"))
        }
    };

    result.into()
    //     // Parse the macro input
    //     let input = extract_original_macro(&parse_macro_input!(input as syn::DeriveInput)).unwrap();
    //
    //     let closure = match syn::parse2::<cpp_common::Closure>(input) {
    //         Ok(x) => x,
    //         Err(err) => return err.to_compile_error().into(),
    //     };
    //
    //     let extern_name = closure.sig.extern_name();
    //
    //     let call = quote! {
    //         Command::new("tsx")
    //         .args([""])
    //         .output()
    //         #extern_name(#(#call_args),*);
    //         #[allow(clippy::useless_transmute)]
    //         ::core::mem::transmute::<(), (#ret_ty)>(())
    //     }
    //
    //     let input = proc_macro2::TokenStream::from_iter([closure.body].iter().cloned());
    //     let rust_invocations = find_all_rust_macro.parse2(input).expect("rust! macro");
    //     let init_callbacks = if !rust_invocations.is_empty() {
    //         let rust_cpp_callbacks = Ident::new(
    //             &format!("rust_cpp_callbacks{}", *FILE_HASH),
    //             Span::call_site(),
    //         );
    //         let offset = (flags >> 32) as isize;
    //         let callbacks: Vec<Ident> = rust_invocations.iter().map(|x| x.id.clone()).collect();
    //         quote! {
    //             use ::std::sync::Once;
    //             static INIT_INVOCATIONS: Once = Once::new();
    //             INIT_INVOCATIONS.call_once(|| {
    //                 // #rust_cpp_callbacks is in fact an array. Since we cannot represent it in rust,
    //                 // we just are gonna take the pointer to it can offset from that.
    //                 extern "C" {
    //                     #[no_mangle]
    //                     static mut #rust_cpp_callbacks: *const ::std::os::raw::c_void;
    //                 }
    //                 let callbacks_array : *mut *const ::std::os::raw::c_void = &mut #rust_cpp_callbacks;
    //                 let mut offset = #offset;
    //                 #(
    //                     offset += 1;
    //                     *callbacks_array.offset(offset - 1) = #callbacks as *const ::std::os::raw::c_void;
    //                 )*
    //             });
    //         }
    //     } else {
    //         quote!()
    //     };
    //
    //     let result = quote! {
    //         extern "C" {
    //             #decl
    //         }
    //
    //         macro_rules! __cpp_closure_impl {
    //             (#(#tt_args),*) => {
    //                 {
    //                     #init_callbacks
    //                     #call
    //                 }
    //             }
    //         }
    //     };
    //     result.into()
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
