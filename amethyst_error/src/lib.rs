//! Contains the `Error` type and company as used by Amethyst.
//!
//! **Note:** This type is not intended to be used outside of Amethyst.
//! If you are integrating a crate from amethyst to use this, it is recommended that you treat this
//! type as an opaque [`std::error::Error`].
//!
//! [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html

#[cfg(all(feature = "backtrace", feature = "std"))]
extern crate backtrace;

use self::internal::Backtrace;
use std::borrow::Cow;
use std::error;
use std::fmt;
use std::result;

/// Internal parts of `Error`.
enum ErrorImpl {
    Message(Cow<'static, str>),
    Error(Box<dyn error::Error + Send + Sync>),
}

impl fmt::Debug for ErrorImpl {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorImpl::Message(ref message) => fmt::Debug::fmt(message, fmt),
            ErrorImpl::Error(ref e) => fmt::Debug::fmt(e, fmt),
        }
    }
}

impl From<&'static str> for ErrorImpl {
    fn from(message: &'static str) -> Self {
        ErrorImpl::Message(Cow::from(message))
    }
}

impl From<String> for ErrorImpl {
    fn from(message: String) -> Self {
        ErrorImpl::Message(Cow::from(message))
    }
}

impl From<Box<dyn error::Error + Send + Sync>> for ErrorImpl {
    fn from(error: Box<dyn error::Error + Send + Sync>) -> Self {
        ErrorImpl::Error(error)
    }
}

/// The error type used by Amethyst.
///
/// Wraps error diagnostics like messages and other errors, and keeps track of causal chains and
/// backtraces.
pub struct Error {
    error_impl: ErrorImpl,
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
            error_impl: ErrorImpl::Error(Box::new(error)),
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
            error_impl: ErrorImpl::Message(message.into()),
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
    pub fn causes(&self) -> Causes {
        Causes {
            current: Some(self),
        }
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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.error_impl {
            ErrorImpl::Message(ref message) => message.fmt(fmt),
            ErrorImpl::Error(ref e) => e.fmt(fmt),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(fmt)
    }
}

impl<M> fmt::Debug for ErrMsgError<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(fmt)
    }
}

#[cfg(not(all(feature = "backtrace", feature = "std")))]
mod internal {
    use std::fmt;

    /// Fake internal representation.
    pub struct Backtrace(());

    impl fmt::Debug for Backtrace {
        fn fmt(&self, _fmt: &mut fmt::Formatter) -> fmt::Result {
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
}
