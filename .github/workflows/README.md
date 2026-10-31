# CI

GitHub Actions workflow at `.github/workflows/ci.yml`.

## What it runs

On every push to `main` and every PR targeting `main`:

1. **`fmt-and-lint`** — `cargo fmt --package arniko --package bliss-dom --check` and `cargo clippy -p arniko -p bliss-dom --all-targets -- -D warnings`.
2. **`build-arniko`** — `cargo build -p arniko` and `cargo build -p mustang` (default features, no warnings).
3. **`test`** — `cargo test --lib` on arniko, bliss-dom, and mustang.
4. **`arniko-crush`** — `cargo clippy` + `cargo test -p arniko-crush` on rust 1.95 (arniko-crush depends on crush-ast's `crush-lang-sdk` 0.8 from crates.io; no sibling checkout).

## Running locally

The same checks the CI runs can be run locally:

```bash
cargo fmt --package arniko --package bliss-dom --check
cargo clippy -p arniko -p bliss-dom --all-targets -- -D warnings
cargo test -p arniko -p bliss-dom -p mustang --lib
```

The arniko-crush job needs rust 1.95 (`crush-lang-sdk` 0.8's MSRV); `crush-lang-sdk` comes from crates.io.

Releases: `.github/workflows/publish.yml` publishes `exo-bliss-net`, `arniko` and `arniko-crush` together when a `vX.Y.Z` tag is pushed.
