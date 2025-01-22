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

pub enum Change {
    New,
    Type,
    Modified,
    Renamed,
    Deleted,
}

pub enum EntryStatus {
    Unknown,
    WorkTree(Change),
    Index(Change),
}

pub struct Entry<'a> {
    entry: StatusEntry<'a>,
}

impl<'a> Entry<'a> {
    pub fn path(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.entry.path_bytes())
    }

    pub fn is_staged(&self) -> bool {
        self.entry.status().is_index_new()
            || self.entry.status().is_index_modified()
            || self.entry.status().is_index_deleted()
            || self.entry.status().is_index_renamed()
            || self.entry.status().is_index_typechange()
    }

    pub fn status(&self) -> EntryStatus {
        match self.entry.status() {
            s if s.is_wt_renamed() => EntryStatus::WorkTree(Change::Renamed),
            s if s.is_index_renamed() => EntryStatus::Index(Change::Renamed),
            s if s.is_wt_new() => EntryStatus::WorkTree(Change::New),
            s if s.is_index_new() => EntryStatus::Index(Change::New),
            s if s.is_wt_modified() => EntryStatus::WorkTree(Change::Modified),
            s if s.is_index_modified() => EntryStatus::Index(Change::Modified),
            s if s.is_wt_deleted() => EntryStatus::WorkTree(Change::Deleted),
            s if s.is_index_deleted() => EntryStatus::Index(Change::Deleted),
            s if s.is_wt_typechange() => EntryStatus::WorkTree(Change::Type),
            s if s.is_index_typechange() => EntryStatus::Index(Change::Type),
            _ => EntryStatus::Unknown,
        }
    }
}

impl<'a> From<StatusEntry<'a>> for Entry<'a> {
    fn from(entry: StatusEntry<'a>) -> Self {
        Self { entry }
    }
}
