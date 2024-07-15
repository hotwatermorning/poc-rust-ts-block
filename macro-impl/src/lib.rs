use quote::quote;
use syn::parse_macro_input;
use ts_macro_common::ClosureSig;

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
    // Parse the macro input
    let input = extract_original_macro(&parse_macro_input!(input as syn::DeriveInput)).unwrap();
    let sig = ClosureSig {
        std_body: format!("{{{}}}", input)
            .chars()
            .filter(|x| !x.is_whitespace())
            .collect(),
    };

    let extern_name = sig.extern_name();

    let call = quote! {
        let output = std::process::Command::new("tsx")
        .args([std::env::var("TS_AUTOGEN_FILE").unwrap(), stringify!(#extern_name).to_string()])
        .output()
        .expect("failed to execute process");
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };

    let result = quote! {
        macro_rules! __ts_block_closure_impl {
            () => ({ #call });
        }
    };
    result.into()
}
