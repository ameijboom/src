use std::str::Utf8Error;

use git2::{Branch, Reference};

#[derive(Debug, thiserror::Error)]
pub enum NameError {
    #[error("invalid name: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("failed to get name: {0}")]
    Git(#[from] git2::Error),
}

pub trait Named {
    fn name_checked(&self) -> Result<&str, NameError>;
}

impl Named for Branch<'_> {
    fn name_checked(&self) -> Result<&str, NameError> {
        std::str::from_utf8(self.name_bytes()?).map_err(Into::into)
    }
}

impl Named for Reference<'_> {
    fn name_checked(&self) -> Result<&str, NameError> {
        std::str::from_utf8(self.name_bytes()).map_err(Into::into)
    }
}
