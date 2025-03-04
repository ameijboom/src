use std::{fs, path::Path, str::FromStr};

#[derive(Debug, thiserror::Error)]
pub enum RebaseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("config error: {0}")]
    Config(#[from] super::config::Error),
    #[error("invalid rebase todo: {0}")]
    Parse(String),
}

pub struct RebaseOp {
    pub oid: git2::Oid,
    pub ty: git2::RebaseOperationType,
    pub message: String,
}

impl FromStr for RebaseOp {
    type Err = RebaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //pick 6b6829f3547a8c3c30b120e5c7cc26ea51ee745e feat: send careplan notifications
        let components = s.splitn(3, ' ').collect::<Vec<_>>();

        if components.len() != 3 {
            return Err(RebaseError::Parse(
                "invalid rebase todo: expected 3 components".to_string(),
            ));
        }

        let ty = match components[0] {
            "p" | "pick" => git2::RebaseOperationType::Pick,
            "r" | "reword" => git2::RebaseOperationType::Reword,
            "e" | "edit" => git2::RebaseOperationType::Edit,
            "s" | "squash" => git2::RebaseOperationType::Squash,
            "f" | "fixup" => git2::RebaseOperationType::Fixup,
            "x" | "exec" => git2::RebaseOperationType::Exec,
            _ => {
                return Err(RebaseError::Parse(
                    "invalid rebase operation type".to_string(),
                ))
            }
        };

        Ok(Self {
            oid: git2::Oid::from_str(components[1])?,
            ty,
            message: components[2].to_string(),
        })
    }
}

pub struct Rebase {
    pub operations: Vec<RebaseOp>,
}

impl Rebase {
    pub fn from_path(path: &Path) -> Result<Self, RebaseError> {
        let operations = fs::read_to_string(path)?
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("#") && !line.starts_with(" "))
            .map(RebaseOp::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { operations })
    }
}
