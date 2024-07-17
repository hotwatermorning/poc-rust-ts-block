# PoC Rust TS Block

Rust 中に TS のコードを埋め込んで実行する仕組みの PoC

（https://docs.rs/cpp/latest/cpp/macro.cpp.html のコードを参考にしている。）


ソースコード
```rust
use ts_macro::ts_block;

fn main() {
    let r1 = ts_block!({
        let str: number[] = ["Hello", "TS", "World"];
        return str.join(" ");
    });
    println!("{}", r1);

    let r2 = ts_block! {{
        let now = new Date();
        return now.toISOString();
    }};
    println!("{}", r2);
}
```

実行結果
```
Hello TS World
2024-07-15T20:33:45.508Z
```

## Prerequisites

* [tsx](https://www.npmjs.com/package/tsx)
* rust 1.78 or later

## How to build

```
cargo run
```
