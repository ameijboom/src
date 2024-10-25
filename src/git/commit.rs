use std::error::Error;

use git2::{Config, Oid, Repository, Signature, Tree};

use crate::git::signer::{ssh::SshSigner, Signer};

use super::Optional;

pub struct Commit<'a> {
    config: &'a Config,
    tree: Tree<'a>,
    repo: &'a Repository,
}

impl<'a> Commit<'a> {
    pub fn build(config: &'a Config, repo: &'a Repository, tree: Tree<'a>) -> Self {
        Self { config, tree, repo }
    }

    pub fn create(
        &self,
        message: &str,
        author: Option<&Signature<'_>>,
        parent: Option<&git2::Commit<'_>>,
    ) -> Result<Oid, Box<dyn Error>> {
        let current_author = super::signature(&self.config)?;
        let author = author.unwrap_or(&current_author);
        let parent = match parent {
            Some(parent) => parent,
            None => &self.repo.head()?.peel_to_commit()?,
        };

        if self.config.get_bool("commit.gpgsign").optional()? == Some(true) {
            let signer: Box<dyn Signer> =
                match self.config.get_string("gpg.format").optional()?.as_deref() {
                    Some("ssh") => {
                        let signer = SshSigner::from_config(self.config)?;
                        Ok::<_, Box<dyn Error>>(Box::new(signer))
                    }
                    Some(format) => Err(format!("Unsupported gpg.format: {format}").into()),
                    None => Err("gpg.format not set".into()),
                }?;

            let buf =
                self.repo
                    .commit_create_buffer(author, author, message, &self.tree, &[parent])?;
            let signed = signer.sign(&buf)?;
            let content = std::str::from_utf8(&buf)?;

            Ok(self.repo.commit_signed(content, &signed, None)?)
        } else {
            Ok(self
                .repo
                .commit(None, author, author, message, &self.tree, &[parent])?)
        }
    }
}
