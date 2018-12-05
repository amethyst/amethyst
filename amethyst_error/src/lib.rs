//! Contains the `Error` type and company as used by Amethyst.
//!
//! **Note:** This type is not intended to be used outside of Amethyst.
//! If you are integrating a crate from amethyst to use this, it is recommended that you treat this
//! type as an opaque [`std::error::Error`].
//!
//! [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(all(feature = "backtrace", feature = "std"))]
extern crate backtrace;

use self::internal::Backtrace;
use std::borrow::Cow;
use std::error;
use std::fmt;
use std::result;

/// Internal parts of `Error`.
pub enum ErrorKind {
    /// Error is a simple message.
    Message(Cow<'static, str>),
    /// Error is a boxed error.
    Error(Box<dyn error::Error + Send + Sync>),
}

impl error::Error for ErrorKind {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ErrorKind::Message(ref message) => message.fmt(fmt),
            ErrorKind::Error(ref e) => e.fmt(fmt),
        }
    }
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ErrorKind::Message(ref message) => fmt::Debug::fmt(message, fmt),
            ErrorKind::Error(ref e) => fmt::Debug::fmt(e, fmt),
        }
    }
}

impl From<&'static str> for ErrorKind {
    fn from(message: &'static str) -> Self {
        ErrorKind::Message(Cow::from(message))
    }
}

impl From<String> for ErrorKind {
    fn from(message: String) -> Self {
        ErrorKind::Message(Cow::from(message))
    }
}

impl From<Box<dyn error::Error + Send + Sync>> for ErrorKind {
    fn from(error: Box<dyn error::Error + Send + Sync>) -> Self {
        ErrorKind::Error(error)
    }
}

/// The error type used by Amethyst.
///
/// Wraps error diagnostics like messages and other errors, and keeps track of causal chains and
/// backtraces.
pub struct Error {
    error_impl: ErrorKind,
    source: Option<Box<Error>>,
    #[cfg(all(feature = "backtrace", feature = "std"))]
    backtrace: Option<Backtrace>,
}

impl Error {
    /// Default constructor for our error types.
    ///
    /// Wraps anything that is an error in a box.
    pub fn new<E>(error: E) -> Self
    where
        E: 'static + error::Error + Send + Sync,
    {
        Self {
            error_impl: ErrorKind::Error(Box::new(error)),
            source: None,
            #[cfg(all(feature = "backtrace", feature = "std"))]
            backtrace: self::internal::bt::new(),
        }
    }

    /// Construct a new error from a string.
    pub fn from_string<M>(message: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self {
            error_impl: ErrorKind::Message(message.into()),
            source: None,
            #[cfg(all(feature = "backtrace", feature = "std"))]
            backtrace: self::internal::bt::new(),
        }
    }

    /// Get backtrace.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        #[cfg(all(feature = "backtrace", feature = "std"))]
        {
            self.backtrace.as_ref()
        }

