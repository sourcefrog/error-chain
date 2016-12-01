#![warn(missing_docs)]

//! A library for consistent and reliable error handling
//!
//! This crate defines an opinionated strategy for error handling in Rust,
//! built on the following principles:
//!
//! * No error should ever be discarded. This library primarily
//!   makes it easy to "chain" errors with the `chain_err` method.
//! * Introducing new errors is trivial. Simple errors can be introduced
//!   at the error site with just a string.
//! * Handling errors is possible with pattern matching.
//! * Conversions between error types are done in an automatic and
//!   consistent way - `From` conversion behavior is never specified
//!   explicitly.
//! * Errors implement Send.
//! * Errors can carry backtraces.
//!
//! ## Quick start
//!
//! If you just want to set up your new project with error-chain,
//! follow the [quickstart.rs] template, and read this [intro]
//! to error-chain.
//!
//! [quickstart.rs]: https://github.com/brson/error-chain/blob/master/examples/quickstart.rs
//! [intro]: http://brson.github.io/2016/11/30/starting-with-error-chain
//!
//! ## error-chain design principles
//!
//! Similar to other libraries like [error-type] and [quick-error], this
//! library defines a macro, `error_chain!` that declares the types
//! and implementation boilerplate necessary for fulfilling a
//! particular error-handling strategy. Most importantly it defines
//! a custom error type (called `Error` by convention) and the `From`
//! conversions that let the `try!` macro and `?` operator work.
//!
//! This library differs in a few ways from previous error libs:
//!
//! * Instead of defining the custom `Error` type as an enum, it is a
//!   struct containing an `ErrorKind` (which defines the
//!   `description` and `display` methods for the error), an opaque,
//!   optional, boxed `std::error::Error + Send + 'static` object
//!   (which defines the `cause`, and establishes the links in the
//!   error chain), and a `Backtrace`.
//! * The macro also defines a `ResultExt` trait that defines a
//!   `chain_err` method. This method on all `std::error::Error + Send + 'static`
//!   types extends the error chain by boxing the current
//!   error into an opaque object and putting it inside a new concrete
//!   error.
//! * It provides automatic `From` conversions between other error types
//!   defined by the `error_chain!` that preserve type information,
//!   and facilitate seamless error composition and matching of composed
//!   errors.
//! * It provides automatic `From` conversions between any other error
//!   type that hides the type of the other error in the `cause` box.
//! * If `RUST_BACKTRACE` is enabled, it collects a single backtrace at
//!   the earliest opportunity and propagates it down the stack through
//!   `From` and `ResultExt` conversions.
//!
//! To accomplish its goals it makes some tradeoffs:
//!
//! * The split between the `Error` and `ErrorKind` types can make it
//!   slightly more cumbersome to instantiate new (unchained) errors,
//!   requiring an `Into` or `From` conversion; as well as slightly
//!   more cumbersome to match on errors with another layer of types
//!   to match.
//! * Because the error type contains `std::error::Error + Send + 'static` objects,
//!   it can't implement `PartialEq` for easy comparisons.
//!
//! ## Declaring error types
//!
//! Generally, you define one family of error types per crate, though
//! it's also perfectly fine to define error types on a finer-grained
//! basis, such as per module.
//!
//! Assuming you are using crate-level error types, typically you will
//! define an `errors` module and inside it call `error_chain!`:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! mod other_error {
//!     error_chain! {}
//! }
//!
//! error_chain! {
//!     // The type defined for this error. These are the conventional
//!     // and recommended names, but they can be arbitrarily chosen.
//!     // It is also possible to leave this block out entirely, or
//!     // leave it empty, and these names will be used automatically.
//!     types {
//!         Error, ErrorKind, ResultExt, Result;
//!     }
//!
//!     // Without the `Result` wrapper:
//!     //
//!     // types {
//!     //     Error, ErrorKind, ResultExt;
//!     // }
//!
//!     // Automatic conversions between this error chain and other
//!     // error chains. In this case, it will e.g. generate an
//!     // `ErrorKind` variant called `Dist` which in turn contains
//!     // the `rustup_dist::ErrorKind`, with conversions from
//!     // `rustup_dist::Error`.
//!     //
//!     // Optionally, some attributes can be added to a variant.
//!     //
//!     // This section can be empty.
//!     links {
//!         Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
//!     }
//!
//!     // Automatic conversions between this error chain and other
//!     // error types not defined by the `error_chain!`. These will be
//!     // wrapped in a new error with, in this case, the
//!     // `ErrorKind::Temp` variant. The description and cause will
//!     // forward to the description and cause of the original error.
//!     //
//!     // Optionally, some attributes can be added to a variant.
//!     //
//!     // This section can be empty.
//!     foreign_links {
//!         Fmt(::std::fmt::Error);
//!         Io(::std::io::Error) #[cfg(unix)];
//!     }
//!
//!     // Define additional `ErrorKind` variants. The syntax here is
//!     // the same as `quick_error!`, but the `from()` and `cause()`
//!     // syntax is not supported.
//!     errors {
//!         InvalidToolchainName(t: String) {
//!             description("invalid toolchain name")
//!             display("invalid toolchain name: '{}'", t)
//!         }
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! Each section, `types`, `links`, `foreign_links`, and `errors` may
//! be omitted if it is empty.
//!
//! This populates the module with a number of definitions,
//! the most important of which are the `Error` type
//! and the `ErrorKind` type. An example of generated code can be found in the
//! [example_generated](example_generated) module.
//!
//! ## Returning new errors
//!
//! Introducing new error chains, with a string message:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! {}
//! fn foo() -> Result<()> {
//!     Err("foo error!".into())
//! }
//! ```
//!
//! Introducing new error chains, with an `ErrorKind`:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! error_chain! {
//!     errors { FooError }
//! }
//!
//! fn foo() -> Result<()> {
//!     Err(ErrorKind::FooError.into())
//! }
//! ```
//!
//! Note that the return type is the typedef `Result`, which is
//! defined by the macro as `pub type Result<T> =
//! ::std::result::Result<T, Error>`. Note that in both cases
//! `.into()` is called to convert a type into the `Error` type; both
//! strings and `ErrorKind` have `From` conversions to turn them into
//! `Error`.
//!
//! When the error is emitted inside a `try!` macro or behind the
//! `?` operator, the explicit conversion isn't needed; `try!` will
//! automatically convert `Err(ErrorKind)` to `Err(Error)`. So the
//! below is equivalent to the previous:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! { errors { FooError } }
//! fn foo() -> Result<()> {
//!     Ok(try!(Err(ErrorKind::FooError)))
//! }
//!
//! fn bar() -> Result<()> {
//!     Ok(try!(Err("bogus!")))
//! }
//! ```
//!
//! ## The `bail!` macro
//!
//! The above method of introducing new errors works but is a little
//! verbose. Instead we can use the `bail!` macro, which, much like `try!`
//! and `?`, performs an early return with conversions. With `bail!` the
//! previous examples look like:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! { errors { FooError } }
//! fn foo() -> Result<()> {
//!     bail!(ErrorKind::FooError);
//!
//!     Ok(())
//! }
//!
//! fn bar() -> Result<()> {
//!     bail!("bogus!");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Chaining errors
//!
//! To extend the error chain:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {}
//! # error_chain! {}
//! # fn do_something() -> Result<()> { unimplemented!() }
//! # fn test() -> Result<()> {
//! let res: Result<()> = do_something().chain_err(|| "something went wrong");
//! # Ok(())
//! # }
//! ```
//!
//! `chain_err` can be called on any `Result` type where the contained
//! error type implements `std::error::Error + Send + 'static`.  If
//! the `Result` is an `Err` then `chain_err` evaluates the closure,
//! which returns *some type that can be converted to `ErrorKind`*,
//! boxes the original error to store as the cause, then returns a new
//! error containing the original error.
//!
//! ## Matching errors
//!
//! error-chain error variants are matched with simple patterns.
//! `Error` is a tuple struct and it's first field is the `ErrorKind`,
//! making dispactching on error kinds relatively compact:
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! # fn main() {
//! error_chain! {
//!     errors {
//!         InvalidToolchainName(t: String) {
//!             description("invalid toolchain name")
//!             display("invalid toolchain name: '{}'", t)
//!         }
//!     }
//! }
//!
//! match Error::from("error!") {
//!     Error(ErrorKind::InvalidToolchainName(_), _) => { }
//!     Error(ErrorKind::Msg(_), _) => { }
//! }
//! # }
//! ```
//!
//! Chained errors are also matched with (relatively) compact syntax
//!
//! ```
//! # #[macro_use] extern crate error_chain;
//! mod utils {
//!     error_chain! {
//!         errors {
//!             BadStuff {
//!                 description("bad stuff")
//!             }
//!         }
//!     }
//! }
//!
//! mod app {
//!     error_chain! {
//!         links {
//!             Utils(::utils::Error, ::utils::ErrorKind);
//!         }
//!     }
//! }
//!
//!
//! # fn main() {
//! match app::Error::from("error!") {
//!     app::Error(app::ErrorKind::Utils(utils::ErrorKind::BadStuff), _) => { }
//!     _ => { }
//! }
//! # }
//! ```
//!
//! ## Foreign links
//!
//! Errors that do not conform to the same conventions as this library
//! can still be included in the error chain. They are considered "foreign
//! errors", and are declared using the `foreign_links` block of the
//! `error_chain!` macro. `Error`s are automatically created from
//! foreign errors by the `try!` macro.
//!
//! Foreign links and regular links have one crucial difference:
//! `From` conversions for regular links *do not introduce a new error
//! into the error chain*, while conversions for foreign links *always
//! introduce a new error into the error chain*. So for the example
//! above all errors deriving from the `temp::Error` type will be
//! presented to the user as a new `ErrorKind::Temp` variant, and the
//! cause will be the original `temp::Error` error. In contrast, when
//! `rustup_utils::Error` is converted to `Error` the two `ErrorKinds`
//! are converted between each other to create a new `Error` but the
//! old error is discarded; there is no "cause" created from the
//! original error.
//!
//! ## Backtraces
//!
//! If the `RUST_BACKTRACE` environment variable is set to anything
//! but ``0``, the earliest non-foreign error to be generated creates
//! a single backtrace, which is passed through all `From` conversions
//! and `chain_err` invocations of compatible types. To read the
//! backtrace just call the `backtrace()` method.
//!
//! Backtrace generation can be disabled by turning off the `backtrace` feature.
//!
//! ## Iteration
//!
//! The `iter` method returns an iterator over the chain of error boxes.
//!
//! [error-type]: https://github.com/DanielKeep/rust-error-type
//! [quick-error]: https://github.com/tailhook/quick-error

