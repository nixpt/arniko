# CI

GitHub Actions workflow at `.github/workflows/ci.yml`.

## What it runs

On every push to `main` and every PR targeting `main`:

1. **`fmt-and-lint`** — `cargo fmt --package arniko --package bliss-dom --check` and `cargo clippy -p arniko -p bliss-dom --all-targets -- -D warnings`.
2. **`build-arniko`** — `cargo build -p arniko` and `cargo build -p mustang` (default features, no warnings).
3. **`test`** — `cargo test --lib` on arniko, bliss-dom, and mustang.
4. **`arniko-crush`** — checks out the sibling `nixpt/crush-ast` repo and runs `cargo check -p arniko-crush` + `cargo fmt --package arniko-crush --check` (arniko-crush depends on crush-ast's `crush-lang-sdk`).

## Running locally

The same checks the CI runs can be run locally:

```bash
cargo fmt --package arniko --package bliss-dom --check
cargo clippy -p arniko -p bliss-dom --all-targets -- -D warnings
cargo test -p arniko -p bliss-dom -p mustang --lib
```

For the arniko-crush job, you need a sibling `crush-ast` checkout at `../projects/crush-ast` (or adjust the `working-directory` paths in the workflow).
