# Coding Conventions

This document outlines coding conventions used in the Amethyst.

We follow the [Rust API Guidelines].
This document only cover topics which aren't already outlined there.

[Rust API Guidelines]: https://rust-lang-nursery.github.io/api-guidelines/about.html

## Error Handling

#### Defining crate-local errors

Custom errors _must_ be defined in a module called `error`.

The error _must_ implement `std::error::Error`, which in turn requires `std::fmt::Display` and `std::fmt::Debug`).

The `std::fmt::Display` implementation _must not_ format the wrapped error since this is already provided through
`source` (see below).

The error _should_ implement `std::fmt::Debug` through the `Debug` derive (see below), unless this is not supported.

The error _must_ implement `From<T>` conversion traits for any error it wraps (e.g. `From<io::Error>`).
The error _must not_ implement conversion methods from non-error types like `u32`.

###### Example

```rust
/// crate::error

use std::{fmt, error, io};

#[derive(Debug)]
pub enum Error {
    /// I/O Error.
    IoError(io::Error),
    /// Permission denied.
    PermissionDenied,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::IoError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Error::IoError(_) => write!(fmt, "I/O Error"),
            Error::PermissionDenied => write!(fmt, "Permission Denied"),
            _ => write!(fmt, "Some error has occurred"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}
```

#### Error enums should include a `__Nonexhaustive` variant

Any enum-based error type _should_ include a non-exhaustive error variant.

This prevents consumers of your API from relying on pattern-matching which is not future proof against added error
variants.

###### Example

Implementing a non-exhaustive variant is done like this:

```rust
pub enum Error {
    PermissionDenied,
    NotConnected,
    #[doc(hidden)]
    __Nonexhaustive,
}
```

It informs users of your API that they should include a catch-all match when matching against the error like this:

```rust
match e {
    PermissionDenied => { /*  */ },
    NotConnected => { /*  */ },
    _ => { /*  */ },
}
```

This in turn guards any matches from being non-exhaustive in case new error variants are added in the future.

#### Use `amethyst_error::Error` for compositional APIs

APIs which composes results from multiple crates _should_ use `amethyst_error::Error`.
This is a generic error type which is capable of boxing any error and annotate it with debugging information.

This must be used when defining APIs which composes errors generically, like with traits.
This is also required when composing errors from other crates, since It is currently the only mechanism available to
communicate backtraces.

###### Do

```rust
// crate `a`
// depends on `c`
mod a {
    use amethyst_error::Error;

    pub mod error {
        pub enum Error {
            Problem,
        }
    }

    struct Example;

    impl c::Example for Example {
        fn foo() -> Result<u32, Error> {
            Err(Error::from(error::Error::Problem))
        }
    }
}

// crate `b`
// depends on `c`
mod b {
    use amethyst_error::Error;

    pub mod error {
        pub enum Error {
            Problem,
        }
    }

    struct Example;

    impl c::Example for Example {
        fn foo() -> Result<u32, Error> {
            Err(Error::from(error::Error::Problem))
        }
    }
}

// crate `c`
// no dependencies
mod c {
    use amethyst_error::Error;

    pub trait Example {
        fn foo() -> Result<u32, Error>;
    }
}
```

###### Don't

```rust
// crate `a`
// depends on `c`
mod a {
    pub mod error {
        pub enum Error {
            Problem,
        }
    }

    struct Example;

    impl c::Example for Example {
        fn foo() -> Result<u32, c::Error> {
            Err(c::Error::A(error::Error::Problem))
        }
    }
}

// crate `b`
// depends on `c`
mod b {
    pub mod error {
        pub enum Error {
            Problem,
        }
    }

    struct Example;

    impl c::Example for Example {
        fn foo() -> Result<u32, c::Error> {
            Err(c::Error::B(error::Error::Problem))
        }
    }
}

// crate `c`
// depends on `a` and `b`.
mod c {
    use crate::{a, b};

    pub enum Error {
        A(a::error::Error),
        B(b::error::Error),
    }

    pub trait Example {
        fn foo() -> Result<u32, Error>;
    }
}
```

#### Do not overload the default `Result`

_Do not_ import a `Result` that overloads the default import.
If you are using a`Result` alias, use it through a locally imported module.

This is a future-proofing pattern that prevents dealing with conflicting `Result` types during refactoring or code
re-use.
This is especially important in cases where multiple modules export their own `Result` types simultaneously.

###### Do

```rust
use std::io;

fn foo() -> io::Result<u32> {
    Ok(42)
}
```

###### Don't

```rust
use std::io::Result;

fn foo() -> Result<u32> {
    Ok(42)
}
```