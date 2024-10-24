use std::path::Path;

use git2::{IndexAddOption, IntoCString, Repository};

pub struct Index<'a> {
    repo: &'a Repository,
    index: git2::Index,
}

impl<'a> Index<'a> {
    pub fn build(repo: &'a Repository) -> Result<Self, git2::Error> {
        Ok(Self {
            index: repo.index()?,
            repo,
        })
    }

    pub fn add(
        &mut self,
        pathspecs: impl Iterator<Item = impl IntoCString>,
        mut callback: impl FnMut(&Path),
    ) -> Result<i32, git2::Error> {
        let mut count = 0;

        self.index.add_all(
            pathspecs,
            IndexAddOption::DEFAULT,
            Some(&mut |path, _| {
                count += 1;
                callback(path);
                0
            }),
        )?;

        Ok(count)
    }

    pub fn write(&mut self) -> Result<(), git2::Error> {
        self.index.write()
    }

    pub fn write_tree(&mut self) -> Result<git2::Tree<'a>, git2::Error> {
        let oid = self.index.write_tree()?;
        self.repo.find_tree(oid)
    }
}
