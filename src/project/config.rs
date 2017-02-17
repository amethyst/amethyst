
/// Trait implemented by the `config!` macro.
pub trait Config where Self: Sized {
    /// Loads a configuration structure from a file.
    /// Defaults if the file fails in any way.
    fn load<P: AsRef<::std::path::Path>>(P) -> Self;

    /// Loads a configuration structure from a file.
    fn direct_load<P: AsRef<::std::path::Path>>(P) -> Result<Self, ::project::ProjectError>;

    /// Writes a configuration structure to a file.
    fn write<P: AsRef<::std::path::Path>>(&self, P) -> Result<(), ::project::ProjectError>;
}

/// The `config!` macro allows defining configuration structures that can load in
/// `.yml`/`.yaml` files.
///
/// It implements the [`serde::Serialize`], [`serde::Deserialize`], and [`Config`] traits.
/// As well as the standard libraries Default trait for the defaulting fields.
///
/// It follows Rust's syntax for defining structures with one addition, defaulting values.
/// If the file fails to specific fields of a configuration, then it will print out
/// an error describing the issue and then load in this defaulting value.
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
/// extern crate amethyst;
/// extern crate serde;
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde_yaml;
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
        #[derive(Serialize)]
        pub struct $identifier {
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
        #[derive(Serialize)]
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
        impl ::serde::de::Deserialize for $identifier {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::de::Deserializer
            {
                const FIELDS: &'static [&'static str] = &[ $( stringify!($field), )* ];
                const CONFIG_NAME: &'static str = stringify!($identifier);

                #[allow(non_camel_case_types)]
                enum Field {
                    $(
                        $field,
                    )*
                }

                impl ::serde::de::Deserialize for Field {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where D: ::serde::de::Deserializer
                    {
                        struct FieldVisitor;

                        impl ::serde::de::Visitor for FieldVisitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                try!(write!(formatter, "one of "));

                                $(
                                    try!(write!(formatter, "`{}`, ", stringify!($field)));
                                )*

                                Ok(())
                            }

                            fn visit_str<E>(self, value: &str) -> Result<Field, E>
                                where E: ::serde::de::Error,
                            {
                                match value {
                                    $(
                                        stringify!($field) => Ok(Field::$field),
                                    )*
                                    _ => {
                                        Err(::serde::de::Error::unknown_field(value, FIELDS))
                                    },
                                }
                            }
                        }

                        deserializer.deserialize_struct_field(FieldVisitor)
                    }
                }

                struct IdentifierVisitor;
                impl ::serde::de::Visitor for IdentifierVisitor {
                    type Value = $identifier;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(formatter, "a {} configuration", stringify!($identifier))
                    }

                    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                        where V: ::serde::de::SeqVisitor,
                    {
                        let result = $identifier {
                            $(
                                $field: {
                                    match visitor.visit() {
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

                    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                        where V: ::serde::de::MapVisitor,
                    {
                        $(
                            let mut $field: Option<$ty> = None; // allows checking for duplicates
                        )*

                        while let Some(key) = try!(visitor.visit_key::<Field>()) {
                            match key {
                                $(
                                    Field::$field => {
                                        if $field.is_some() {
                                            use ::serde::de::Error;
                                            let err: V::Error = V::Error::duplicate_field(stringify!($field));
                                            println!("{}: {}", CONFIG_NAME, err);
                                        }

                                        $field = match visitor.visit_value() {
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
                                    use ::serde::de::Error;
                                    let err: V::Error = V::Error::missing_field(stringify!($field));
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

        impl $crate::project::Config for $identifier {
            fn load<P: AsRef<::std::path::Path>>(path: P) -> Self {
                match $identifier::direct_load(path.as_ref()) {
                    Ok(v) => v,
                    Err(err) => {
                        println!("{}: {}", stringify!($identifier), err.description());
                        $identifier::default()
                    },
                }
            }

            fn write<P: AsRef<::std::path::Path>>(&self, path: P) -> Result<(), $crate::project::ProjectError> {
                use ::std::io::Write;

                let result = ::serde_yaml::to_string(self);
                let serialized = try!(result.map_err(|e| $crate::project::ProjectError::Parser(e.to_string()) ));
                let mut file = try!(::std::fs::File::create(path));
                try!(file.write(&serialized.into_bytes()));

                Ok(())
            }

            fn direct_load<P: AsRef<::std::path::Path>>(path: P) -> Result<Self, $crate::project::ProjectError> {
                let content = try!($crate::project::directory::Directory::load(path));
                let parsed = ::serde_yaml::from_str::<$identifier>(&content);
                parsed.map_err(|e| $crate::project::ProjectError::Parser(e.to_string()) )
            }
        }
    };
);

