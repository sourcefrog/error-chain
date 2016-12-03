/// Convenient wrapper to be able to use `try!` and such in the main. You can
/// use it with a separated function:
///
/// ```ignore
/// # #[macro_use] extern crate error_chain;
/// # error_chain! {}
/// quick_main!(run);
///
/// fn run() -> Result<()> {
///     Err("error".into())
/// }
/// ```
///
/// or with a closure:
///
/// ```ignore
/// # #[macro_use] extern crate error_chain;
/// # error_chain! {}
/// quick_main!(|| {
///     Err("error".into())
/// }
/// ```
#[macro_export]
macro_rules! quick_main {
    ($main:expr) => {
        let ret_value: ::std::result::Result<(), _> = $ret_value;
        if let Err(e) = ret_value {
            println!("Error: {}", e);

            for e in e.iter().skip(1) {
                println!("Caused by: {}", e);
            }

            if let Some(backtrace) = e.backtrace() {
                println!("{:?}", backtrace);
            }

            ::std::process::exit(1);
        }
    };
}
