/// Prefer to use `error_chain` instead of this macro.
#[macro_export]
macro_rules! error_chain_processed {
    // Default values for `types`.
    (
        types {}
        $( $rest: tt )*
    ) => {
        error_chain_processed! {
            types {
                Error, ErrorKind, ResultExt, Result;
            }
            $( $rest )*
        }
    };
    // With `Result` wrapper.
    (
        types {
            $error_name:ident, $error_kind_name:ident,
            $result_ext_name:ident, $result_name:ident;
        }
        $( $rest: tt )*
    ) => {
        error_chain_processed! {
            types {
                $error_name, $error_kind_name,
                $result_ext_name;
            }
            $( $rest )*
        }
        /// Convenient wrapper around `std::Result`.
        pub type $result_name<T> = ::std::result::Result<T, $error_name>;
    };
    // Without `Result` wrapper.
    (
        types {
            $error_name:ident, $error_kind_name:ident,
            $result_ext_name:ident;
        }

        links {
            $( $link_variant:ident ( $link_error_path:path, $link_kind_path:path )
               $( #[$meta_links:meta] )*; ) *
        }

        foreign_links {
            $( $foreign_link_variant:ident ( $foreign_link_error_path:path )
               $( #[$meta_foreign_links:meta] )*; )*
        }

        errors {
            $( $error_chunks:tt ) *
        }

    ) => {
        /// The Error type.
        ///
        /// This struct is made of three things:
        ///
        /// - an `ErrorKind` which is used to determine the type of the error.
        /// - a backtrace, generated when the error is created.
        /// - an error chain, used for the implementation of `Error::cause()`.
        #[derive(Debug)]
        pub struct $error_name(
            // The members must be `pub` for `links`.
            /// The kind of the error.
            #[doc(hidden)]
            pub $error_kind_name,
            /// Contains the error chain and the backtrace.
            #[doc(hidden)]
            pub $crate::State,
        );

        impl $crate::ChainedError for $error_name {
            type ErrorKind = $error_kind_name;

            fn new(kind: $error_kind_name, state: $crate::State) -> $error_name {
                $error_name(kind, state)
            }

            impl_extract_backtrace!($error_name
                                    $error_kind_name
                                    $([$link_error_path, $(#[$meta_links])*])*);
        }

        #[allow(dead_code)]
        impl $error_name {
            /// Constructs an error from a kind, and generates a backtrace.
            pub fn from_kind(kind: $error_kind_name) -> $error_name {
                $error_name(
                    kind,
                    $crate::State::default(),
                )
            }

            /// Returns the kind of the error.
            pub fn kind(&self) -> &$error_kind_name {
                &self.0
            }

            /// Iterates over the error chain.
            pub fn iter(&self) -> $crate::ErrorChainIter {
                $crate::ErrorChainIter(Some(self))
            }

            /// Returns the backtrace associated with this error.
            pub fn backtrace(&self) -> Option<&$crate::Backtrace> {
                self.1.backtrace()
            }
        }

        impl ::std::error::Error for $error_name {
            fn description(&self) -> &str {
                self.0.description()
            }

            fn cause(&self) -> Option<&::std::error::Error> {
                match self.1.next_error {
                    Some(ref c) => Some(&**c),
                    None => {
                        match self.0 {
                            $(
                                $(#[$meta_foreign_links])*
                                $error_kind_name::$foreign_link_variant(ref foreign_err) => {
                                    foreign_err.cause()
                                }
                            ) *
                            _ => None
                        }
                    }
                }
            }
        }

        impl ::std::fmt::Display for $error_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        $(
            $(#[$meta_links])*
            impl From<$link_error_path> for $error_name {
                fn from(e: $link_error_path) -> Self {
                    $error_name(
                        $error_kind_name::$link_variant(e.0),
                        e.1,
                    )
                }
            }
        ) *

        $(
            $(#[$meta_foreign_links])*
            impl From<$foreign_link_error_path> for $error_name {
                fn from(e: $foreign_link_error_path) -> Self {
                    $error_name::from_kind(
                        $error_kind_name::$foreign_link_variant(e)
                    )
                }
            }
        ) *

        impl From<$error_kind_name> for $error_name {
            fn from(e: $error_kind_name) -> Self {
                $error_name::from_kind(e)
            }
        }

        impl<'a> From<&'a str> for $error_name {
            fn from(s: &'a str) -> Self {
                $error_name::from_kind(s.into())
            }
        }

        impl From<String> for $error_name {
            fn from(s: String) -> Self {
                $error_name::from_kind(s.into())
            }
        }

        impl ::std::ops::Deref for $error_name {
            type Target = $error_kind_name;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }


        // The ErrorKind type
        // --------------

        quick_error! {
            /// The kind of an error.
            #[derive(Debug)]
            pub enum $error_kind_name {

                /// A convenient variant for String.
                Msg(s: String) {
                    description(&s)
                    display("{}", s)
                }

                $(
                    $(#[$meta_links])*
                    $link_variant(e: $link_kind_path) {
                        description(e.description())
                        display("{}", e)
                    }
                ) *

                $(
                    $(#[$meta_foreign_links])*
                    $foreign_link_variant(err: $foreign_link_error_path) {
                        description(::std::error::Error::description(err))
                        display("{}", err)
                    }
                ) *

                $($error_chunks)*
            }
        }

        $(
            $(#[$meta_links])*
            impl From<$link_kind_path> for $error_kind_name {
                fn from(e: $link_kind_path) -> Self {
                    $error_kind_name::$link_variant(e)
                }
            }
        ) *

        impl<'a> From<&'a str> for $error_kind_name {
            fn from(s: &'a str) -> Self {
                $error_kind_name::Msg(s.to_string())
            }
        }

        impl From<String> for $error_kind_name {
            fn from(s: String) -> Self {
                $error_kind_name::Msg(s)
            }
        }

        impl From<$error_name> for $error_kind_name {
            fn from(e: $error_name) -> Self {
                e.0
            }
        }

        // The ResultExt trait defines the `chain_err` method.

        /// Additional methods for `Result`, for easy interaction with this crate.
        pub trait $result_ext_name<T, E> {
            /// If the `Result` is an `Err` then `chain_err` evaluates the closure,
            /// which returns *some type that can be converted to `ErrorKind`*, boxes
            /// the original error to store as the cause, then returns a new error
            /// containing the original error.
            fn chain_err<F, EK>(self, callback: F) -> ::std::result::Result<T, $error_name>
                where F: FnOnce() -> EK,
                      EK: Into<$error_kind_name>;
        }

        impl<T, E> $result_ext_name<T, E> for ::std::result::Result<T, E> where E: ::std::error::Error + Send + 'static {
            fn chain_err<F, EK>(self, callback: F) -> ::std::result::Result<T, $error_name>
                where F: FnOnce() -> EK,
                      EK: Into<$error_kind_name> {
                self.map_err(move |e| {
                    let state = $crate::State::new::<$error_name>(Box::new(e), );
                    $crate::ChainedError::new(callback().into(), state)
                })
            }
        }


    };
}

/// Internal macro used for reordering of the fields.
#[doc(hidden)]
#[macro_export]
macro_rules! error_chain_processing {
    (
        ({}, $b:tt, $c:tt, $d:tt)
        types $content:tt
        $( $tail:tt )*
    ) => {
        error_chain_processing! {
            ($content, $b, $c, $d)
            $($tail)*
        }
    };
    (
        ($a:tt, {}, $c:tt, $d:tt)
        links $content:tt
        $( $tail:tt )*
    ) => {
        error_chain_processing! {
            ($a, $content, $c, $d)
            $($tail)*
        }
    };
    (
        ($a:tt, $b:tt, {}, $d:tt)
        foreign_links $content:tt
        $( $tail:tt )*
    ) => {
        error_chain_processing! {
            ($a, $b, $content, $d)
            $($tail)*
        }
    };
    (
        ($a:tt, $b:tt, $c:tt, {})
        errors $content:tt
        $( $tail:tt )*
    ) => {
        error_chain_processing! {
            ($a, $b, $c, $content)
            $($tail)*
        }
    };
    ( ($a:tt, $b:tt, $c:tt, $d:tt) ) => {
        error_chain_processed! {
            types $a
            links $b
            foreign_links $c
            errors $d
        }
    };
}

/// This macro is used for handling of duplicated and out-of-order fields. For
/// the exact rules, see `error_chain_processed`.
#[macro_export]
macro_rules! error_chain {
    ( $( $block_name:ident { $( $block_content:tt )* } )* ) => {
        error_chain_processing! {
            ({}, {}, {}, {})
            $($block_name { $( $block_content )* })*
        }
    };
}

/// Macro used to manage the `backtrace` feature.
///
/// See
/// https://www.reddit.com/r/rust/comments/57virt/hey_rustaceans_got_an_easy_question_ask_here/da5r4ti/?context=3
/// for more details.
#[macro_export]
#[doc(hidden)]
#[cfg(feature = "backtrace")]
macro_rules! impl_extract_backtrace {
    ($error_name: ident
     $error_kind_name: ident
     $([$link_error_path: path, $(#[$meta_links: meta])*])*) => {
        fn extract_backtrace(e: &(::std::error::Error + Send + 'static))
            -> Option<::std::sync::Arc<$crate::Backtrace>> {
            if let Some(e) = e.downcast_ref::<$error_name>() {
                return e.1.backtrace.clone();
            }
            $(
                $( #[$meta_links] )*
                {
                    if let Some(e) = e.downcast_ref::<$link_error_path>() {
                        return e.1.backtrace.clone();
                    }
                }
            ) *
            None
        }
    }
}

/// Macro used to manage the `backtrace` feature.
///
/// See
/// https://www.reddit.com/r/rust/comments/57virt/hey_rustaceans_got_an_easy_question_ask_here/da5r4ti/?context=3
/// for more details.
#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "backtrace"))]
macro_rules! impl_extract_backtrace {
    ($error_name: ident
     $error_kind_name: ident
     $([$link_error_path: path, $(#[$meta_links: meta])*])*) => {}
}
