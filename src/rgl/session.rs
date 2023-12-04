use anyhow::{bail, Result};
use fslock::LockFile;
use std::fs;

pub struct Session {
    file: LockFile,
}

impl Session {
    pub fn lock() -> Result<Self> {
        let _ = fs::create_dir(".regolith");
        let mut file = LockFile::open(".regolith/session_lock")?;
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