#[cfg(feature = "backtrace")]
extern crate backtrace;

use std::error;
use std::iter::Iterator;
#[cfg(feature = "backtrace")]
use std::sync::Arc;

#[cfg(feature = "backtrace")]
pub use backtrace::Backtrace;
#[cfg(not(feature = "backtrace"))]
/// Dummy type used when the `backtrace` feature is disabled.
pub type Backtrace = ();

#[macro_use]
mod quick_error;
#[macro_use]
mod error_chain;
#[cfg(feature = "example_generated")]
pub mod example_generated;

/// Iterator over the error chain using the `Error::cause()` method.
pub struct ErrorChainIter<'a>(pub Option<&'a error::Error>);

impl<'a> Iterator for ErrorChainIter<'a> {
    type Item = &'a error::Error;

    fn next<'b>(&'b mut self) -> Option<&'a error::Error> {
        match self.0.take() {
            Some(e) => {
                self.0 = e.cause();
                Some(e)
            }
            None => None,
        }
    }
}

/// Returns a backtrace of the current call stack if `RUST_BACKTRACE`
/// is set to anything but ``0``, and `None` otherwise.  This is used
/// in the generated error implementations.
#[cfg(feature = "backtrace")]
#[doc(hidden)]
pub fn make_backtrace() -> Option<Arc<Backtrace>> {
    match std::env::var_os("RUST_BACKTRACE") {
        Some(ref val) if val != "0" => Some(Arc::new(Backtrace::new())),
        _ => None,
    }
}

