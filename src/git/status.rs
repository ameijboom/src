use git2::{StatusEntry, Statuses};

pub struct Status<'a>(pub Statuses<'a>);

impl<'a> Status<'a> {
    pub fn entries(&'a self) -> impl Iterator<Item = Entry<'a>> {
        self.0
            .into_iter()
            .filter(|e| e.status() != git2::Status::CURRENT)
            .map(Entry::from)
    }
}

pub struct Entry<'a> {
    entry: StatusEntry<'a>,
}

impl Entry<'_> {
    pub fn path(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.entry.path_bytes())
    }
}

impl<'a> From<StatusEntry<'a>> for Entry<'a> {
    fn from(entry: StatusEntry<'a>) -> Self {
        Self { entry }
    }
}
