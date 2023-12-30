# Chronograf

A stopwatch library. Yeah, there are a thousand, and there's something I hate about all of them.

```rust
fn main() {
    let sw = chronograf::start();
    let sum: u64 = (0..=1_000_000).filter(|&n| is_prime(n)).sum();
    let elapsed = sw.finish();
    println!("{sum} / {elapsed:?}");
}
```

## Code of Conduct

All conversations and contributions to this project shall adhere to the [Code of Conduct](./CODE_OF_CONDUCT.md).