        #[cfg(not(all(feature = "backtrace", feature = "std")))]
        {
            None
        }
    }

    /// Get the source of the error.
    ///
    /// # Examples
    ///
    /// The only way to set the source is through [`ResultExt`](ResultExt) using
    /// [`with_context`](ResultExt::with_context).
    ///
    /// ```rust
    /// use amethyst_error::{Error, ResultExt};
    /// use std::io;
    ///
    /// let e = io::Error::new(io::ErrorKind::Other, "wrapped");
    /// let a = Error::new(e);
    ///
    /// let res = Result::Err::<(), Error>(a).with_context(|_| Error::from_string("top"));
    /// let e = res.expect_err("no error");
    ///
    /// assert_eq!("top", e.to_string());
    /// assert_eq!("wrapped", e.source().expect("no source").to_string());
    /// ```
    pub fn source(&self) -> Option<&Error> {
        self.source.as_ref().map(|e| &**e)
    }

    /// Iterate over all causes, including this one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amethyst_error::{Error, ResultExt};
    ///
    /// fn failing_function() -> Result<(), Error> {
    ///     Err(Error::from_string("failing"))
    /// }
    ///
    /// fn other_function() -> Result<(), Error> {
    ///     Ok(failing_function().with_context(|_| Error::from_string("other"))?)
    /// }
    ///
    /// let e = other_function().expect_err("no error");
    ///
    /// let messages = e.causes().map(|e| e.to_string()).collect::<Vec<_>>();
    /// assert_eq!(vec!["other", "failing"], messages);
    /// ```
    pub fn causes(&self) -> Causes<'_> {
        Causes {
            current: Some(self),
        }
    }

    /// Convert into a sized `std::error::Error`.
    ///
    /// This can be useful for integrating with systems that operate on `std::error::Error`.
    ///
    /// **Warning:** This erases most diagnostics in favor of returning only the top error.
    /// `std::error::Error` is expanded further.
    ///
    /// ```rust
    /// # extern crate amethyst_error;
    /// # extern crate failure;
    ///
    /// use amethyst_error::Error;
    /// use failure;
    ///
    /// fn foo() -> Result<u32, Error> {
    ///     Ok(0)
    /// }
    ///
    /// fn bar() -> Result<u32, failure::Error> {
    ///     let v = foo().map_err(Error::into_error)?;
    ///     Ok(v + 1)
    /// }
    /// ```
    pub fn into_error(self) -> ErrorKind {
        self.error_impl
    }
}

/// Blanket implementation.
///
/// Encapsulate errors which are Send + Sync.
impl<T> From<T> for Error
where
    T: 'static + error::Error + Send + Sync,
{
    fn from(value: T) -> Error {
        Error::new(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error_impl.fmt(fmt)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Error")
            .field("error_impl", &self.error_impl)
            .field("source", &self.source)
            .finish()
    }
}

/// Extra convenience functions for results based on core errors.
pub trait ResultExt<T>
where
    Self: Sized,
{
    /// Provide a context for the result in case it is an error.
    ///
    /// The context callback is expected to return a new error, which will replace the given error
    /// and set the replaced error as its [`source`](Error::source).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amethyst_error::{Error, ResultExt};
    ///
    /// fn failing_function() -> Result<(), Error> {
    ///     Err(Error::from_string("failing"))
    /// }
    ///
    /// fn other_function() -> Result<(), Error> {
    ///     Ok(failing_function().with_context(|_| Error::from_string("other"))?)
    /// }
    ///
    /// let e = other_function().expect_err("no error");
    ///
    /// assert_eq!("other", e.to_string());
    /// assert_eq!("failing", e.source().expect("no source").to_string());
    /// assert!(e.source().expect("no source").source().is_none());
    /// ```
    fn with_context<C, D>(self, chain: C) -> Result<T, Error>
    where
        C: FnOnce(&Error) -> D,
        D: Into<Error>;
}

impl<T, E> ResultExt<T> for result::Result<T, E>
where
    E: Into<Error>,
{
    fn with_context<C, D>(self, chain: C) -> Result<T, Error>
    where
        C: FnOnce(&Error) -> D,
        D: Into<Error>,
    {
        match self {
            Err(e) => {
                let e = e.into();
                let mut new = chain(&e).into();
                new.source = Some(Box::new(e));
                Err(new)
            }
            Ok(value) => Ok(value),
        }
    }
}

/// An iterator over all the causes for this error.
///
/// Created using [`Error::causes`](Error::causes).
#[derive(Debug, Clone)]
pub struct Causes<'a> {
    current: Option<&'a Error>,
}

impl<'a> Iterator for Causes<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(e) = self.current {
            self.current = e.source();
            return Some(e);
        }

        None
    }
}

/// Constructs an `Error` using the standard string interpolation syntax.
///
/// ```rust
/// #[macro_use] extern crate amethyst_error;
///
/// fn main() {
///     let err = format_err!("number: {}", 42);
///     assert_eq!("number: 42", err.to_string());
/// }
/// ```
#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => { $crate::Error::from_string(format!($($arg)*)) }
}

