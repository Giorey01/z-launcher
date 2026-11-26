//! Add-ons named by title id: a Switch dump names its file with the 16-hex-digit title id in
//! brackets (`Game [0100633007D48000][v0].nsp`). The base game's id ends in 13 zero bits; its
//! update sets `0x800` and its DLC set `0x1000` and count up. Files whose id leads to a base game
//! of the same platform are members of that game, not games of their own; two dumps of the same
//! id (`.nsp` and `.nsz`) are one game. An update or DLC whose base is not there stays a game,
//! so nothing on disk disappears from the library.

use std::collections::HashMap;

use crate::group::Unit;
use crate::walk::FileEntry;

/// The bits of a title id that tell the base game from its update and DLC.
const ADDON_BITS: u64 = 0x1FFF;

/// The title id in a file name: the first bracketed run of exactly 16 hex digits.
pub(crate) fn title_id(name: &str) -> Option<u64> {
    name.split('[').skip(1).find_map(|part| {
        let id = part.split(']').next()?;
        (id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| u64::from_str_radix(id, 16).ok())
            .flatten()
    })
}

/// The base game's id of `id`.
fn base_of(id: u64) -> u64 {
    id & !ADDON_BITS
}

/// Folds the units whose file names carry the id of an add-on (or a second dump) into the unit
/// of their base game; the others are returned as they were.
pub(crate) fn merge_addons(files: &[FileEntry], units: Vec<Unit>) -> Vec<Unit> {
    let id_of = |u: &Unit| {
        let name = files[u.primary]
            .path
            .file_name()?
            .to_string_lossy()
            .into_owned();
        title_id(&name)
    };
    // The base game of each id: the first unit (by primary path) whose own id is a base id.
    let mut base_unit: HashMap<u64, usize> = HashMap::new();
    for (i, u) in units.iter().enumerate() {
        if let Some(id) = id_of(u).filter(|id| base_of(*id) == *id) {
            base_unit.entry(id).or_insert(i);
        }
    }
    let mut absorbed: Vec<Vec<usize>> = vec![Vec::new(); units.len()];
    let mut keep = vec![true; units.len()];
    for (i, u) in units.iter().enumerate() {
        let Some(base) = id_of(u).map(base_of) else {
            continue;
        };
        match base_unit.get(&base) {
            Some(&owner) if owner != i => {
                absorbed[owner].extend(u.members.iter().copied());
                keep[i] = false;
            }
            _ => {}
        }
    }
    units
        .into_iter()
        .zip(absorbed)
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|((mut u, extra), _)| {
            u.members.extend(extra);
            u.members.sort_unstable();
            u.members.dedup();
            u
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_id_is_the_bracketed_sixteen_hex_digits() {
        assert_eq!(
            title_id("Hollow Knight [0100633007D48800][v589824] (1.20 GB).nsp"),
            Some(0x0100_6330_07D4_8800)
        );
        assert_eq!(
            title_id("Boomerang Fu [DLC][010081A00EE63002].nsp"),
            Some(0x0100_81A0_0EE6_3002)
        );
        assert_eq!(
            title_id("Mario Tennis Aces[0100BDE00862A000][US][v0].nsp"),
            Some(0x0100_BDE0_0862_A000)
        );
        assert_eq!(title_id("Super Mario World (USA).sfc"), None);
        assert_eq!(title_id("Game [v0][12345].nsp"), None);
    }

    #[test]
    fn updates_and_dlc_lead_to_their_base_game() {
        let base = 0x0100_81A0_0EE6_2000;
        assert_eq!(base_of(0x0100_81A0_0EE6_2800), base);
        assert_eq!(base_of(0x0100_81A0_0EE6_3001), base);
        assert_eq!(base_of(0x0100_81A0_0EE6_3002), base);
        assert_eq!(base_of(base), base);
    }
}
