//! Contains the `Error` type and company as used by Amethyst.
//!
//! **Note:** This type is not intended to be used outside of Amethyst.
//! If you are integrating a crate from amethyst to use this, it is recommended that you treat this
//! type as an opaque [`std::error::Error`].
//!
//! [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html

// Parts copied from failure:
// https://github.com/rust-lang-nursery/failure

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use backtrace::Backtrace;
use std::{
    borrow::Cow,
    env, error, fmt, result,
    sync::atomic::{self, AtomicUsize},
};

const RUST_BACKTRACE: &str = "RUST_BACKTRACE";

/// Internal parts of `Error`.
#[derive(Debug)]
struct Inner<T: ?Sized> {
    source: Option<Box<Error>>,
    backtrace: Option<Backtrace>,
    error: T,
}

/// The error type used by Amethyst.
///
/// Wraps error diagnostics like messages and other errors, and keeps track of causal chains and
/// backtraces.
pub struct Error {
    inner: Box<Inner<dyn error::Error + Send + Sync>>,
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
            inner: Box::new(Inner {
                source: None,
                backtrace: new_backtrace(),
                error: Box::new(error),
            }),
        }
    }

    /// Update the source of an error.
    pub fn with_source<S>(mut self, source: S) -> Self
    where
        S: 'static + Into<Error>,
    {
        self.inner.source = Some(Box::new(source.into()));
        self
    }

    /// Construct a new error from a string.
    pub fn from_string<M>(message: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        /// Wrapper for string errors.
        #[derive(Debug)]
        struct StringError(Cow<'static, str>);

        impl fmt::Display for StringError {
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(fmt)
            }
        }

        impl error::Error for StringError {}

        Self {
            inner: Box::new(Inner {
                source: None,
                backtrace: new_backtrace(),
                error: Box::new(StringError(message.into())),
            }),
        }
    }

    /// Get backtrace.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace.as_ref()
    }

    /// Get the source of the error.
    ///
    /// # Examples
    ///
    /// The source can be set using [`with_source`](struct.Error.html#method.with_source) or
    /// through [`ResultExt`](trait.ResultExt.html) using
    /// [`with_context`](trait.ResultExt.html#method.with_context).
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
        self.inner.source.as_deref()
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

    /// Access the internal `std::error::Error` as a trait.
    ///
    /// This can be useful for integrating with systems that operate on `std::error::Error`.
    ///
    /// **Warning:** This erases most diagnostics in favor of returning only the top error.
    /// `std::error::Error` is expanded further.
    pub fn as_error(&self) -> &(dyn error::Error + 'static) {
        &self.inner.error
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
        fmt::Display::fmt(&self.inner.error, fmt)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Error")
            .field("inner", &self.inner)
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
    /// and set the replaced error as its [`source`](struct.Error.html#method.source).
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
                Err(chain(&e).into().with_source(e))
            }
            Ok(value) => Ok(value),
        }
    }
}

/// An iterator over all the causes for this error.
///
/// Created using [`Error::causes`](struct.Error.html#method.causes).
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

/// Constructs an [`Error`](struct.Error.html) from a string.
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

impl<M> error::Error for ErrMsgError<M> where M: fmt::Debug + fmt::Display {}

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

/// Test if backtracing is enabled.
fn is_backtrace_enabled() -> bool {
    match env::var_os(RUST_BACKTRACE) {
        Some(ref val) if val != "0" => true,
        _ => false,
    }
}

// 0: unchecked
// 1: disabled
// 2: enabled
static BACKTRACE_STATUS: AtomicUsize = AtomicUsize::new(0);

/// Constructs a new backtrace, if backtraces are enabled.
fn new_backtrace() -> Option<Backtrace> {
    match BACKTRACE_STATUS.load(atomic::Ordering::Relaxed) {
        0 => {
            let enabled = is_backtrace_enabled();

            BACKTRACE_STATUS.store(enabled as usize + 1, atomic::Ordering::Relaxed);

            if !enabled {
                return None;
            }
        }
        1 => return None,
        _ => {}
    }

    Some(Backtrace::new())
}

#[cfg(test)]
mod tests {
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
        assert_eq!(messages, vec!["top", "wrapped"]);
    }

    #[test]
    fn test_try_compat() {
        use std::io;

        fn foo() -> Result<u32, io::Error> {
            Err(io::Error::new(io::ErrorKind::Other, "foo"))
        }

        fn bar() -> Result<u32, Error> {
            let v = foo().with_context(|_| Error::from_string("bar"))?;
            Ok(v + 1)
        }

        let e = bar().expect_err("no error");
        let messages = e.causes().map(|e| e.to_string()).collect::<Vec<_>>();
        assert_eq!(messages, vec!["bar", "foo"]);
    }

    #[test]
    fn test_with_source() {
        let e = Error::from_string("foo");
        assert_eq!(e.to_string(), "foo");
        assert!(e.source().is_none());

        let e = e.with_source(Error::from_string("bar"));
        assert_eq!(e.to_string(), "foo");
        assert_eq!(e.source().map(|e| e.to_string()), Some(String::from("bar")));
    }

    // Note: all backtrace tests have to be in the same test case since they
    // depend on the state of the global `BACKTRACE_STATUS`.
    #[test]
    fn test_backtrace() {
        use super::BACKTRACE_STATUS;
        use std::sync::atomic;

        BACKTRACE_STATUS.store(2, atomic::Ordering::Relaxed);

        #[allow(warnings)]
        #[inline(never)]
        #[no_mangle]
        fn a_really_unique_name_42() -> Error {
            Error::from_string("an error")
        }

        let e = a_really_unique_name_42();
        let bt = e.backtrace().expect("a backtrace");

        let frame_names = bt
            .frames()
            .iter()
            .flat_map(|f| f.symbols().iter().flat_map(|s| s.name()))
            .map(|n| n.to_string())
            .collect::<Vec<_>>();

        assert!(frame_names
            .iter()
            .any(|n| n.ends_with("a_really_unique_name_42")));

        // Test disabled.
        BACKTRACE_STATUS.store(1, atomic::Ordering::Relaxed);
        assert!(Error::from_string("an error").backtrace().is_none());
    }
}
