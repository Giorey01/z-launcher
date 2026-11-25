//! Semantic versioning for Pods (strict `MAJOR.MINOR.PATCH`).
//!
//! Defined in `docs/spec/pods.md` (§3, §6).

use std::fmt;

use crate::error::PodIssue;

/// A strict 3-part semantic version (`MAJOR.MINOR.PATCH`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PodVersion {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Patch version number.
    pub patch: u32,
}

impl PodVersion {
    /// Parses a string into a strict 3-part version.
    ///
    /// Returns [`PodIssue::BadVersion`] if the string does not consist of exactly
    /// three non-negative integers separated by `.`, without any prerelease tags or leading/trailing chars.
    pub fn parse(s: &str) -> Result<PodVersion, PodIssue> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(PodIssue::BadVersion);
        }

        let parse_part = |p: &str| -> Result<u32, PodIssue> {
            if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
                return Err(PodIssue::BadVersion);
            }
            p.parse::<u32>().map_err(|_| PodIssue::BadVersion)
        };

        let major = parse_part(parts[0])?;
        let minor = parse_part(parts[1])?;
        let patch = parse_part(parts[2])?;

        Ok(PodVersion {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for PodVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_versions() {
        assert_eq!(
            PodVersion::parse("1.2.0"),
            Ok(PodVersion {
                major: 1,
                minor: 2,
                patch: 0
            })
        );
        assert_eq!(
            PodVersion::parse("0.1.0"),
            Ok(PodVersion {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert_eq!(
            PodVersion::parse("10.20.30"),
            Ok(PodVersion {
                major: 10,
                minor: 20,
                patch: 30
            })
        );
    }

    #[test]
    fn parse_invalid_versions() {
        assert_eq!(PodVersion::parse("1.0"), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse("1.0.0.0"), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse("1.0.0-alpha"), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse("1.0.0b"), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse("v1.0.0"), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse(""), Err(PodIssue::BadVersion));
        assert_eq!(PodVersion::parse(".."), Err(PodIssue::BadVersion));
    }

    #[test]
    fn version_ordering() -> Result<(), PodIssue> {
        let v1_9_0 = PodVersion::parse("1.9.0")?;
        let v1_10_0 = PodVersion::parse("1.10.0")?;
        let v2_0_0 = PodVersion::parse("2.0.0")?;

        assert!(v1_10_0 > v1_9_0);
        assert!(v2_0_0 > v1_10_0);
        Ok(())
    }
}
