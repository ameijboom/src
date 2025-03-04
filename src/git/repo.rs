use std::{borrow::Cow, error::Error};

use git2::{
    build::CheckoutBuilder, string_array::StringArray, BranchType, DiffFindOptions, DiffOptions,
    ErrorClass, ErrorCode, RebaseOptions, StashApplyOptions, StashFlags, StatusOptions,
};

use crate::git::signer::{ssh::SshSigner, Signer};

use super::{
    config::Config,
    index::Index,
    objects::{Branch, Commit, Ref, Tree},
    rebase::{Rebase, RebaseError},
    remote::Remote,
    status::Status,
};

#[derive(Debug, thiserror::Error)]
pub enum CheckoutError {
    #[error("checkout results in conflict: {0}")]
    Conflict(git2::Error),
    #[error("git error: {0}")]
    Git(git2::Error),
}

impl From<git2::Error> for CheckoutError {
    fn from(e: git2::Error) -> Self {
        match e.code() {
            git2::ErrorCode::Conflict if e.class() == ErrorClass::Checkout => Self::Conflict(e),
            _ => Self::Git(e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StashError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("config error: {0}")]
    Config(#[from] super::config::Error),
}

pub struct Remotes<'a> {
    i: usize,
    repo: &'a Repo,
    names: StringArray,
}

impl<'a> Iterator for Remotes<'a> {
    type Item = Result<Remote<'a>, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.names.len() {
            let name = self.names.get(self.i).unwrap();
            self.i += 1;

            Some(self.repo.find_remote(name))
        } else {
            None
        }
    }
}

enum DiffType<'a> {
    All(&'a Tree<'a>),
    Staged(&'a Tree<'a>),
    Unstaged,
}

pub struct DiffOpts<'a> {
    ty: DiffType<'a>,
    diff_opts: DiffOptions,
}

impl Default for DiffOpts<'_> {
    fn default() -> Self {
        let mut opts = DiffOptions::new();
        opts.force_text(true)
            .ignore_whitespace(true)
            .ignore_whitespace_change(false)
            .include_ignored(false)
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .show_untracked_content(true);

        Self {
            ty: DiffType::Unstaged,
            diff_opts: opts,
        }
    }
}

impl<'a> DiffOpts<'a> {
    pub fn with_all(mut self, tree: &'a Tree<'a>) -> Self {
        self.ty = DiffType::All(tree);
        self
    }

    pub fn with_staged(mut self, tree: &'a Tree<'a>) -> Self {
        self.ty = DiffType::Staged(tree);
        self
    }

    pub fn with_pathspec(mut self, pathspec: &str) -> Self {
        self.diff_opts.pathspec(pathspec);
        self
    }
}

fn map_unique_commits(
    repo: &git2::Repository,
    base: git2::Oid,
    tip: git2::Oid,
) -> Result<Vec<Commit<'_>>, git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push(tip)?;
    revwalk.hide(base)?;
    let oids = revwalk.collect::<Result<Vec<_>, _>>()?;

    oids.into_iter()
        .map(|oid| repo.find_commit(oid).map(Commit::from))
        .collect::<Result<Vec<_>, _>>()
}

pub struct Repo {
    repo: git2::Repository,
}

impl From<git2::Repository> for Repo {
    fn from(repo: git2::Repository) -> Self {
        Self { repo }
    }
}