/// Constructs an [`Error`](Error) from a string.
pub fn err_msg<M>(message: M) -> Error
where
    M: 'static + Send + Sync + fmt::Debug + fmt::Display,
{
    Error::new(ErrMsgError { message })
}

/// Treat something that can be displayed as an error.
struct ErrMsgError<M> {
    message: M,
}

impl<M> error::Error for ErrMsgError<M> where M: 'static + Send + Sync + fmt::Debug + fmt::Display {}

impl<M> fmt::Display for ErrMsgError<M>
where
    M: fmt::Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(fmt)
    }
}

impl<M> fmt::Debug for ErrMsgError<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(fmt)
    }
}

#[cfg(not(all(feature = "backtrace", feature = "std")))]
mod internal {
    use std::fmt;

    /// Fake internal representation.
    pub struct Backtrace(());

    impl fmt::Debug for Backtrace {
        fn fmt(&self, _fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

#[cfg(all(feature = "backtrace", feature = "std"))]
mod internal {
    pub use backtrace::Backtrace;
    use std::{env, ffi, sync::atomic};

    const RUST_BACKTRACE: &str = "RUST_BACKTRACE";

    /// Test if backtracing is enabled.
    pub fn is_backtrace_enabled<F: Fn(&str) -> Option<ffi::OsString>>(get_var: F) -> bool {
        match get_var(RUST_BACKTRACE) {
            Some(ref val) if val != "0" => true,
            _ => false,
        }
    }

    /// Constructs a new backtrace, if backtraces are enabled.
    fn new() -> Option<Backtrace> {
        static ENABLED: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

        match ENABLED.load(atomic::Ordering::SeqCst) {
            0 => {
                let enabled = is_backtrace_enabled(|var| env::var_os(var));

                ENABLED.store(enabled as usize + 1, atomic::Ordering::SeqCst);

                if !enabled {
                    return None;
                }
            }
            1 => return None,
            _ => {}
        }

        Some(Backtrace::new())
    }
}

#[cfg(test)]
mod tests {
    extern crate failure;

    use super::{Error, ResultExt};

    #[test]
    fn test_error_from_string() {
        assert_eq!("foo", Error::from_string("foo").to_string());
    }

    #[test]
    fn test_error_from_error() {
        use std::io;
        let e = io::Error::new(io::ErrorKind::Other, "i/o other");
        assert_eq!("i/o other", Error::new(e).to_string());
    }

    #[test]
    fn test_result_ext_source() {
        use std::io;

        let e = io::Error::new(io::ErrorKind::Other, "wrapped");
        let a = Error::new(e);

        let res = Result::Err::<(), Error>(a).with_context(|_| Error::from_string("top"));
        let e = res.expect_err("no error");

        assert_eq!("top", e.to_string());
        assert_eq!("wrapped", e.source().expect("no source").to_string());
    }

    #[test]
    fn test_sources() {
        use std::io;

        let e = io::Error::new(io::ErrorKind::Other, "wrapped");
        let a = Error::new(e);

        let res = Result::Err::<(), Error>(a).with_context(|_| Error::from_string("top"));
        let e = res.expect_err("no error");

        let messages = e.causes().map(|e| e.to_string()).collect::<Vec<_>>();

        assert_eq!(vec!["top", "wrapped"], messages);
    }

    #[test]
    fn test_failure_compat() {
        fn foo() -> Result<u32, Error> {
            Ok(0)
        }

        #[allow(unused)]
        fn bar() -> Result<u32, failure::Error> {
            let v = foo().map_err(Error::into_error)?;
            Ok(v + 1)
        }

        let a = Error::from_string("foo");

        // Note: has to be `Send + Sync` for this to work.
        // Also loses all error information _except_ the top error.
        failure::Error::from(a.into_error());
    }
}
