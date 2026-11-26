//! The volumes of a RAR set: which file starts a set, where the later volumes are, and what the
//! headers of each volume say (read once, before extraction).
//!
//! Names follow the two schemes RAR writes: `name.part1.rar`, `name.part2.rar`, … (RAR 3 and
//! later) and `name.rar`, `name.r00`, `name.r01`, … (older). Only the first volume starts a set;
//! the others are read through it.

use crate::ArchiveError;
use rars::{Archive, ArchiveReadOptions, ArchiveReader};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

/// A RAR set opened for extraction: its volumes and their parsed headers.
pub struct VolumeSet {
    /// Every volume, in order.
    pub paths: Vec<PathBuf>,
    /// The parsed headers of every volume, in the same order.
    pub archives: Vec<Archive>,
}

impl VolumeSet {
    /// What the set holds, from its headers.
    pub fn info(&self) -> super::RarInfo {
        let members = || self.archives.iter().flat_map(Archive::members);
        let unpacked = members()
            .filter(|m| !m.meta.is_split_before && !m.meta.is_directory)
            .map(|m| m.meta.unpacked_size)
            .sum();
        let largest_volume = self
            .paths
            .iter()
            .filter_map(|p| p.metadata().ok())
            .map(|m| m.len())
            .max()
            .unwrap_or(0);
        super::RarInfo {
            volumes: self.paths.clone(),
            unpacked,
            largest_volume,
            solid: members().any(|m| m.is_solid()),
        }
    }

    /// For every member name, the volumes its entries start in, in archive order.
    pub fn starts(&self) -> HashMap<Vec<u8>, VecDeque<usize>> {
        let mut starts: HashMap<Vec<u8>, VecDeque<usize>> = HashMap::new();
        for (index, archive) in self.archives.iter().enumerate() {
            for member in archive.members() {
                if !member.meta.is_split_before {
                    starts
                        .entry(member.meta.name_bytes().to_vec())
                        .or_default()
                        .push_back(index);
                }
            }
        }
        starts
    }
}

/// Opens the set that starts at `first`: finds the later volumes next to it and parses them all.
pub fn open_set(first: &Path, password: Option<&str>) -> Result<VolumeSet, ArchiveError> {
    let read = |p: &Path| {
        let options = ArchiveReadOptions::with_optional_password(password.map(str::as_bytes));
        ArchiveReader::read_path_with_options(p, options).map_err(ArchiveError::from)
    };
    let head = read(first)?;
    let mut paths = vec![first.to_path_buf()];
    let mut archives = vec![head.clone()];
    if head.is_volume() {
        let mut index = 1;
        while let Some(next) = head.volume_path(first, index).filter(|p| p.is_file()) {
            archives.push(read(&next)?);
            paths.push(next);
            index += 1;
        }
    }
    Ok(VolumeSet { paths, archives })
}

/// True when `name` ends in `.rar` (any case).
pub fn is_rar_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".rar")
}

/// True when `name` can start a RAR set: a `.rar` that is not a later `partN` volume.
///
/// `game.rar`, `game.part1.rar` and `game.part01.rar` start a set; `game.part2.rar` does not, and
/// neither do `game.r00`, `game.r01`, … (the later volumes of the old scheme).
pub fn is_first_volume_name(name: &str) -> bool {
    if !is_rar_name(name) {
        return false;
    }
    let stem = &name[..name.len() - 4];
    match part_number(stem) {
        Some(n) => n == 1,
        None => true,
    }
}

/// True when `name` is one volume of a split RAR set: `….partN.rar` (any N) or the old `….rNN`.
/// A plain `….rar` is not: alone it is a whole archive, and with `.rNN` next to it the caller
/// counts it as the first volume.
pub fn is_split_volume_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if let Some(at) = lower.rfind(".r") {
        let digits = &lower[at + 2..];
        if (2..=3).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit()) {
            return true;
        }
    }
    is_rar_name(name) && part_number(&name[..name.len() - 4]).is_some()
}

/// How many volumes the file names `names` make up (0 when they hold no split RAR set): the
/// distinct split volume names, plus the plain `.rar` files when old-style `.rNN` are there.
pub fn split_volume_count<'a>(names: impl IntoIterator<Item = &'a str>) -> usize {
    let names: std::collections::BTreeSet<String> =
        names.into_iter().map(|n| n.to_ascii_lowercase()).collect();
    let split = names.iter().filter(|n| is_split_volume_name(n)).count();
    let old_style = names
        .iter()
        .any(|n| is_split_volume_name(n) && !is_rar_name(n));
    let plain = names
        .iter()
        .filter(|n| is_rar_name(n) && !is_split_volume_name(n))
        .count();
    match split {
        0 => 0,
        n if old_style => n + plain,
        n => n,
    }
}

/// The number of a `.partN` suffix of `stem`, if it has one.
fn part_number(stem: &str) -> Option<u64> {
    let lower = stem.to_ascii_lowercase();
    let at = lower.rfind(".part")?;
    let digits = &lower[at + 5..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// True when the file at `path` starts a RAR set: its name says so and, when the headers can be
/// read, they do not mark it as a later volume.
pub fn is_first_volume(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !is_first_volume_name(&name) {
        return false;
    }
    match ArchiveReader::read_path(path) {
        Ok(archive) => archive.is_first_volume() != Some(false),
        Err(_) => true,
    }
}
