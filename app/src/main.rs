fn main() {
    println!(
        "check TS_AUTOGEN_FILE: {}",
        std::env::var("TS_AUTOGEN_FILE").unwrap()
    );

    let r1 = ts_macro::ts_block!({
        let str: number[] = ["Hello", "TS", "World"];
        return str.join();
    });
    println!("{}", r1);

    let r2 = ts_macro::ts_block! {{
        let now = new Date();
        return now.toISOString();
    }};
    println!("{}", r2);
}
