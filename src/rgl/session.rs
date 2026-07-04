use super::get_dot_regolith_dir;
use anyhow::{bail, Result};
use fslock::LockFile;
use std::fs;

pub struct Session {
    file: LockFile,
}

impl Session {
    pub fn lock() -> Result<Self> {
        let dot_regolith = get_dot_regolith_dir()?;
        let _ = fs::create_dir_all(&dot_regolith);
        let mut file = LockFile::open(&dot_regolith.join("session_lock"))?;
        file.try_lock_with_pid()?;
        if !file.owns_lock() {
            bail!(
                "Failed to acquire session lock\n\
                 <yellow> >></> Another instance of rgl is already running\n\
                 <yellow> >></> If you are sure that this is not the case, delete the lock file manually"
            );
        }
        Ok(Self { file })
    }

    pub fn unlock(&mut self) -> Result<()> {
        self.file.unlock()?;
        Ok(())
    }
}
