use std::string::FromUtf8Error;

use git2::{Oid, Reference, Repository};

#[derive(Debug, thiserror::Error)]
pub enum NameError {
    #[error("failed to find remote: {0}")]
    Git2(#[from] git2::Error),
    #[error("failed to parse UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("failed to parse UTF-8: {0}")]
    FromUtf8(#[from] FromUtf8Error),
}

pub fn find_remote_ref<'a>(
    repo: &'a Repository,
    refname: &str,
) -> Result<Reference<'a>, NameError> {
    let remote = repo.branch_upstream_name(refname)?;
    let remote = std::str::from_utf8(&remote)?;

    Ok(repo.find_reference(remote)?)
}

pub fn parse_remote(refname: &str) -> Option<&str> {
    if refname.starts_with("refs/") {
        return refname
            .strip_prefix("refs/remotes/")
            .and_then(|n| n.split('/').next());
    }

    let (remote, _) = refname.split_once('/')?;
    Some(remote)
}

pub fn short(oid: &Oid) -> String {
    oid.to_string().chars().take(7).collect()
}
