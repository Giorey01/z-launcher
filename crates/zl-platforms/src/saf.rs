//! Storage Access Framework URIs built without Android: the `content://` document URI an
//! emulator needs for a ROM under a folder the user granted with `ACTION_OPEN_DOCUMENT_TREE`.
//!
//! The encoding is the one of Android's `Uri.encode`: UTF-8 bytes as uppercase `%XX`, except
//! letters, digits and `_ - ! . ~ ' ( ) *`. So `(` and `)` stay, while `,`, space, `[`, `:` and
//! `/` inside an id become `%2C`, `%20`, `%5B`, `%3A`, `%2F`. These are the URIs seen in the
//! launch log of a real device (`docs/reference/launch-observed.md`).

use crate::launch::RomUris;

/// Characters other than letters and digits that `Uri.encode` leaves as they are.
const UNRESERVED: &str = "_-!.~'()*";

/// Prefix of a tree URI's path, before the tree document id.
const TREE_SEGMENT: &str = "/tree/";

/// Prefix that every content URI starts with.
const CONTENT_SCHEME: &str = "content://";

/// Where the primary (internal) volume is mounted for apps with all-files access.
const PRIMARY_VOLUME_PATH: &str = "/storage/emulated/0";

/// Volume id the external storage provider uses for the internal storage.
const PRIMARY_VOLUME_ID: &str = "primary";

/// Authority of the provider that serves the device's own storage and its removable volumes.
const EXTERNAL_STORAGE_AUTHORITY: &str = "com.android.externalstorage.documents";

/// Prefix of the per-user mounts of the internal storage (`/storage/emulated/<user id>`).
const EMULATED_PREFIX: &str = "/storage/emulated/";

/// Prefix of the mount points of removable volumes.
const STORAGE_PREFIX: &str = "/storage/";

/// Percent-encodes `text` like Android's `Uri.encode(text, allow)`; characters in `allow` are
/// also left as they are.
pub fn uri_encode(text: &str, allow: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        let ch = char::from(byte);
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(ch) || allow.contains(ch) {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Decodes `%XX` sequences; `None` when one is malformed or the result is not UTF-8.
pub fn uri_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3)?;
            // `from_str_radix` would also take `+f`: only two real hex digits are an escape.
            if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The authority and the decoded tree document id of a tree URI such as
/// `content://com.android.externalstorage.documents/tree/primary%3AGames` (a trailing
/// `/document/...` part is ignored).
fn split_tree_uri(tree_uri: &str) -> Option<(&str, String)> {
    let rest = tree_uri.strip_prefix(CONTENT_SCHEME)?;
    let (authority, after) = rest.split_once(TREE_SEGMENT)?;
    let encoded = after.split('/').next().filter(|id| !id.is_empty())?;
    Some((authority, uri_decode(encoded)?))
}

/// The folder a tree document id stands for on the file system, for the external storage
/// provider: `primary:Games` is `/storage/emulated/0/Games`, `ABCD-1234:Roms` is
/// `/storage/ABCD-1234/Roms`. `None` for an id without a volume.
fn tree_base_path(doc_id: &str) -> Option<String> {
    let (volume, path) = doc_id.split_once(':')?;
    if volume.is_empty() {
        return None;
    }
    let base = if volume == PRIMARY_VOLUME_ID {
        PRIMARY_VOLUME_PATH.to_string()
    } else {
        format!("/storage/{volume}")
    };
    Some(if path.is_empty() {
        base
    } else {
        format!("{base}/{path}")
    })
}

/// The file system folder a granted tree URI stands for (`primary:Games` is
/// `/storage/emulated/0/Games`); `None` when the URI is malformed or on an unknown provider form.
pub fn tree_path(tree_uri: &str) -> Option<String> {
    let (_, doc_id) = split_tree_uri(tree_uri)?;
    tree_base_path(&doc_id)
}

/// Mount of the internal storage of another Android user (`/storage/emulated/10/...`), which
/// the external storage provider also calls `primary`; any other path is returned as it is.
fn primary_alias(path: &str) -> std::borrow::Cow<'_, str> {
    let Some(rest) = path.strip_prefix(EMULATED_PREFIX) else {
        return path.into();
    };
    let (user, tail) = rest.split_once('/').unwrap_or((rest, ""));
    if user.is_empty() || !user.bytes().all(|b| b.is_ascii_digit()) {
        return path.into();
    }
    if tail.is_empty() {
        PRIMARY_VOLUME_PATH.into()
    } else {
        format!("{PRIMARY_VOLUME_PATH}/{tail}").into()
    }
}

