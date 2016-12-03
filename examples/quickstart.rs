// Simple and robust error handling with error-chain!
// Use this as a template for new projects.

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { }
}

use errors::*;

// Wrapper macro for a `main` with `Result` as return type. You may
// want to set the `RUST_BACKTRACE` env variable to see a backtrace.
quick_main!(run);

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<()> {
    use std::fs::File;

    // This operation will fail
    File::open("tretrete")
        .chain_err(|| "unable to open tretrete file")?;

    Ok(())
}