/// This trait is implemented on all the errors generated by the `error_chain`
/// macro.
pub trait ChainedError: error::Error + Send + 'static {
    /// Associated kind type.
    type ErrorKind;

    /// Creates an error from it's parts.
    #[doc(hidden)]
    fn new(kind: Self::ErrorKind, state: State) -> Self;

    /// Returns the first known backtrace, either from it's State or from one
    /// of the errors from `foreign_links`.
    #[cfg(feature = "backtrace")]
    #[doc(hidden)]
    fn extract_backtrace(e: &(error::Error + Send + 'static)) -> Option<Arc<Backtrace>>;
}

/// Common state between errors.
#[derive(Debug)]
#[doc(hidden)]
pub struct State {
    /// Next error in the error chain.
    pub next_error: Option<Box<error::Error + Send>>,
    /// Backtrace for the current error.
    #[cfg(feature = "backtrace")]
    pub backtrace: Option<Arc<Backtrace>>,
}

impl Default for State {
    fn default() -> State {
        #[cfg(feature = "backtrace")]
        let state = State {
            next_error: None,
            backtrace: make_backtrace(),
        };
        #[cfg(not(feature = "backtrace"))]
        let state = State { next_error: None };
        state
    }
}

impl State {
    /// Creates a new State type
    pub fn new<CE: ChainedError>(e: Box<error::Error + Send>) -> State {
        #[cfg(feature = "backtrace")]
        let state = {
            let backtrace = CE::extract_backtrace(&*e).or_else(make_backtrace);
            State {
                next_error: Some(e),
                backtrace: backtrace,
            }
        };
        #[cfg(not(feature = "backtrace"))]
        let state = State { next_error: Some(e) };

        state
    }

    /// Returns the inner backtrace if present.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        #[cfg(feature = "backtrace")]
        let b = self.backtrace.as_ref().map(|v| &**v);
        #[cfg(not(feature = "backtrace"))]
        let b = None;
        b
    }
}

