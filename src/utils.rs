use std::string::FromUtf8Error;

use git2::Oid;

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