impl Repo {
    pub fn head(&self) -> Result<Ref<'_>, git2::Error> {
        self.repo.head().map(Into::into)
    }

    pub fn find_commit(&self, oid: git2::Oid) -> Result<Commit<'_>, git2::Error> {
        self.repo.find_commit(oid).map(Into::into)
    }

    pub fn find_annotated_commit(
        &self,
        oid: git2::Oid,
    ) -> Result<git2::AnnotatedCommit, git2::Error> {
        self.repo.find_annotated_commit(oid)
    }

    pub fn merge_analysis(
        &self,
        commit: &git2::AnnotatedCommit,
    ) -> Result<(git2::MergeAnalysis, git2::MergePreference), git2::Error> {
        self.repo.merge_analysis(&[commit])
    }

    pub fn find_tree(&self, oid: git2::Oid) -> Result<Tree<'_>, git2::Error> {
        self.repo.find_tree(oid).map(Into::into)
    }

    pub fn find_remote(&self, name: &str) -> Result<Remote<'_>, git2::Error> {
        self.repo.find_remote(name).map(Into::into)
    }

    pub fn find_ref(&self, name: &str) -> Result<Ref<'_>, git2::Error> {
        self.repo.find_reference(name).map(Into::into)
    }

    pub fn find_branch(&self, name: &str) -> Result<Branch<'_>, git2::Error> {
        self.repo
            .find_branch(name, BranchType::Local)
            .map(Into::into)
    }

    pub fn find_remote_branch(&self, name: &str) -> Result<Branch<'_>, git2::Error> {
        self.repo
            .find_branch(name, BranchType::Remote)
            .map(Into::into)
    }

    pub fn checkout_tree(&self, Tree(tree): &Tree<'_>, force: bool) -> Result<(), git2::Error> {
        let mut cb = CheckoutBuilder::default();

        if force {
            cb.force();
        } else {
            cb.safe();
        }

        self.repo.checkout_tree(tree.as_object(), Some(&mut cb))
    }

    pub fn checkout(&self, reference: &Ref<'_>) -> Result<(), CheckoutError> {
        let tree = reference.find_tree()?;

        self.checkout_tree(&tree, false)?;
        self.repo.set_head_bytes(reference.0.name_bytes())?;

        Ok(())
    }

    pub fn read_rebase(&self) -> Result<Rebase, RebaseError> {
        Rebase::from_path(&self.repo.path().join("rebase-merge/git-rebase-todo.backup"))
    }

    pub fn rebase(
        &self,
        branch: &git2::AnnotatedCommit<'_>,
        upstream: &git2::AnnotatedCommit<'_>,
    ) -> Result<Option<git2::Oid>, RebaseError> {
        let mut cb = CheckoutBuilder::default();
        cb.safe();

        let mut rebase = self.repo.rebase(
            Some(branch),
            Some(upstream),
            None,
            Some(RebaseOptions::default().checkout_options(cb)),
        )?;

        let config = Config::open_default()?;
        let author = config.user.signature()?;
        let mut oid = None;

        while let Some(op) = rebase.next() {
            let run = || {
                let op = op?;
                rebase.commit(None, &author, None)?;
                Ok::<_, RebaseError>(op.id())
            };

            match run() {
                Ok(new_oid) => oid = Some(new_oid),
                Err(e) => return Err(e),
            }
        }

        rebase.finish(None)?;

        Ok(oid)
    }

    pub fn branches(
        &self,
    ) -> Result<impl Iterator<Item = Result<Branch<'_>, git2::Error>> + '_, git2::Error> {
        Ok(self
            .repo
            .branches(Some(BranchType::Local))?
            .map(|result| result.map(|(branch, _)| branch.into())))
    }

    pub fn remotes(
        &self,
    ) -> Result<impl Iterator<Item = Result<Remote<'_>, git2::Error>> + '_, git2::Error> {
        let names = self.repo.remotes()?;
        Ok(Remotes {
            i: 0,
            repo: self,
            names,
        })
    }

    pub fn create_branch(
        &self,
        name: &str,
        Commit(commit): &Commit,
    ) -> Result<Branch<'_>, git2::Error> {
        self.repo.branch(name, commit, false).map(Into::into)
    }

    pub fn commits(
        &self,
        reference: &Ref<'_>,
    ) -> Result<impl Iterator<Item = Result<Commit<'_>, git2::Error>>, git2::Error> {
        let mut walker = self.repo.revwalk()?;
        walker.push_ref(
            reference
                .name()
                .map_err(|e| git2::Error::new(ErrorCode::User, ErrorClass::None, e.to_string()))?,
        )?;

        Ok(walker.map(|oid| oid.and_then(|oid| self.find_commit(oid))))
    }

    pub fn stashes(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<Commit<'_>, git2::Error>>, git2::Error> {
        let mut stashes = vec![];

        self.repo.stash_foreach(|_, _, oid| {
            stashes.push(*oid);
            true
        })?;

        Ok(stashes.into_iter().map(|oid| self.find_commit(oid)))
    }

    pub fn pop_stash(&mut self, index: usize) -> Result<(), git2::Error> {
        let mut cb = CheckoutBuilder::default();
        cb.safe();

        self.repo.stash_pop(
            index,
            Some(StashApplyOptions::default().checkout_options(cb)),
        )
    }

    pub fn save_stash(&mut self, message: &str) -> Result<git2::Oid, StashError> {
        let config = Config::open_default()?;
        let signature = config.user.signature()?;

        Ok(self
            .repo
            .stash_save(&signature, message, Some(StashFlags::INCLUDE_UNTRACKED))?)
    }

    pub fn create_ref(&self, name: &str, target: git2::Oid) -> Result<Ref<'_>, git2::Error> {
        self.repo.reference(name, target, true, "").map(Into::into)
    }

    pub fn create_commit(
        &self,
        tree: &Tree<'_>,
        message: &str,
        parent: Option<&Commit<'_>>,
    ) -> Result<git2::Oid, Box<dyn Error>> {
        let config = Config::open_default()?;
        let author = config.user.signature()?;
        let parent_commit = match parent {
            Some(parent) => Some(Cow::Borrowed(&parent.0)),
            None => match self.repo.head() {
                Ok(head) => Some(Cow::Owned(head.peel_to_commit()?)),
                Err(e) if e.code() == ErrorCode::UnbornBranch => None,
                Err(e) => return Err(e.into()),
            },
        };
        let parents = parent_commit
            .as_ref()
            .map(|c| vec![c.as_ref()])
            .unwrap_or_default();

        if config.commit.gpg_sign {
            match config.gpg.format {
                Some(super::config::GpgFormat::Ssh) => {
                    let signer = SshSigner::from_config(&config)?;
                    let buf = self
                        .repo
                        .commit_create_buffer(&author, &author, message, &tree.0, &parents)?;
                    let signed = signer.sign(&buf)?;
                    let content = std::str::from_utf8(&buf)?;

                    Ok(self.repo.commit_signed(content, &signed, None)?)
                }
                None => Err("gpg.format unsupported".into()),
            }
        } else {
            Ok(self
                .repo
                .commit(None, &author, &author, message, &tree.0, &parents)?)
        }
    }

    pub fn diff(&self, mut opts: DiffOpts) -> Result<git2::Diff, git2::Error> {
        let mut diff = match opts.ty {
            DiffType::All(tree) => self
                .repo
                .diff_tree_to_workdir_with_index(Some(&tree.0), Some(&mut opts.diff_opts))?,
            DiffType::Staged(tree) => {
                self.repo
                    .diff_tree_to_index(Some(&tree.0), None, Some(&mut opts.diff_opts))?
            }
            DiffType::Unstaged => self
                .repo
                .diff_index_to_workdir(None, Some(&mut opts.diff_opts))?,
        };

        let mut find_opts = DiffFindOptions::new();
        diff.find_similar(Some(find_opts.renames(true).copies(true)))?;

        Ok(diff)
    }

    pub fn index(&self) -> Result<Index, git2::Error> {
        self.repo.index().map(Into::into)
    }

    pub fn status(&self) -> Result<Status, git2::Error> {
        Ok(Status(
            self.repo.statuses(Some(
                StatusOptions::new()
                    .include_ignored(false)
                    .include_untracked(true)
                    .recurse_untracked_dirs(true)
                    .exclude_submodules(true)
                    .rename_threshold(50)
                    .renames_head_to_index(true),
            ))?,
        ))
    }

    pub fn find_upstream_branch(
        &self,
        reference: &Ref<'_>,
    ) -> Result<Option<Ref<'_>>, Box<dyn Error>> {
        let name = reference.name()?;

        match self.repo.branch_upstream_name(name) {
            Ok(remote) => Ok(Some(self.find_ref(std::str::from_utf8(&remote)?)?)),
            Err(e) if e.code() == ErrorCode::NotFound && e.class() == ErrorClass::Config => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn graph_ahead_behind(
        &self,
        local: git2::Oid,
        remote: git2::Oid,
    ) -> Result<(usize, usize), git2::Error> {
        self.repo.graph_ahead_behind(local, remote)
    }

    pub fn commits_ahead_behind(
        &self,
        local: git2::Oid,
        remote: git2::Oid,
    ) -> Result<(Vec<Commit<'_>>, Vec<Commit<'_>>), git2::Error> {
        let merge_base = self.repo.merge_base(local, remote)?;
        let ahead = map_unique_commits(&self.repo, merge_base, local)?;
        let behind = map_unique_commits(&self.repo, merge_base, remote)?;

        Ok((ahead, behind))
    }

    pub fn state(&self) -> git2::RepositoryState {
        self.repo.state()
    }
}
