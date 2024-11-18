use std::str::Utf8Error;

use chrono::{DateTime, Local};
use git2::Signature;

use crate::term::fmt::FmtString;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("invalid name: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("missing remote")]
    MissingRemote,
}

pub struct Tree<'a>(pub git2::Tree<'a>);

impl<'a> From<git2::Tree<'a>> for Tree<'a> {
    fn from(tree: git2::Tree<'a>) -> Self {
        Self(tree)
    }
}

#[allow(dead_code)]
pub struct Object<'a>(pub git2::Object<'a>);

impl<'a> From<git2::Object<'a>> for Object<'a> {
    fn from(object: git2::Object<'a>) -> Self {
        Self(object)
    }
}

pub struct Branch<'a>(pub git2::Branch<'a>);

impl<'a> From<git2::Branch<'a>> for Branch<'a> {
    fn from(branch: git2::Branch<'a>) -> Self {
        Self(branch)
    }
}

impl<'a> Branch<'a> {
    pub fn name(&self) -> Result<&str, Error> {
        Ok(std::str::from_utf8(self.0.name_bytes()?)?)
    }

    pub fn remote_name(&self) -> Result<&str, Error> {
        let name = self.name()?;

        if name.starts_with("refs/") {
            return name
                .strip_prefix("refs/remotes/")
                .and_then(|n| n.split('/').next())
                .ok_or(Error::MissingRemote);
        }

        name.split_once('/')
            .map(|(name, _)| name)
            .ok_or(Error::MissingRemote)
    }

    pub fn upstream(&self) -> Result<Branch<'a>, git2::Error> {
        self.0.upstream().map(Into::into)
    }

    pub fn set_upstream(&mut self, name: &str) -> Result<(), git2::Error> {
        self.0.set_upstream(Some(name))
    }

    pub fn target(&self) -> Result<git2::Oid, git2::Error> {
        self.0.get().target().ok_or_else(|| {
            git2::Error::new(
                git2::ErrorCode::NotFastForward,
                git2::ErrorClass::Reference,
                "missing target",
            )
        })
    }

    pub fn into_ref(self) -> Ref<'a> {
        Ref(self.0.into_reference())
    }
}

pub struct Commit<'a>(pub git2::Commit<'a>);

impl<'a> From<git2::Commit<'a>> for Commit<'a> {
    fn from(commit: git2::Commit<'a>) -> Self {
        Self(commit)
    }
}

impl<'a> Commit<'a> {
    pub fn id(&self) -> git2::Oid {
        self.0.id()
    }

    pub fn find_tree(&self) -> Result<Tree<'a>, git2::Error> {
        self.0.tree().map(Into::into)
    }

    pub fn author(&self) -> Signature<'_> {
        self.0.author()
    }

    pub fn time(&self) -> DateTime<Local> {
        super::parse_local_time(self.0.time())
    }

    pub fn headers_formatted(&self) -> FmtString {
        FmtString::new(format!(
            "Date: {}\nAuthor: {}",
            self.time().format("%Y-%m-%d %H:%M"),
            self.author()
        ))
    }

    pub fn message(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.0.message_bytes())
    }

    pub fn message_formatted(&self) -> String {
        self.message()
            .map(|msg| {
                msg.lines()
                    .map(|l| format!("  {}", l))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }

    pub fn parent(&self) -> Result<Option<Commit<'a>>, git2::Error> {
        self.parent_n(1)
    }

    pub fn parent_n(&self, n: usize) -> Result<Option<Commit<'a>>, git2::Error> {
        if self.0.parent_count() == 0 {
            return Ok(None);
        }

        let parent = self.0.parent(0)?.into();

        if n == 1 {
            return Ok(Some(parent));
        }

        parent.parent_n(n - 1)
    }

    pub fn is_signed(&self) -> bool {
        self.0
            .header_field_bytes("gpgsig")
            .map(|sig| !sig.is_empty())
            .unwrap_or(false)
    }
}

pub struct Ref<'a>(pub git2::Reference<'a>);

impl<'a> From<git2::Reference<'a>> for Ref<'a> {
    fn from(reference: git2::Reference<'a>) -> Self {
        Self(reference)
    }
}

impl<'a> From<Branch<'a>> for Ref<'a> {
    fn from(branch: Branch<'a>) -> Self {
        Ref(branch.0.into_reference())
    }
}

impl<'a> Ref<'a> {
    pub fn name(&self) -> Result<&str, Error> {
        Ok(std::str::from_utf8(self.0.name_bytes())?)
    }

    pub fn shorthand(&self) -> Result<&str, Error> {
        Ok(std::str::from_utf8(self.0.shorthand_bytes())?)
    }

    pub fn find_commit(&self) -> Result<Commit<'_>, git2::Error> {
        self.0.peel_to_commit().map(Into::into)
    }

    pub fn find_tree(&self) -> Result<Tree<'a>, git2::Error> {
        self.0.peel_to_tree().map(Into::into)
    }

    pub fn into_branch(self) -> Result<Branch<'a>, git2::Error> {
        Ok(git2::Branch::wrap(self.0).into())
    }

    pub fn target(&self) -> Result<git2::Oid, git2::Error> {
        self.0.target().ok_or_else(|| {
            git2::Error::new(
                git2::ErrorCode::NotFastForward,
                git2::ErrorClass::Reference,
                "missing target",
            )
        })
    }

    pub fn set_target(&mut self, oid: git2::Oid, message: &str) -> Result<Ref<'_>, git2::Error> {
        self.0.set_target(oid, message).map(Into::into)
    }

    pub fn is_branch(&self) -> bool {
        self.0.is_branch()
    }

    pub fn is_tag(&self) -> bool {
        self.0.is_tag()
    }
}
