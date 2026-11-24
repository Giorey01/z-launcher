//! Colour parsing (spec 3.3): `#RGB`, `#RRGGBB`, `#RRGGBBAA` (CSS order), `rgb()` and `rgba()`.

use std::str::FromStr;

/// An 8-bit-per-channel colour with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red, 0-255.
    pub r: u8,
    /// Green, 0-255.
    pub g: u8,
    /// Blue, 0-255.
    pub b: u8,
    /// Alpha, 0 (transparent) to 255 (opaque).
    pub a: u8,
}

/// The text is not a colour in any accepted syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a colour")]
pub struct ColourError;

impl Rgba {
    /// Opaque colour from three channels.
    pub fn opaque(r: u8, g: u8, b: u8) -> Self {
        Rgba { r, g, b, a: 255 }
    }

    /// The colour as `[r, g, b, a]`.
    pub fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl FromStr for Rgba {
    type Err = ColourError;

    /// Parses any accepted syntax; case and whitespace are tolerated.
    fn from_str(text: &str) -> Result<Self, ColourError> {
        let t = text.trim().to_ascii_lowercase();
        if let Some(hex) = t.strip_prefix('#') {
            parse_hex(hex)
        } else if let Some(args) = function_args(&t, "rgba").or_else(|| function_args(&t, "rgb")) {
            parse_function(args)
        } else {
            Err(ColourError)
        }
    }
}

/// Returns the text between the parentheses of `name( ... )`.
fn function_args<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(name)?.trim_start();
    rest.strip_prefix('(')?.strip_suffix(')')
}

/// Parses the digits after `#`: 3, 6 or 8 hex digits.
fn parse_hex(hex: &str) -> Result<Rgba, ColourError> {
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ColourError);
    }
    let digit = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).map_err(|_| ColourError);
    let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ColourError);
    match hex.len() {
        3 => Ok(Rgba::opaque(digit(0)? * 17, digit(1)? * 17, digit(2)? * 17)),
        6 => Ok(Rgba::opaque(pair(0)?, pair(2)?, pair(4)?)),
        8 => Ok(Rgba {
            r: pair(0)?,
            g: pair(2)?,
            b: pair(4)?,
            a: pair(6)?,
        }),
        _ => Err(ColourError),
    }
}

/// Parses the arguments of `rgb(...)` / `rgba(...)`: three channels 0-255 and an optional alpha
/// 0-1 (rounded to the nearest 1/255).
fn parse_function(args: &str) -> Result<Rgba, ColourError> {
    let parts: Vec<&str> = args.split(',').map(str::trim).collect();
    if parts.len() != 3 && parts.len() != 4 {
        return Err(ColourError);
    }
    let channel = |s: &str| s.parse::<u8>().map_err(|_| ColourError);
    let a = match parts.get(3) {
        Some(text) => {
            let alpha: f64 = text.parse().map_err(|_| ColourError)?;
            if !(0.0..=1.0).contains(&alpha) {
                return Err(ColourError);
            }
            (alpha * 255.0).round() as u8
        }
        None => 255,
    };
    Ok(Rgba {
        r: channel(parts[0])?,
        g: channel(parts[1])?,
        b: channel(parts[2])?,
        a,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses or panics with the input, for terse assertions.
    fn p(text: &str) -> Rgba {
        text.parse()
            .unwrap_or_else(|_| panic!("should parse: {text}"))
    }

    /// Every accepted syntax gives the expected channels.
    #[test]
    fn accepted_syntaxes() {
        assert_eq!(p("#34beed"), Rgba::opaque(52, 190, 237));
        assert_eq!(
            p("#8a8a8a66"),
            Rgba {
                r: 138,
                g: 138,
                b: 138,
                a: 102
            }
        );
        assert_eq!(p("#fff"), Rgba::opaque(255, 255, 255));
        assert_eq!(p("#F80"), Rgba::opaque(255, 136, 0));
        assert_eq!(
            p("rgba(0,0,0,0.5)"),
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 128
            }
        );
        assert_eq!(
            p("  RGBA( 10 , 20,30 , 1 ) "),
            Rgba {
                r: 10,
                g: 20,
                b: 30,
                a: 255
            }
        );
        assert_eq!(p("rgb(1, 2, 3)"), Rgba::opaque(1, 2, 3));
    }

    /// Eight digits are CSS order: alpha is the last pair.
    #[test]
    fn eight_digits_alpha_last() {
        assert_eq!(p("#ffffff66").a, 0x66);
        assert_eq!(p("#ffffff66").r, 0xff);
    }

    /// Garbage and out-of-range channels are rejected.
    #[test]
    fn rejected() {
        for bad in [
            "",
            "banana",
            "#12",
            "#12345",
            "#gg0000",
            "rgb(256,0,0)",
            "rgba(0,0,0,1.5)",
            "rgba(0,0)",
            "#ffffff6",
        ] {
            assert!(bad.parse::<Rgba>().is_err(), "should reject {bad:?}");
        }
    }
}