/// The tree URI the folder picker should start at for a folder (the inverse of [`tree_path`]);
/// `None` for a path that is not on a storage volume.
pub fn tree_uri_for_path(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/');
    let path = primary_alias(path);
    let path = path.as_ref();
    let doc_id = match path.strip_prefix(PRIMARY_VOLUME_PATH) {
        Some("") => format!("{PRIMARY_VOLUME_ID}:"),
        Some(rest) => format!("{PRIMARY_VOLUME_ID}:{}", rest.strip_prefix('/')?),
        None => {
            let rest = path.strip_prefix(STORAGE_PREFIX)?;
            let (volume, folder) = rest.split_once('/').unwrap_or((rest, ""));
            (!volume.is_empty()).then(|| format!("{volume}:{folder}"))?
        }
    };
    Some(format!(
        "{CONTENT_SCHEME}{EXTERNAL_STORAGE_AUTHORITY}{TREE_SEGMENT}{}",
        uri_encode(&doc_id, "")
    ))
}

/// Path of `file` below `base`, without a leading slash and with doubled slashes collapsed;
/// `None` when it is not below it or climbs out with a `.` or `..` segment (a document id cannot
/// carry those).
fn relative_to(file: &str, base: &str) -> Option<String> {
    let rest = file.strip_prefix(base.trim_end_matches('/'))?;
    let rel = rest.strip_prefix('/')?;
    let segments: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() || segments.iter().any(|s| matches!(*s, "." | "..")) {
        return None;
    }
    Some(segments.join("/"))
}

/// The SAF document URI of `file_path`, a file under the folder the user granted as `tree_uri`.
///
/// The folder the tree stands for is derived from the tree id (`primary:Games` is
/// `/storage/emulated/0/Games`); when the id has no known volume form, `root_path` is taken as
/// that folder. `None` when the tree URI is malformed or the file is not under the tree.
pub fn saf_document_uri(tree_uri: &str, root_path: &str, file_path: &str) -> Option<String> {
    let (authority, tree_id) = split_tree_uri(tree_uri)?;
    let base = tree_base_path(&tree_id).unwrap_or_else(|| root_path.to_string());
    let rel = relative_to(file_path, &base)?;
    let doc_id = if tree_id.ends_with(':') {
        format!("{tree_id}{rel}")
    } else {
        format!("{tree_id}/{rel}")
    };
    Some(format!(
        "{CONTENT_SCHEME}{authority}{TREE_SEGMENT}{}/document/{}",
        uri_encode(&tree_id, ""),
        uri_encode(&doc_id, "")
    ))
}

/// The `file://` URI of a path, encoded like `Uri.fromFile` (slashes are kept).
pub fn file_uri(path: &str) -> String {
    format!("file://{}", uri_encode(path, "/"))
}

