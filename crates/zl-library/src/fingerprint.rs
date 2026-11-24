//! A stable 64-bit fingerprint of a game's shape, to detect changes with one comparison.

use zl_db::{GameFile, GameKind};

/// FNV-1a offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Feeds bytes into the running hash.
fn mix(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
    // Separator so ("ab","c") and ("a","bc") differ.
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(FNV_PRIME);
}

/// Hashes kind, title and every member file (path, size, mtime). `files` must be in a stable
/// order. FNV is written out here because `DefaultHasher` is not stable across Rust versions
/// and the value is stored on disk.
pub(crate) fn fingerprint(kind: GameKind, title: &str, files: &[GameFile]) -> i64 {
    let mut hash = FNV_OFFSET;
    mix(&mut hash, kind.as_str().as_bytes());
    mix(&mut hash, title.as_bytes());
    for file in files {
        mix(&mut hash, file.path.as_bytes());
        mix(&mut hash, &file.size.to_le_bytes());
        mix(&mut hash, &file.mtime_ms.to_le_bytes());
    }
    hash as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds one member.
    fn file(path: &str, size: i64, mtime: i64) -> GameFile {
        GameFile {
            path: path.into(),
            size,
            mtime_ms: mtime,
        }
    }

    /// Any change of title, size, mtime or members changes the hash; equal input does not.
    #[test]
    fn changes_are_detected() {
        let base = [file("/a", 1, 2)];
        let h = fingerprint(GameKind::Rom, "t", &base);
        assert_eq!(h, fingerprint(GameKind::Rom, "t", &[file("/a", 1, 2)]));
        assert_ne!(h, fingerprint(GameKind::Rom, "u", &base));
        assert_ne!(h, fingerprint(GameKind::M3u, "t", &base));
        assert_ne!(h, fingerprint(GameKind::Rom, "t", &[file("/a", 2, 2)]));
        assert_ne!(h, fingerprint(GameKind::Rom, "t", &[file("/a", 1, 3)]));
        assert_ne!(
            h,
            fingerprint(GameKind::Rom, "t", &[file("/a", 1, 2), file("/b", 1, 2)])
        );
    }
}
