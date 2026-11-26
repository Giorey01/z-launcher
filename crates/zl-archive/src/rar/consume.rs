//! Consuming a volume set: each volume is emptied and deleted as soon as extraction is past it.
//!
//! `rars` decodes the members of a set in archive order, front to back. When the writer of a
//! member opens, every member before it is finished, so every volume before the one this member
//! starts in will not be read again. That volume is known from the headers read up front (the
//! member's name and the volume where its first part is), not from which volume `rars` happens to
//! have opened: it may open later volumes early (a RAR 5 member whose size is only known at its
//! end). A member whose name cannot be matched frees nothing, so a mismatch only costs space,
//! never data. This holds for solid sets too: a solid decoder never reads back.
//!
//! A volume is emptied (`set_len(0)`) before it is deleted, so its space is freed at once even
//! where a handle on it is still open.

use super::output::Shared;
use super::volumes::VolumeSet;
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Which volumes can go, and how many already went.
pub struct Plan {
    /// Every volume, in order.
    paths: Vec<PathBuf>,
    /// For every member name, the volumes its entries start in, in archive order.
    starts: HashMap<Vec<u8>, VecDeque<usize>>,
    /// Volumes before this index are gone.
    freed: usize,
    /// True when the caller asked to consume the set.
    enabled: bool,
    /// The extraction's shared state (told when the first volume goes).
    shared: Arc<Shared>,
}

impl Plan {
    /// The plan for `set`; it frees nothing unless `enabled` and the set has several volumes.
    pub fn new(set: &VolumeSet, enabled: bool, shared: Arc<Shared>) -> Self {
        let enabled = enabled && set.paths.len() > 1;
        Self {
            paths: set.paths.clone(),
            starts: if enabled {
                set.starts()
            } else {
                HashMap::new()
            },
            freed: 0,
            enabled,
            shared,
        }
    }

    /// True when at least one volume is gone.
    pub fn consumed(&self) -> bool {
        self.freed > 0
    }

    /// Called when the member `name` is about to be written: frees the volumes before its first.
    pub fn on_member(&mut self, name: &[u8]) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let start = self
            .starts
            .get_mut(name)
            .and_then(VecDeque::pop_front)
            .unwrap_or(self.freed);
        while self.freed < start {
            self.shared.commit();
            free(&self.paths[self.freed])?;
            self.freed += 1;
        }
        Ok(())
    }
}

/// Empties the volume at `path` (its space comes back at once) and deletes it.
fn free(path: &Path) -> io::Result<()> {
    OpenOptions::new().write(true).open(path)?.set_len(0)?;
    // Where a handle is still open (Windows) the delete can fail; the file is already empty and
    // the caller deletes the volumes again at the end.
    let _ = fs::remove_file(path);
    Ok(())
}
