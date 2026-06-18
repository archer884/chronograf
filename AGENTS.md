# AGENTS.md

Onboarding notes for agents working on chronograf, a small stopwatch library.

## Purpose

Chronograf provides a minimal, clean stopwatch API for Rust. The design goal is
ergonomics: the most common case (time a block of code, get the elapsed duration)
should be a single call with no ceremony.

## Public API surface

Everything lives in `src/lib.rs`. There are three things to know:

- `chronograf::start()` — the main entry point. Returns an `Sw<quanta::Instant>`
  that is already running. This is what the README example uses and what most
  callers want.
- `Sw<I>` — the stopwatch itself. Holds an accumulated `Duration` and an optional
  start instant. `started == None` means the watch is currently stopped.
- `Instant` trait — abstracts the time source so `Sw` can work with non-`quanta`
  clocks (e.g. fake clocks in tests). `quanta::Instant` is the only impl shipped,
  selected because `quanta` is significantly faster than `std::time::Instant`.

## Lifecycle of an `Sw`

- `new()` / `default()` — create a stopped watch (so the caller can pick a
  custom `Instant` impl).
- `start()` — begins a stopped watch. Calling this on a running watch is a
  no-op (mirrors `stop()`); the in-flight interval is preserved, not discarded.
- `stop()` — freezes the watch, folding the current run into `elapsed`. Calling
  on a stopped watch is a no-op.
- `peek()` — read-only query. Combines `elapsed` with any in-flight interval and
  returns the sum **without mutating the watch**. Use this when you want to know
  the current total without disturbing a running timing.
- `finish()` — `stop()` + consume + return total `elapsed`. This is the
  idiomatic end of a timing block.

`stop()`/`start()` are idempotent in the "wrong" state by design; never add
panics or warnings for double-stop or double-start.

## Conventions for changes

- Keep the public API small. New features should justify themselves against the
  "clean stopwatch" goal stated in the README.
- `Sw` is `Clone` but deliberately **not** `Copy` (see the derive at
  `src/lib.rs:29`). It is a stateful accumulator: implicit copies would
  silently diverge, and giving up `Copy` is what lets `finish()` genuinely
  consume. Don't re-add `Copy`, and don't take `Sw` by value when `&self` /
  `&mut self` works.
- All public methods are `#[inline]` — preserve that for any new public
  methods; this is a hot-path library.
- `Duration` arithmetic goes through the `Instant` trait method
  `saturating_duration_since`, never direct subtraction. Follow that pattern
  when adding time source logic.
- Comments in the codebase are doc-comments on public items only. Do not add
  inline `//` comments unless asked.

## Build / test

- Edition 2024, single dependency (`quanta`).
- Tests live in a `#[cfg(test)] mod tests` block at the bottom of `lib.rs`
  (preferred over a separate file). They use a deterministic `FakeInstant`
  driven by a thread-local nanosecond counter — no real-time sleeps, and safe
  under parallel test execution (each test resets the clock at its start).
  `FakeInstant` is also the only non-`quanta` `Instant` impl, so it doubles as
  proof the trait abstraction works.
- Lint/format with `cargo fmt` and `cargo clippy` before considering work done.
