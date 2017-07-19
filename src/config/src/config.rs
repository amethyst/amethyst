
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io;
use std::fmt;

/// Error related to anything that manages/creates configurations as well as
/// "workspace"-related things.
#[derive(Debug)]
pub enum ConfigError {
    /// Forward to the `std::io::Error` error.
    File(io::Error),
    /// Errors related to serde's parsing of configuration files.
    Parser(String),
    /// Related to the path of the file.
    Extension(PathBuf),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::File(ref err) => write!(f, "{}", err),
            ConfigError::Parser(ref msg) => write!(f, "{}", msg),
            ConfigError::Extension(ref path) => {
                let found = match path.extension() {
                    Some(extension) => format!("{:?}", extension),
                    None => format!("a directory."),
                };

                write!(
                    f,
                    "{}: Invalid path extension, expected \"yml\", \"yaml\", or \"toml\". Got {}. ", 
                    path.display().to_string(),
                    found,
                )
            },
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::File(e)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::File(_) => "Project file error",
            ConfigError::Parser(_) => "Project parser error",
            ConfigError::Extension(_) => "Invalid extension or directory for a file",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ConfigError::File(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Trait implemented by the `config!` macro.
pub trait Config where Self: Sized {
    /// Loads a configuration structure from a file.
    /// Defaults if the file fails in any way.
    fn load<P: AsRef<Path>>(P) -> Self;

    /// Loads a configuration structure from a file.
    fn load_no_fallback<P: AsRef<Path>>(P) -> Result<Self, ConfigError>;

    /// Writes a configuration structure to a file.
    fn write<P: AsRef<Path>>(&self, P) -> Result<(), ConfigError>;
}

/// The `config!` macro allows defining configuration structures that can load in
/// `.yml`/`.yaml`/`.toml` files.
///
/// It implements the [`serde::Serialize`], [`serde::Deserialize`], and [`Config`] traits.
/// As well as the standard libraries `std::default::Default` trait for the defaulting fields.
///
/// It follows Rust's syntax for defining structures with one addition: defaulting values.
/// If the file does not contain a field of the configuration, then it will print out
/// an error describing the issue and then load in this default value.
///
/// In the case that the file is not found, or the configuration does not exist at all
/// inside the file, then the configuration will have all default values.
///
/// Note: Documentation must be put on the line before the field, rather than on the same line.
///
///## Example Usage:
/// 
/// ```rust
/// #[macro_use]
/// extern crate amethyst_config;
///
///config!(
///    pub struct DisplayConfig {
///        pub title: String = "Amethyst Game".to_string(),
///
///        /// Resolution of the window size at start.
///        pub resolution: (u32, u32) = (1920, 1080),
///        
///        pub fullscreen: bool = false,
///    }
///);
///# fn main(){}
/// ```
///
/// [`serde::Serialize`]: ../serde/trait.Serialize.html
/// [`serde::Deserialize`]: ../serde/trait.Deserialize.html
/// [`Config`]: ../amethyst/project/config/trait.Config.html
#[macro_export]
macro_rules! config(
    (
        // public configuration structure
        $(#[$identifier_meta:meta])*
        pub struct $identifier:ident {
            $(
                $(#[$field_meta:meta])*
                pub $field:ident: $ty:ty = $default:expr $(,)*
            )*
        }
    ) => {
        $( #[$identifier_meta] )*
        pub struct $identifier {
            $(
                $( #[$field_meta] )*
                ///
                pub $field: $ty,
            )*
        }

        // Implements custom `Deserialize` for configuration structures
        config!(
            impl $identifier {
                $(
                    $field: $ty = $default,
                )*
            }
        );
    };
    (
        // private configuration structure
        $(#[$identifier_meta:meta])*
        struct $identifier:ident {
            $(
                $(#[$field_meta:meta])*
                pub $field:ident: $ty:ty = $default:expr $(,)*
            )*
        }
    ) => {
        $( #[$identifier_meta] )*
        struct $identifier {
            $(
                $( #[$field_meta] )*
                pub $field: $ty,
            )*
        }

        // Implements custom `Deserialize` for configuration structures
        config!(
            impl $identifier {
                $(
                    $field: $ty = $default,
                )*
            }
        );
    };
    (
        // Implements the configuration structure's methods.
        impl $identifier:ident {
            $(
                $field:ident: $ty:ty = $default:expr,
            )*
        }
    ) => {
        impl $crate::serde::ser::Serialize for $identifier {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: $crate::serde::ser::Serializer,
            {
                use $crate::serde::ser::SerializeStruct;

                // Find number of fields.
                let mut fields = 0;
                $(
                    fields += 1;
                    let _ = stringify!($field);
                )*

                let mut state = serializer.serialize_struct(stringify!($identifier), fields)?;
                
                $(
                    state.serialize_field(stringify!($field), &self.$field)?;
                )*

                state.end()
            }
        }

        impl<'de> $crate::serde::de::Deserialize<'de> for $identifier {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: $crate::serde::de::Deserializer<'de>
            {
                const FIELDS: &'static [&'static str] = &[ $( stringify!($field), )* ];
                const CONFIG_NAME: &'static str = stringify!($identifier);

                #[allow(non_camel_case_types)]
                enum Field {
                    $(
                        $field,
                    )*
                }

                impl<'de> $crate::serde::de::Deserialize<'de> for Field {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where D: $crate::serde::de::Deserializer<'de>
                    {
                        struct FieldVisitor;

                        impl<'de> $crate::serde::de::Visitor<'de> for FieldVisitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                write!(formatter, "one of ")?;

                                $(
                                    write!(formatter, "`{}`, ", stringify!($field))?;
                                )*

                                Ok(())
                            }

                            fn visit_str<E>(self, value: &str) -> Result<Field, E>
                                where E: $crate::serde::de::Error,
                            {
                                match value {
                                    $(
                                        stringify!($field) => Ok(Field::$field),
                                    )*
                                    _ => {
                                        Err($crate::serde::de::Error::unknown_field(value, FIELDS))
                                    },
                                }
                            }
                        }

                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }

                struct IdentifierVisitor;
                impl<'de> $crate::serde::de::Visitor<'de> for IdentifierVisitor {
                    type Value = $identifier;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(formatter, "a {} configuration", stringify!($identifier))
                    }

                    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                        where S: $crate::serde::de::SeqAccess<'de>,
                    {
                        let result = $identifier {
                            $(
                                $field: {
                                    match seq.next_element() {
                                        Ok(opt) => match opt {
                                            Some(v) => v,
                                            None => {
                                                $default
                                            },
                                        },
                                        Err(err) => {
                                            println!("{}: {}, expecting: `{}`", CONFIG_NAME, err, stringify!($ty));
                                            $default
                                        },
                                    }
                                },
                            )*
                        };
                        Ok(result)
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                        where M: $crate::serde::de::MapAccess<'de>,
                    {
                        $(
                            let mut $field: Option<$ty> = None; // allows checking for duplicates
                        )*

                        while let Some(key) = map.next_key::<Field>()? {
                            match key {
                                $(
                                    Field::$field => {
                                        if $field.is_some() {
                                            use $crate::serde::de::Error;
                                            let err: M::Error = M::Error::duplicate_field(stringify!($field));
                                            println!("{}: {}", CONFIG_NAME, err);
                                        }

                                        $field = match map.next_value() {
                                            Ok(v) => Some(v),
                                            Err(err) => {
                                                println!("{}: {}, expecting: `{}`", CONFIG_NAME, err, stringify!($ty));
                                                Some($default)
                                            },
                                        };
                                    },
                                )*
                            }
                        }

                        $(
                            let $field = match $field {
                                Some(v) => v,
                                None => {
                                    use $crate::serde::de::Error;
                                    let err: M::Error = M::Error::missing_field(stringify!($field));
                                    println!("{}: {}, expecting: `{}`", CONFIG_NAME, err, stringify!($ty));
                                    $default
                                },
                            };
                        )*

                        Ok(
                            $identifier {
                                $(
                                    $field: $field,
                                )*
                            }
                        )
                    }
                }

                deserializer.deserialize_struct(stringify!($identifier), FIELDS, IdentifierVisitor)
            }
        }

        impl ::std::default::Default for $identifier {
            fn default() -> Self {
                $identifier {
                    $(
                        $field: $default,
                    )*
                }
            }
        }

        impl $crate::Config for $identifier {
            fn load<P: AsRef<::std::path::Path>>(path: P) -> Self {
                match $identifier::load_no_fallback(path.as_ref()) {
                    Ok(v) => v,
                    Err(err) => {
                        println!("1: {}: {}", stringify!($identifier), err);
                        $identifier::default()
                    },
                }
            }

            fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> Result<(), $crate::ConfigError> {
                use ::std::io::Write;

                let result = $crate::serde_yaml::to_string(self);
                let serialized = result.map_err(|e| $crate::ConfigError::Parser(e.to_string()) )?;
                let mut file = ::std::fs::File::create(path)?;
                file.write(&serialized.into_bytes())?;

                Ok(())
            }

            fn load_no_fallback<P: AsRef<::std::path::Path>>(path: P) -> Result<Self, $crate::ConfigError> {
                use ::std::io::Read;
                let path = path.as_ref();

                let content = {
                    let mut file = ::std::fs::File::open(path)?;
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)?;
                    buffer
                };

                let mut result = None;
                if let Some(extension) = path.extension() {
                    if let Some(extension) = extension.to_str() {
                        match extension {
                            "yml" | "yaml" => {
                                let parsed = $crate::serde_yaml::from_str::<$identifier>(&content);
                                let parsed = parsed.map_err(|e| $crate::ConfigError::Parser(e.to_string()) );
                                result = Some(parsed);
                            }
                            "toml" => {
                                let parsed = $crate::toml::from_str::<$identifier>(&content);
                                let parsed = parsed.map_err(|e| $crate::ConfigError::Parser(e.to_string()) );
                                result = Some(parsed);
                            },
                            _ => { },
                        }
                    }
                }

                if let Some(parsed) = result {
                    parsed
                }
                else {
                    Err($crate::ConfigError::Extension(path.to_path_buf()))
                }
            }
        }
    };
);

