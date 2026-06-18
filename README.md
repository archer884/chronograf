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

## Notes

- Backed by [`quanta`](https://crates.io/crates/quanta) — fast monotonic time.
- Generic over an `Instant` trait, so you can drop in a fake clock for tests.

## Code of Conduct

All conversations and contributions to this project shall adhere to the [Code of Conduct](./CODE_OF_CONDUCT.md).
