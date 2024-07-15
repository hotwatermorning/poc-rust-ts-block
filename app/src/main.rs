use ts_macro::ts_block;

fn main() {
    let r1 = ts_block!({
        let str: number[] = ["Hello", "TS", "World"];
        return str.join();
    });
    println!("{}", r1);

    let r2 = ts_block! {{
        let now = new Date();
        return now.toISOString();
    }};
    println!("{}", r2);
}
