fn main() {
    println!(
        "check TS_AUTOGEN_FILE: {}",
        std::env::var("TS_AUTOGEN_FILE").unwrap()
    );

    let x = ts_macro::ts_block! {{
        const i = 3;
        const j = 4;
        console.log("i * j = ", i * j);
    }};

    let r = ts_macro::ts_block!({
        console.log("hello TS world.");
    });
    assert_eq!(r, "hello TS world.")
}
