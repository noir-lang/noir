use std::{fs::File, path::PathBuf};

use fs2::FileExt as _;

/// A lock used to lock the Nargo.toml file so two concurrent runs of nargo
/// commands (for example two `nargo execute`) don't overwrite output artifacts.
pub(crate) struct Lock {
    file: File,
}

impl Lock {
    #[allow(clippy::self_named_constructors)]
    pub(crate) fn lock(toml_path: PathBuf) -> Self {
        let file = File::open(toml_path).expect("Expected Nargo.toml to exist");
        if file.try_lock_exclusive().is_err() {
            eprintln!("Waiting for lock on Nargo.toml...");
        }

        file.lock_exclusive().expect("Failed to lock Nargo.toml");
        Self { file }
    }

    pub(crate) fn unlock(&self) {
        self.file.unlock().expect("Failed to unlock Nargo.toml");
    }
}
