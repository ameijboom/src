use std::path::Path;

use git2::{IndexAddOption, IntoCString};

pub struct Index(git2::Index);

impl From<git2::Index> for Index {
    fn from(index: git2::Index) -> Self {
        Self(index)
    }
}

impl Index {
    pub fn add(
        &mut self,
        pathspecs: impl IntoIterator<Item = impl IntoCString>,
        mut callback: impl FnMut(&Path),
    ) -> Result<i32, git2::Error> {
        let mut count = 0;

        self.0.add_all(
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
        self.0.write()
    }

    pub fn write_tree(&mut self) -> Result<git2::Oid, git2::Error> {
        self.0.write_tree()
    }
}
