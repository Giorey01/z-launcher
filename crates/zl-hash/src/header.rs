//! Headers that ROM dumps carry but the DATs of No-Intro leave out, so the content hashes the
//! same as the DAT's entry.
//!
//! Facts of the formats (public, widely documented):
//! - iNES (`.nes`): 16 bytes starting with `NES` and `0x1A`.
//! - Famicom Disk System (`.fds`): 16 bytes starting with `FDS` and `0x1A`, when present.
//! - Atari Lynx (`.lnx`): 64 bytes starting with `LYNX`.
//! - Atari 7800 (`.a78`): 128 bytes; the text `ATARI7800` starts at the second byte.
//! - SNES copier dumps (`.smc`, `.swc`, `.fig`, `.sfc`): a 512-byte header, recognised by a size
//!   that is a multiple of 1024 plus 512.

/// Bytes of the file needed to recognise every header (the longest one is the Atari 7800's).
pub(crate) const PROBE_BYTES: usize = 128;

/// Length of the header a file carries, 0 when it has none. `name` is the file name (its
/// extension picks the format), `size` the size of the whole content and `head` its first bytes
/// (at least [`PROBE_BYTES`] when the file is that long).
pub fn header_len(name: &str, size: u64, head: &[u8]) -> u32 {
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let (len, ok) = match ext.as_str() {
        "nes" => (16, head.starts_with(b"NES\x1A")),
        "fds" => (16, head.starts_with(b"FDS\x1A")),
        "lnx" => (64, head.starts_with(b"LYNX")),
        "a78" => (128, head.get(1..10) == Some(b"ATARI7800".as_slice())),
        "smc" | "swc" | "fig" | "sfc" => (512, size % 1024 == 512),
        _ => (0, false),
    };
    if ok && size > u64::from(len) {
        len
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each format is recognised by its own marker and extension.
    #[test]
    fn recognises_the_headers() {
        assert_eq!(header_len("a.NES", 100, b"NES\x1Arest"), 16);
        assert_eq!(header_len("a.nes", 100, b"XXXX"), 0);
        assert_eq!(header_len("a.fds", 100, b"FDS\x1A"), 16);
        assert_eq!(header_len("a.lnx", 100, b"LYNXabc"), 64);
        let mut a78 = vec![1u8];
        a78.extend_from_slice(b"ATARI7800");
        assert_eq!(header_len("a.a78", 1000, &a78), 128);
        assert_eq!(header_len("a.a78", 1000, b"\x01ATARI7801"), 0);
    }

    /// SNES copier headers are told by the size alone, and only on the SNES extensions.
    #[test]
    fn snes_by_size() {
        assert_eq!(header_len("g.smc", 1024 * 1024 + 512, b""), 512);
        assert_eq!(header_len("g.sfc", 1024 * 1024, b""), 0);
        assert_eq!(header_len("g.bin", 1024 * 1024 + 512, b""), 0);
        assert_eq!(header_len("g.smc", 512, b""), 0);
    }

    /// A file shorter than its header has nothing to hash without it.
    #[test]
    fn short_file_has_no_header() {
        assert_eq!(header_len("a.nes", 16, b"NES\x1A"), 0);
    }
}
