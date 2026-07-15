//! Shared helpers for `necom pl` pipeline subcommands.
//!
//! Pure pipeline orchestration logic (no clap dependency): workflow context
//! and path resolution for `pl condense`.

mod ctx;

pub use ctx::PipelineCtx;

use std::path::PathBuf;

/// RAII guard that restores the working directory on drop.
pub struct CwdGuard {
    prev_dir: PathBuf,
}

impl CwdGuard {
    /// Change to `new_dir` and return a guard that restores the previous
    /// directory on drop.
    pub fn enter(new_dir: &str) -> anyhow::Result<Self> {
        let prev_dir = std::env::current_dir()?;
        std::env::set_current_dir(new_dir)?;
        Ok(Self { prev_dir })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        if let Err(e) = std::env::set_current_dir(&self.prev_dir) {
            log::warn!("failed to restore working directory: {}", e);
        }
    }
}
