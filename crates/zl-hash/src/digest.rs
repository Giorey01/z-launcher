//! The three digests computed together over one stream of bytes.

use md5::{Digest as _, Md5};
use sha1::Sha1;

/// CRC32, MD5 and SHA1 of some content, as lower-case hexadecimal text (`crc32` has 8 digits,
/// `md5` 32, `sha1` 40). A field is `None` when the source does not carry it (a CHD header only
/// declares a SHA1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Digests {
    /// CRC32 as 8 hex digits.
    pub crc32: Option<String>,
    /// MD5 as 32 hex digits.
    pub md5: Option<String>,
    /// SHA1 as 40 hex digits.
    pub sha1: Option<String>,
}

/// Lower-case hexadecimal text of some bytes.
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The three running digests; fed chunk by chunk.
pub(crate) struct Hasher {
    /// Running CRC32.
    crc: crc32fast::Hasher,
    /// Running MD5.
    md5: Md5,
    /// Running SHA1.
    sha1: Sha1,
}

impl Hasher {
    /// Starts empty digests.
    pub(crate) fn new() -> Hasher {
        Hasher {
            crc: crc32fast::Hasher::new(),
            md5: Md5::new(),
            sha1: Sha1::new(),
        }
    }

    /// Feeds bytes to the three digests.
    pub(crate) fn update(&mut self, data: &[u8]) {
        self.crc.update(data);
        self.md5.update(data);
        self.sha1.update(data);
    }

    /// Ends the digests.
    pub(crate) fn finish(self) -> Digests {
        Digests {
            crc32: Some(format!("{:08x}", self.crc.finalize())),
            md5: Some(to_hex(&self.md5.finalize())),
            sha1: Some(to_hex(&self.sha1.finalize())),
        }
    }
}
