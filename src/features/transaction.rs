//! **transaction** makes multi-file operations all-or-nothing
//!
//! Installing, uninstalling, enabling and disabling a mod all touch several files.
//! If one step fails halfway (a locked file, a full disk...) the game folder must not be
//! left half-modified while data.json says otherwise. A [`Transaction`] records every
//! change it performs and undoes all of them if it is dropped without being committed.
//! Deletions are delayed until the commit, so they can be undone as well.
//!
//! Main type: [`Transaction`]

use crate::utils::files::{move_file, prune_empty_dirs, unique_path, with_suffix};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A change that has to be reverted if the transaction is rolled back
enum Undo {
    /// A file was created at this path (only installations create files)
    #[cfg_attr(not(feature = "archives"), allow(dead_code))]
    Remove(PathBuf),
    /// A file was moved: move it from `from` back to `to`
    MoveBack { from: PathBuf, to: PathBuf },
}

/// Records file system changes so they can be undone if an operation fails halfway
pub(crate) struct Transaction {
    /// Game folder, inside which empty folders left behind are cleaned up
    game_root: PathBuf,
    undo: Vec<Undo>,
    /// Files set aside by [`Transaction::discard_file`], deleted on commit
    trash: Vec<PathBuf>,
    /// Locations that may leave an empty folder behind once the transaction ends
    vacated: Vec<PathBuf>,
    committed: bool,
}

impl Transaction {
    /// Starts a new transaction on files of the game installed at `game_root`
    pub fn new(game_root: &Path) -> Self {
        Self {
            game_root: game_root.to_path_buf(),
            undo: Vec::new(),
            trash: Vec::new(),
            vacated: Vec::new(),
            committed: false,
        }
    }

    /// Moves `from` to `to` (creating missing folders)
    pub fn move_file(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        move_file(from, to)?;
        self.undo.push(Undo::MoveBack {
            from: to.to_path_buf(),
            to: from.to_path_buf(),
        });
        self.vacated.push(from.to_path_buf());
        Ok(())
    }

    /// Copies `from` to `to` (creating missing folders). `to` must not exist
    #[cfg(feature = "archives")]
    pub fn copy_file(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(from, to)?;
        self.undo.push(Undo::Remove(to.to_path_buf()));
        Ok(())
    }

    /// Puts `from` at `to`, moving it when `consume_source` is set (faster) or copying it otherwise
    #[cfg(feature = "archives")]
    pub fn place_file(&mut self, from: &Path, to: &Path, consume_source: bool) -> io::Result<()> {
        if consume_source {
            self.move_file(from, to)
        } else {
            self.copy_file(from, to)
        }
    }

    /// Deletes `path` when the transaction is committed (a missing file is not an error)
    ///
    /// Until then the file is only renamed, so a rollback can restore it.
    pub fn discard_file(&mut self, path: &Path) -> io::Result<()> {
        let aside = unique_path(&with_suffix(path, ".ata-trash"));

        match fs::rename(path, &aside) {
            Ok(()) => {
                self.undo.push(Undo::MoveBack {
                    from: aside.clone(),
                    to: path.to_path_buf(),
                });
                self.trash.push(aside);
                self.vacated.push(path.to_path_buf());
                Ok(())
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Makes every change permanent: discarded files are deleted and empty folders removed
    pub fn commit(mut self) {
        self.committed = true;

        for file in self.trash.drain(..) {
            let _ = fs::remove_file(file);
        }
        for location in &self.vacated {
            prune_empty_dirs(location, &self.game_root);
        }
    }
}

impl Drop for Transaction {
    /// Rolls back every change, in reverse order, unless the transaction was committed
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        for undo in self.undo.drain(..).rev() {
            let restored_from = match undo {
                Undo::Remove(path) => {
                    let _ = fs::remove_file(&path);
                    path
                }
                Undo::MoveBack { from, to } => {
                    let _ = move_file(&from, &to);
                    from
                }
            };
            prune_empty_dirs(&restored_from, &self.game_root);
        }
    }
}
