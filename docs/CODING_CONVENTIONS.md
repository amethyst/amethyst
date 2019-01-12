# Coding Conventions

This document outlines coding conventions used in the Amethyst.

We follow the [Rust API Guidelines].
This document only cover topics which aren't already outlined there.

[Rust API Guidelines]: https://rust-lang-nursery.github.io/api-guidelines/about.html

## Terminology

In this document we use the keywords _must_, _should_, _must not_, and _should not_.

These loosely conform to [RFC 2119]. Here is a summary of the keywords used:

* _must_ indicates something that is required.
* _should_ is a recommendation, that can be ignored if there is a good reason.

Adding _not_ inverts the meaning of the keywords.

[RFC 2119]: https://www.ietf.org/rfc/rfc2119.txt

## Error Handling

#### Defining crate-local errors

Custom errors _must_ be defined in a module called `error`.

The error _must_ implement `std::error::Error`, which in turn requires `std::fmt::Display` and `std::fmt::Debug`).

The `std::fmt::Display` implementation _must not_ format the wrapped error since this is already provided through
`source` (see below).

The error _should_ implement `std::fmt::Debug` through the `Debug` derive (see below), unless this is not supported.

The error _must_ implement `From<T>` conversion traits for any error it wraps (e.g. `From<io::Error>`).
The error _must not_ implement conversion methods from non-error types like `u32`.

A lot of this can be implemented using [`err-derive`], as showcased below.

[`err-derive`]: https://crates.io/crates/err-derive

###### Example

```rust
/// crate::error

use std::{fmt, error, io};
use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// I/O Error.
    #[error(display = "I/O Error")]
    Io(#[cause] io::Error),
    /// Permission denied.
    #[error(display = "Permission Denied")]
    PermissionDenied,
    #[error(display = "Non-exhaustive Error")]
    #[doc(hidden)]
    __Nonexhaustive,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}
```

#### Error enums should include a `__Nonexhaustive` variant

Any enum-based error type _should_ include a non-exhaustive error variant.

This prevents consumers of our API from relying on pattern-matching which is not future proof against added error
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

This informs users of the API to include a catch-all arm when matching against the error:

```rust
match e {
    PermissionDenied => { /*  */ },
    NotConnected => { /*  */ },
    _ => { /*  */ },
}
```

This pattern guards against any matches being non-exhaustive in case new error variants are added in the future.

#### Use `amethyst_error::Error` for compositional APIs

APIs which composes results from multiple crates _should_ use `amethyst_error::Error`.
This is a generic error type which is capable of boxing any error and annotate it with debugging information.

This must be used when defining APIs which composes errors generically, like with traits.
This is also required when composing errors from other crates, since it is currently the only mechanism available to
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

#### Avoid overloading the default `Result`

We _should not_ import a `Result` that overloads the default import.
Instead, if we are using a `Result` alias, use it through a module (e.g. `io::Result`).

This is a future-proofing pattern that prevents having to deal with conflicting `Result` types during refactoring or
code re-use.
Overloading `Result` also makes it harder to use the default `Result` when it's needed.
This is especially useful when multiple modules export their own `Result` types.

Crates _should not_ define their own `Result`.
Instead prefer using `Result` directly with the crate-local error type, like this:

```rust
use crate::error::Error;

fn foo() -> Result<u32, Error> {
    Ok(42)
}
```

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
