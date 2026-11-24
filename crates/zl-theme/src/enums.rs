//! The `string_enum!` macro: enums that are spelled as text in a file and read without regard
//! to case.

/// Enum that maps to and from the text used in theme files.
pub trait StringEnum: Sized + Copy + 'static {
    /// Every variant, in declaration order.
    const ALL: &'static [Self];
    /// The canonical text of the variant.
    fn text(self) -> &'static str;
    /// Reads a variant from text, ignoring case and surrounding whitespace.
    fn parse_text(text: &str) -> Option<Self> {
        let wanted = text.trim();
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.text().eq_ignore_ascii_case(wanted))
    }
}

/// Declares a fieldless enum with a canonical text per variant and implements [`StringEnum`].
macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(
                #[doc = concat!("Spelled `", $text, "` in the file.")]
                $(#[$vmeta])*
                $variant
            ),+
        }

        impl $crate::enums::StringEnum for $name {
            const ALL: &'static [Self] = &[$($name::$variant),+];
            fn text(self) -> &'static str {
                match self { $($name::$variant => $text),+ }
            }
        }

        impl $name {
            /// The canonical text of the variant.
            pub fn as_str(self) -> &'static str {
                <Self as $crate::enums::StringEnum>::text(self)
            }
        }
    };
}
