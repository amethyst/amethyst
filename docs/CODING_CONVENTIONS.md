# Coding Conventions

This document outlines coding conventions used in the Amethyst.

We follow the [Rust API Guidelines].
This document only covers topics which aren't already outlined there.

## Terminology

In this document we use the keywords _must_, _should_, _must not_, and _should not_.

These loosely conform to [RFC 2119]. Here is a summary of the keywords used:

- _must_ indicates something that is required.
- _should_ is a recommendation, that can be ignored if there is a good reason.

Adding _not_ inverts the meaning of the keywords.

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

###### Example

```rust
use err_derive::Error;

use std::{error, fmt, io};

#[derive(Debug, Error)]
pub enum MyError {
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
# pub enum Error {
#   PermissionDenied,
#   NotConnected,
#   #[doc(hidden)]
#   __Nonexhaustive,
# }
# 
# fn main() {
#   let e = Error::PermissionDenied;
    match e {
        PermissionDenied => { /*  */ }
        NotConnected => { /*  */ }
        _ => { /*  */ }
    }
# }
```

This pattern guards against any matches being non-exhaustive in case new error variants are added in the future.

#### Use `amethyst_error::Error` for compositional APIs

APIs which compose results from multiple crates _should_ use `amethyst_error::Error`.
This is a generic error type which is capable of boxing any error and annotate it with debugging information.

This must be used when defining APIs which composes errors generically, like with traits.
This is also required when composing errors from other crates, since it is currently the only mechanism available to
communicate backtraces.

###### Do

```rust
# mod example {
    // crate `a`
    // depends on `c`
    mod a {
        use amethyst_error::Error;

        pub mod error {
            use derive_more::Display;

            #[derive(Display, Debug)]
            pub enum Error {
                Problem,
            }

            impl std::error::Error for Error {}
        }

        struct Example;

        impl super::c::Example for Example {
            fn foo() -> Result<u32, amethyst_error::Error> {
                Err(error::Error::Problem.into())
            }
        }
    }

    // crate `b`
    // depends on `c`
    mod b {
        use amethyst_error::Error;

        pub mod error {
            use derive_more::Display;

            #[derive(Display, Debug)]
            pub enum Error {
                Problem,
            }

            impl std::error::Error for Error {}
        }

        struct Example;

        impl super::c::Example for Example {
            fn foo() -> Result<u32, amethyst_error::Error> {
                Err(error::Error::Problem.into())
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
# }
```

###### Don't

```rust
# mod example {
    // crate `a`
    // depends on `c`
    mod a {
        pub mod error {
            pub enum Error {
                Problem,
            }
        }

        struct Example;

        impl super::c::Example for Example {
            fn foo() -> Result<u32, super::c::Error> {
                Err(super::c::Error::A(error::Error::Problem))
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

        impl super::c::Example for Example {
            fn foo() -> Result<u32, super::c::Error> {
                Err(super::c::Error::B(error::Error::Problem))
            }
        }
    }

    // crate `c`
    // depends on `a` and `b`.
    mod c {
        pub enum Error {
            A(super::a::error::Error),
            B(super::b::error::Error),
        }

        pub trait Example {
            fn foo() -> Result<u32, Error>;
        }
    }
# }
```

#### Avoid overloading the default `Result`

We _should not_ import a `Result` that overloads the default import.
Instead, if we are using a `Result` alias, use it through a module (e.g. `io::Result`).

This is a future-proofing pattern that prevents having to deal with conflicting `Result` types during refactoring or
code re-use.
Overloading `Result` also makes it harder to use the default `Result` when it's needed.
This is especially useful when multiple modules export their own `Result` types.

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

Crates _should not_ define their own `Result`.
Instead prefer using `Result` directly with the crate-local error type, like this:

```rust
pub mod error {
    pub enum Error {
        Problem,
    }
}

fn foo() -> Result<u32, error::Error> {
    Ok(42)
}
```

[rfc 2119]: https://www.ietf.org/rfc/rfc2119.txt
[rust api guidelines]: https://rust-lang-nursery.github.io/api-guidelines/about.html
[`err-derive`]: https://crates.io/crates/err-derive