/// Exits a function early with an error
///
/// The `bail!` macro provides an easy way to exit a function.
/// `bail!(expr)` is equivalent to writing.
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # error_chain! { }
/// # fn main() { }
/// # fn foo() -> Result<()> {
/// # let expr = "";
///     return Err(expr.into());
/// # }
/// ```
///
/// And as shorthand it takes a formatting string ala `println!`:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # error_chain! { }
/// # fn main() { }
/// # fn foo() -> Result<()> {
/// # let n = 0;
/// bail!("bad number: {}", n);
/// # }
/// ```
///
/// # Examples
///
/// Bailing on a custom error:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # fn main() {}
/// error_chain! {
///     errors { FooError }
/// }
///
/// fn foo() -> Result<()> {
///     if bad_condition() {
///         bail!(ErrorKind::FooError);
///     }
///
///     Ok(())
/// }
///
/// # fn bad_condition() -> bool { true }
/// ```
///
/// Bailing on a formatted string:
///
/// ```
/// # #[macro_use] extern crate error_chain;
/// # fn main() {}
/// error_chain! { }
///
/// fn foo() -> Result<()> {
///     if let Some(bad_num) = bad_condition() {
///         bail!("so bad: {}", bad_num);
///     }
///
///     Ok(())
/// }
///
/// # fn bad_condition() -> Option<i8> { None }
/// ```
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}

#[doc(hidden)]
pub mod mock {
    error_chain!{}
}
