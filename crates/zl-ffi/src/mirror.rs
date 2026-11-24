//! Macros that declare a UniFFI enum or record and its conversions to and from the core type of
//! the same shape, so the adapter cannot drift from the core by a typo.

/// Declares a fieldless UniFFI enum that mirrors the core enum `$core` variant by variant.
macro_rules! mirror_enum {
    ($(#[$m:meta])* $name:ident <=> $core:ty { $($v:ident),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
        pub enum $name {
            $(
                /// A value; see the type for what the values mean.
                $v
            ),+
        }

        impl From<$core> for $name {
            /// One to one.
            fn from(v: $core) -> Self {
                match v { $(<$core>::$v => $name::$v),+ }
            }
        }

        impl From<$name> for $core {
            /// One to one.
            fn from(v: $name) -> Self {
                match v { $($name::$v => <$core>::$v),+ }
            }
        }
    };
}

/// The type of a mirrored field: `plain T` is `T`, `opt T` is `Option<T>`, `list T` is `Vec<T>`.
macro_rules! field_ty {
    (plain $t:ty) => { $t };
    (opt $t:ty) => { Option<$t> };
    (list $t:ty) => { Vec<$t> };
}

/// Converts a field value of a mirrored record, by the field's kind.
macro_rules! field_conv {
    (plain $e:expr) => {
        $e.into()
    };
    (opt $e:expr) => {
        $e.map(Into::into)
    };
    (list $e:expr) => {
        $e.into_iter().map(Into::into).collect()
    };
}

/// Declares a UniFFI record that mirrors the core record `$core` field by field, with both
/// conversions. Each field is `plain`, `opt` (an `Option`) or `list` (a `Vec`) of a type that
/// converts with `Into`.
macro_rules! mirror_record {
    ($(#[$m:meta])* $name:ident <=> $core:ty {
        $($(#[$fm:meta])* $kind:ident $f:ident : $t:ty),+ $(,)?
    }) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, uniffi::Record)]
        pub struct $name {
            $($(#[$fm])* pub $f: field_ty!($kind $t)),+
        }

        impl From<$core> for $name {
            /// One to one.
            fn from(v: $core) -> Self {
                $name { $($f: field_conv!($kind v.$f)),+ }
            }
        }

        impl From<$name> for $core {
            /// One to one.
            fn from(v: $name) -> Self {
                Self { $($f: field_conv!($kind v.$f)),+ }
            }
        }
    };
}
