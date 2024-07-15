use lazy_static::lazy_static;
use regex::Regex;
use std::{
    env, fmt,
    fs::{create_dir, remove_dir_all, File},
    io::{Read, Write},
    mem::swap,
    path::{Path, PathBuf},
};

use strnom::*;
use ts_macro_common::*;
mod strnom;

fn warnln_impl(a: &str) {
    for s in a.lines() {
        println!("cargo:warning={}", s);
    }
}

macro_rules! warnln {
    ($($all:tt)*) => {
        $crate::warnln_impl(&format!($($all)*));
    }
}

// Like the write! macro, but add the #line directive (pointing to this file).
// Note: the string literal must be on on the same line of the macro
macro_rules! write_add_line {
    ($o:expr, $($e:tt)*) => {
        (|| {
            writeln!($o, "/*line {} \"{}\"*/", line!(), file!().replace('\\', "\\\\"))?;
            write!($o, $($e)*)
        })()
    };
}

fn new_cursor(s: &str) -> Cursor {
    Cursor {
        rest: s,
        off: 0,
        line: 0,
        column: 0,
    }
}

fn skip_literal(mut input: Cursor) -> PResult<bool> {
    //input = whitespace(input)?.0;
    if input.starts_with("\"") {
        input = cooked_string(input.advance(1))?.0;
        debug_assert!(input.starts_with("\""));
        return Ok((input.advance(1), true));
    }
    if input.starts_with("b\"") {
        input = cooked_byte_string(input.advance(2))?.0;
        debug_assert!(input.starts_with("\""));
        return Ok((input.advance(1), true));
    }
    if input.starts_with("\'") {
        input = input.advance(1);
        let cur = cooked_char(input)?.0;
        if !cur.starts_with("\'") {
            return Ok((symbol(input)?.0, true));
        }
        return Ok((cur.advance(1), true));
    }
    if input.starts_with("b\'") {
        input = cooked_byte(input.advance(2))?.0;
        if !input.starts_with("\'") {
            return Err(LexError { line: input.line });
        }
        return Ok((input.advance(1), true));
    }
    lazy_static! {
        static ref RAW: Regex = Regex::new(r##"^b?r#*""##).unwrap();
    }
    if RAW.is_match(input.rest) {
        let q = input.rest.find('r').unwrap();
        input = input.advance(q + 1);
        return raw_string(input).map(|x| (x.0, true));
    }
    Ok((input, false))
}

// advance the cursor until it finds the needle.
fn find_delimited<'a>(mut input: Cursor<'a>, needle: &str) -> PResult<'a, ()> {
    let mut stack: Vec<&'static str> = vec![];
    while !input.is_empty() {
        input = skip_whitespace(input);
        input = skip_literal(input)?.0;
        if input.is_empty() {
            break;
        }
        if stack.is_empty() && input.starts_with(needle) {
            return Ok((input, ()));
        } else if stack.last().map_or(false, |x| input.starts_with(x)) {
            stack.pop();
        } else if input.starts_with("(") {
            stack.push(")");
        } else if input.starts_with("[") {
            stack.push("]");
        } else if input.starts_with("{") {
            stack.push("}");
        } else if input.starts_with(")") || input.starts_with("]") || input.starts_with("}") {
            return Err(LexError { line: input.line });
        }
        input = input.advance(1);
    }
    Err(LexError { line: input.line })
}

fn line_directive(path: &Path, cur: Cursor) -> String {
    let mut line = format!(
        "//line {} \"{}\"\n",
        cur.line + 1,
        path.to_string_lossy().replace('\\', "\\\\")
    );
    for _ in 0..cur.column {
        line.push(' ');
    }
    line
}

#[derive(Debug)]
struct LineError(u32, String);

impl LineError {
    fn add_line(self, a: u32) -> LineError {
        LineError(self.0 + a, self.1)
    }
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0 + 1, self.1)
    }
}

impl From<LexError> for LineError {
    fn from(e: LexError) -> Self {
        LineError(e.line, "Lexing error".into())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
    ParseCannotOpenFile {
        src_path: String,
    },
    ParseSyntaxError {
        src_path: String,
        error: syn::parse::Error,
    },
    LexError {
        src_path: String,
        line: u32,
    },
}

#[derive(Default)]
pub struct Parser {
    pub closures: Vec<Closure>,
    pub callbacks_count: u32,
    current_path: PathBuf, // The current file being parsed
}

impl Parser {
    pub fn parse_source(&mut self, source_file_path: PathBuf) -> Result<(), Error> {
        self.parse_ts_block(source_file_path)
    }

    fn parse_ts_block(&mut self, mod_path: PathBuf) -> Result<(), Error> {
        let mut s = String::new();
        let mut f = File::open(&mod_path).map_err(|_| Error::ParseCannotOpenFile {
            src_path: mod_path.to_str().unwrap().to_owned(),
        })?;
        f.read_to_string(&mut s)
            .map_err(|_| Error::ParseCannotOpenFile {
                src_path: mod_path.to_str().unwrap().to_owned(),
            })?;

        let mut current_path = mod_path;
        swap(&mut self.current_path, &mut current_path);

        self.find_ts_block_macros(&s)?;
        Ok(())
    }