/// The ROM URIs the launch variables need: always `file://`, and the SAF document and tree when
/// the root has a granted tree that contains the file.
pub fn rom_uris(file_path: &str, root_path: &str, tree_uri: Option<&str>) -> RomUris {
    let saf = tree_uri.and_then(|tree| saf_document_uri(tree, root_path, file_path));
    RomUris {
        file: Some(file_uri(file_path)),
        saf_tree: saf.as_ref().and(tree_uri.map(str::to_string)),
        saf,
        provider: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tree granted on `/storage/emulated/0/Games`, as the picker returns it.
    const TREE: &str = "content://com.android.externalstorage.documents/tree/primary%3AGames";

    /// The URI observed in a real launch log: parentheses stay, comma and space are encoded.
    #[test]
    fn matches_the_observed_document_uri() {
        let uri = saf_document_uri(
            TREE,
            "/storage/emulated/0/Games",
            "/storage/emulated/0/Games/Ps2/God of War (Europe, Australia) (En,Fr,De,Es,It).iso",
        );
        assert_eq!(
            uri.as_deref(),
            Some(
                "content://com.android.externalstorage.documents/tree/primary%3AGames/document/\
                 primary%3AGames%2FPs2%2FGod%20of%20War%20(Europe%2C%20Australia)%20\
                 (En%2CFr%2CDe%2CEs%2CIt).iso"
            )
        );
    }

    /// Brackets, ampersands, apostrophes, `!` and non-ASCII letters follow `Uri.encode`.
    #[test]
    fn encodes_like_android() {
        assert_eq!(
            uri_encode("a b[c]&d'e!é~*_-.", ""),
            "a%20b%5Bc%5D%26d'e!%C3%A9~*_-."
        );
        assert_eq!(uri_encode("x/y:z", "/"), "x/y%3Az");
        assert_eq!(
            uri_decode("primary%3AGames%2FPs2"),
            Some("primary:Games/Ps2".into())
        );
        assert_eq!(uri_decode("bad%zz"), None);
        assert_eq!(uri_decode("cut%4"), None);
    }

    /// The Wii U launch file sits inside a game folder, so its document id has more segments.
    #[test]
    fn nested_launch_file_for_wiiu() {
        let uri = saf_document_uri(
            TREE,
            "/storage/emulated/0/Games",
            "/storage/emulated/0/Games/wiiu/Mario Kart 8 [00050000]/code/MK8.rpx",
        );
        assert_eq!(
            uri.as_deref(),
            Some(
                "content://com.android.externalstorage.documents/tree/primary%3AGames/document/\
                 primary%3AGames%2Fwiiu%2FMario%20Kart%208%20%5B00050000%5D%2Fcode%2FMK8.rpx"
            )
        );
    }

    /// A tree on the whole volume has an id ending in a colon: no extra slash, and the root's
    /// own folder name becomes part of the path.
    #[test]
    fn whole_volume_tree() {
        let tree = "content://com.android.externalstorage.documents/tree/primary%3A";
        let uri = saf_document_uri(
            tree,
            "/storage/emulated/0/Games",
            "/storage/emulated/0/Games/Wii/a.iso",
        );
        assert_eq!(
            uri.as_deref(),
            Some(
                "content://com.android.externalstorage.documents/tree/primary%3A/document/\
                 primary%3AGames%2FWii%2Fa.iso"
            )
        );
    }

    /// A removable volume maps `ABCD-1234:Roms` to `/storage/ABCD-1234/Roms`.
    #[test]
    fn removable_volume_tree() {
        let tree = "content://com.android.externalstorage.documents/tree/ABCD-1234%3ARoms";
        let uri = saf_document_uri(
            tree,
            "/storage/ABCD-1234/Roms",
            "/storage/ABCD-1234/Roms/nds/x.nds",
        );
        assert_eq!(
            uri.as_deref(),
            Some(
                "content://com.android.externalstorage.documents/tree/ABCD-1234%3ARoms/document/\
                 ABCD-1234%3ARoms%2Fnds%2Fx.nds"
            )
        );
    }

    /// Files outside the tree, malformed trees and prefix look-alikes give no URI.
    #[test]
    fn rejects_files_outside_the_tree() {
        let root = "/storage/emulated/0/Games";
        assert_eq!(
            saf_document_uri(TREE, root, "/storage/emulated/0/Other/a.iso"),
            None
        );
        assert_eq!(
            saf_document_uri(TREE, root, "/storage/emulated/0/GamesX/a.iso"),
            None
        );
        assert_eq!(
            saf_document_uri(TREE, root, "/storage/emulated/0/Games"),
            None
        );
        assert_eq!(saf_document_uri("file:///x", root, "/x/a"), None);
        assert_eq!(saf_document_uri("content://a/tree/", root, "/x/a"), None);
    }

    /// A tree URI that carries a `/document/...` suffix is read by its tree part.
    #[test]
    fn tree_uri_with_document_suffix() {
        let tree = format!("{TREE}/document/primary%3AGames");
        let got = saf_document_uri(
            &tree,
            "/storage/emulated/0/Games",
            "/storage/emulated/0/Games/a/b.iso",
        );
        assert!(got.is_some_and(|u| u.ends_with("/document/primary%3AGames%2Fa%2Fb.iso")));
    }

    /// Tree URIs and folders convert both ways, for internal and removable volumes.
    #[test]
    fn tree_and_path_are_inverse() {
        for (path, tree) in [
            ("/storage/emulated/0/Games", "primary%3AGames"),
            ("/storage/emulated/0", "primary%3A"),
            ("/storage/ABCD-1234/Roms/Sub", "ABCD-1234%3ARoms%2FSub"),
            ("/storage/ABCD-1234", "ABCD-1234%3A"),
        ] {
            let uri = format!("content://com.android.externalstorage.documents/tree/{tree}");
            assert_eq!(
                tree_uri_for_path(path).as_deref(),
                Some(uri.as_str()),
                "{path}"
            );
            assert_eq!(tree_path(&uri).as_deref(), Some(path), "{uri}");
        }
        assert_eq!(tree_uri_for_path("/data/local"), None);
        assert_eq!(tree_uri_for_path("/storage/"), None);
        assert_eq!(tree_path("content://a/tree/nocolon"), None);
    }

    /// `rom_uris` fills the file URI always and the SAF pair only with a tree.
    #[test]
    fn rom_uris_variants() {
        let path = "/storage/emulated/0/Games/psp/A B.iso";
        let with = rom_uris(path, "/storage/emulated/0/Games", Some(TREE));
        assert_eq!(
            with.file.as_deref(),
            Some("file:///storage/emulated/0/Games/psp/A%20B.iso")
        );
        assert_eq!(with.saf_tree.as_deref(), Some(TREE));
        assert!(with.saf.is_some());
        let without = rom_uris(path, "/storage/emulated/0/Games", None);
        assert!(without.saf.is_none() && without.saf_tree.is_none());
        let outside = rom_uris("/elsewhere/a.iso", "/storage/emulated/0/Games", Some(TREE));
        assert!(outside.saf.is_none() && outside.saf_tree.is_none());
    }
}
