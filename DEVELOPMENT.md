
# Development

`burette` is written in Rust and uses the `cargo` build system.
It uses `rustfmt` and `clippy` for code formatting and linting.

Quick start:
 - `cargo build` builds the project.
 - `cargo run` runs the project.
 - `cargo test` runs the unit tests.
 - `cargo fmt` formats the code.
 - `cargo clippy` lints the code.

## Testing

There is a `Makefile` that should be used to perform the various checks and
tests.

Simply run `make` or `make check` to run all checks and tests.
See the [Makefile](Makefile) for more details.

### Unit Tests

The unit tests are written as part of the source code as is idiomatic in Rust.
They can be run with `cargo test`.

### System Tests

To test the command line interface as a whole, there are a bunch of system
tests in the `system_tests` directory.
See [system_tests/README.md](system_tests/README.md) for more information.
