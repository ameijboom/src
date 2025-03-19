use gix::{revision::walk::Info, Repository};

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("failed to iterate: {0}")]
    Iter(#[from] gix::revision::walk::iter::Error),
    #[error("failed to get all commits: {0}")]
    Platform(#[from] gix::revision::walk::Error),
    #[error("failed to find merge base: {0}")]
    Merge(#[from] gix::repository::merge_base::Error),
}

fn walk<'r>(
    repo: &'r Repository,
    base: gix::Id<'_>,
    tip: gix::Id<'_>,
) -> Result<Vec<Info<'r>>, GraphError> {
    Ok(repo
        .rev_walk([tip])
        .with_pruned([base])
        .all()?
        .collect::<Result<Vec<_>, _>>()?)
}

pub struct Graph<'r> {
    pub ahead: Vec<Info<'r>>,
    pub behind: Vec<Info<'r>>,
}

impl<'r> Graph<'r> {
    pub fn ahead_behind(
        repo: &'r Repository,
        left: gix::Id<'_>,
        right: gix::Id<'_>,
    ) -> Result<Graph<'r>, GraphError> {
        let merge_base = repo.merge_base(left, right)?;

        Ok(Graph {
            ahead: walk(repo, merge_base, left)?,
            behind: walk(repo, merge_base, right)?,
        })
    }
}