    fn find_ts_block_macros(&mut self, source: &str) -> Result<(), Error> {
        let mut cursor = new_cursor(source);
        while !cursor.is_empty() {
            cursor = skip_whitespace(cursor);
            let r = skip_literal(cursor).map_err(|e| self.lex_error(e))?;
            cursor = r.0;
            if r.1 {
                continue;
            }
            if let Ok((cur, ident)) = symbol(cursor) {
                cursor = cur;
                if ident != "ts_block" {
                    continue;
                }
                cursor = skip_whitespace(cursor);
                if !cursor.starts_with("!") {
                    continue;
                }
                cursor = skip_whitespace(cursor.advance(1));
                let delim = if cursor.starts_with("(") {
                    ")"
                } else if cursor.starts_with("[") {
                    "]"
                } else if cursor.starts_with("{") {
                    "}"
                } else {
                    continue;
                };
                cursor = cursor.advance(1);
                let mut macro_cur = cursor;
                cursor = find_delimited(cursor, delim)
                    .map_err(|e| self.lex_error(e))?
                    .0;
                let size = (cursor.off - macro_cur.off) as usize;
                macro_cur.rest = &macro_cur.rest[..size];

                self.handle_ts_block(macro_cur).unwrap_or_else(|e| {
                    panic!(
                        "Error while parsing ts_block! macro:\n{:?}:{}",
                        self.current_path, e
                    )
                });
                continue;
            }
            if cursor.is_empty() {
                break;
            }
            cursor = cursor.advance(1); // Not perfect, but should work
        }
        Ok(())
    }

    fn lex_error(&self, e: LexError) -> Error {
        Error::LexError {
            src_path: self.current_path.clone().to_str().unwrap().to_owned(),
            line: e.line,
        }
    }

    fn handle_ts_block(&mut self, x: Cursor) -> Result<(), LineError> {
        // Since syn don't give the exact string, we extract manually
        let begin = (find_delimited(x, "{")?.0).advance(1);
        let end = find_delimited(begin, "}")?.0;
        let extracted = &begin.rest[..(end.off - begin.off) as usize];

        let input: ::proc_macro2::TokenStream = x
            .rest
            .parse()
            .map_err(|_| LineError(x.line, "TokenStream parse error".into()))?;
        match ::syn::parse2::<Macro>(input).map_err(|e| LineError(x.line, e.to_string()))? {
            Macro::Closure(mut c) => {
                c.callback_offset = self.callbacks_count;
                c.body_str = line_directive(&self.current_path, begin) + extracted;
                self.closures.push(c);
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref OUT_DIR: PathBuf = PathBuf::from(env::var("OUT_DIR").expect(
        r#"
-- rust-cpp fatal error --

The OUT_DIR environment variable was not set.
NOTE: rustc must be run by Cargo."#
    ));
    static ref TS_AUTOGEN_DIR: PathBuf = OUT_DIR.join("ts_block_macro_test");
    static ref CARGO_MANIFEST_DIR: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect(
        r#"
-- rust-cpp fatal error --

The CARGO_MANIFEST_DIR environment variable was not set.
NOTE: rust-cpp's build function must be run in a build script."#
    ));
}

fn gen_cpp_lib(visitor: &Parser) -> PathBuf {
    let result_path = TS_AUTOGEN_DIR.join("autogen.ts");
    println!("result file path: {}", result_path.to_str().unwrap());
    let mut output = File::create(&result_path).expect("Unable to generate *.ts file");

    for Closure { body_str, sig, .. } in &visitor.closures {
        let name = sig.extern_name();

        #[rustfmt::skip]
        write_add_line!(output, r#"
const {name} = () => {{
    {body}
}}
"#,
            name = &name,
            body = body_str
        )
        .unwrap();
    }

    #[rustfmt::skip]
    write_add_line!(output, r#"
const invoke = (func_name: string) => {{
"#).unwrap();
    for c in visitor.closures.iter() {
        let name = c.sig.extern_name();
        #[rustfmt::skip]
        write_add_line!(output, r#"
    if(func_name === "{name}") {{ return {name}(); }}"#).unwrap();
    }
    #[rustfmt::skip]
    write_add_line!(output, r#"
    return "";
}}

import {{ argv }} from 'node:process';
invoke(argv[2]);"#).unwrap();

    result_path
}

fn clean_artifacts() {
    if TS_AUTOGEN_DIR.is_dir() {
        remove_dir_all(&*TS_AUTOGEN_DIR).expect(
            r#"
-- rust-cpp fatal error --

Failed to remove existing build artifacts from output directory."#,
        );
    }

    create_dir(&*TS_AUTOGEN_DIR).expect(
        r#"
-- rust-cpp fatal error --

Failed to create output object directory."#,
    );
}

/// Run the `cpp` build process on the crate with a root at the given path.
/// Intended to be used within `build.rs` files.
pub fn build<P: AsRef<Path>>(source_path: P) {
    // Clean up any leftover artifacts
    clean_artifacts();

    let mut visitor = Parser::default();
    if let Err(err) = visitor.parse_source(source_path.as_ref().to_owned()) {
        warnln!(
            r#"-- rust-cpp parse error --
There was an error parsing the crate for the rust-cpp build script:
{:?}
In order to provide a better error message, the build script will exit successfully, such that rustc can provide an error message."#,
            err
        );
        return;
    }

    // Generate the C++ library code
    let filename = gen_cpp_lib(&visitor);

    println!(
        "cargo::rustc-env=TS_AUTOGEN_FILE={}",
        filename.to_str().unwrap()
    );
}
