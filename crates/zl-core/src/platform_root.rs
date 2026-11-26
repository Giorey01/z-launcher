//! The folder of a platform in the library: where a game of that platform belongs when an
//! extension (a Pod) puts one there. It is the folder of a ROM root that already holds the
//! platform's games, the one with the most of them; with no game yet, a folder of a root named
//! after the platform (its id or an alias); otherwise there is none.

use std::collections::BTreeMap;
use std::iter::once;
use std::path::Path;

use crate::{CoreError, RootInfo, Service};

/// The folder of a platform and the SAF tree of the root it is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformRoot {
    /// Absolute path of the folder.
    pub path: String,
    /// SAF tree URI of the root the folder is in, if the user granted one.
    pub saf_tree: Option<String>,
}

impl Service {
    /// The folder of `platform` (id or alias), `None` when the library has none.
    pub fn platform_root(&self, platform: &str) -> Result<Option<PlatformRoot>, CoreError> {
        let Some(p) = self.platforms.platform(platform) else {
            return Ok(None);
        };
        let names: Vec<String> = once(&p.id)
            .chain(&p.aliases)
            .map(|n| n.to_lowercase())
            .collect();
        let roots: Vec<RootInfo> = self.roots()?.into_iter().filter(|r| r.enabled).collect();
        if let Some(found) = self.folder_with_most_games(&names, &roots)? {
            return Ok(Some(found));
        }
        Ok(roots.iter().find_map(|r| named_folder(r, &names)))
    }

    /// Among the folders right under a root that hold games of the platform (any of `names`),
    /// the one with the most; on a tie the first by name.
    fn folder_with_most_games(
        &self,
        names: &[String],
        roots: &[RootInfo],
    ) -> Result<Option<PlatformRoot>, CoreError> {
        let mut counts: BTreeMap<(i64, String), usize> = BTreeMap::new();
        let db = self.db();
        for name in names {
            for game in db.list_games_by_platform(name)? {
                let Some(root) = roots.iter().find(|r| r.id == game.root_id) else {
                    continue;
                };
                if let Some(first) = first_segment(&root.path, &game.path) {
                    *counts.entry((root.id, first)).or_default() += 1;
                }
            }
        }
        let best = counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)));
        Ok(best.and_then(|((root_id, folder), _)| {
            let root = roots.iter().find(|r| r.id == root_id)?;
            Some(PlatformRoot {
                path: format!("{}/{folder}", root.path),
                saf_tree: root.saf_tree.clone(),
            })
        }))
    }
}

/// The first folder of `path` under `root` (`Ps2` for `<root>/Ps2/God of War.iso`); `None` for a
/// file right in the root or a path outside it. A relative `path` is taken as under the root.
fn first_segment(root: &str, path: &str) -> Option<String> {
    let rel = path
        .strip_prefix(root)
        .map_or(path, |r| r.trim_start_matches('/'));
    let mut parts = rel.split('/').filter(|s| !s.is_empty());
    let first = parts.next()?;
    parts.next().map(|_| first.to_string())
}

/// A folder right under `root` whose name is one of `names` (any case).
fn named_folder(root: &RootInfo, names: &[String]) -> Option<PlatformRoot> {
    let entries = std::fs::read_dir(Path::new(&root.path)).ok()?;
    let mut found: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| names.contains(&n.to_lowercase()))
        .collect();
    found.sort();
    found.into_iter().next().map(|n| PlatformRoot {
        path: format!("{}/{n}", root.path),
        saf_tree: root.saf_tree.clone(),
    })
}

#[cfg(test)]
mod tests {
    //! The first folder of a game's path.

    use super::first_segment;

    /// A game in a platform folder, deeper in it, right in the root, and outside the root.
    #[test]
    fn first_folder_of_a_game() {
        assert_eq!(first_segment("/g", "/g/Ps2/God.iso"), Some("Ps2".into()));
        assert_eq!(
            first_segment("/g", "/g/Switch/Games/A.nsp"),
            Some("Switch".into())
        );
        assert_eq!(first_segment("/g", "/g/loose.iso"), None);
        assert_eq!(first_segment("/g", "Psp/Patapon.iso"), Some("Psp".into()));
    }
}
