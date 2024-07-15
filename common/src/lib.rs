use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use proc_macro2::{Span, TokenTree};
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result, Token,
};

// 呼び出す処理のシグネチャ
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ClosureSig {
    pub std_body: String,
}

impl ClosureSig {
    pub fn name_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        // 自身のメンバの状態からハッシュ値を生成
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn extern_name(&self) -> Ident {
        Ident::new(
            &format!("__cpp_closure_{}", self.name_hash()),
            Span::call_site(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub sig: ClosureSig,
    pub body: TokenTree,
    pub body_str: String,
    pub callback_offset: u32,
}

impl Parse for Closure {
    /// Parse the inside of a `cpp!` macro when this macro is a closure.
    /// Example: `unsafe [foo as "int"] -> u32 as "int" { /*... */ }
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Option<Token![unsafe]>>()?;

        let body = input.parse::<TokenTree>()?;
        // Need to filter the spaces because there is a difference between
        // proc_macro2 and proc_macro and the hashes would not match
        let std_body = body
            .to_string()
            .chars()
            .filter(|x| !x.is_whitespace())
            .collect();

        Ok(Closure {
            sig: ClosureSig { std_body },
            body,
            body_str: String::new(),
            callback_offset: 0,
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Macro {
    Closure(Closure),
}

impl Parse for Macro {
    ///! Parse the inside of a `ts_block!` macro (a literal or a closure)
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Macro::Closure(input.parse::<Closure>()?))
    }
}
