use std::{fs, path::Path, str::FromStr};

use gix::Repository;

#[derive(Debug, thiserror::Error)]
pub enum RebaseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid rebase todo: {0}")]
    Parse(String),
    #[error("invalid object id: {0}")]
    ObjectId(#[from] gix_hash::decode::Error),
}

pub enum RebaseOperationType {
    Pick,
    Reword,
    Edit,
    Squash,
    Fixup,
    Exec,
}

pub struct RebaseOp {
    pub oid: gix::ObjectId,
    pub ty: RebaseOperationType,
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
            "p" | "pick" => RebaseOperationType::Pick,
            "r" | "reword" => RebaseOperationType::Reword,
            "e" | "edit" => RebaseOperationType::Edit,
            "s" | "squash" => RebaseOperationType::Squash,
            "f" | "fixup" => RebaseOperationType::Fixup,
            "x" | "exec" => RebaseOperationType::Exec,
            _ => {
                return Err(RebaseError::Parse(
                    "invalid rebase operation type".to_string(),
                ))
            }
        };

        Ok(Self {
            oid: gix::ObjectId::from_str(components[1])?,
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

    pub fn from_repo(repo: &Repository) -> Result<Self, RebaseError> {
        Rebase::from_path(&repo.path().join("rebase-merge/git-rebase-todo.backup"))
    }
}
