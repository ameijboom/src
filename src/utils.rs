use std::string::FromUtf8Error;

use git2::{Error, ErrorClass, ErrorCode, Oid, Reference, Repository, StatusOptions, Statuses};

#[derive(Debug, thiserror::Error)]
pub enum FindRemoteError {
    #[error("failed to find remote: {0}")]
    Git2(#[from] git2::Error),
    #[error("failed to parse UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("failed to parse UTF-8: {0}")]
    FromUtf8(#[from] FromUtf8Error),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse remote")]
pub struct ParseRemoteError;

pub fn find_remote_ref<'a>(
    repo: &'a Repository,
    refname: &str,
) -> Result<Reference<'a>, FindRemoteError> {
    let remote = repo.branch_upstream_name(refname)?;
    let remote = std::str::from_utf8(&remote)?;

    Ok(repo.find_reference(remote)?)
}

pub fn parse_remote(refname: &str) -> Result<&str, ParseRemoteError> {
    if refname.starts_with("refs/") {
        return refname
            .strip_prefix("refs/remotes/")
            .and_then(|n| n.split('/').next())
            .ok_or(ParseRemoteError);
    }

    let (remote, _) = refname.split_once('/').ok_or(ParseRemoteError)?;
    Ok(remote)
}

pub fn short(oid: &Oid) -> String {
    oid.to_string().chars().take(7).collect()
}

pub fn config_opt<T>(result: Result<T, Error>) -> Result<Option<T>, Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(e) if e.code() == ErrorCode::NotFound && e.class() == ErrorClass::Config => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn status_entries(repo: &Repository) -> Result<Statuses<'_>, git2::Error> {
    let mut opts = StatusOptions::new();
    repo.statuses(Some(
        opts.include_ignored(false)
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .exclude_submodules(true),
    ))
}
