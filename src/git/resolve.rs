use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::i32,
    IResult, Parser,
};

use super::{Optional, Repo};

#[derive(Debug, PartialEq)]
pub enum Pattern<'a> {
    Head,
    Branch(&'a str),
    Parent((usize, Box<Pattern<'a>>)),
}

fn prefix(pattern: &str) -> IResult<&str, Pattern<'_>> {
    let (input, name) = alt((
        tag("HEAD"),
        tag("@"),
        take_till(|c| c == '@' || c == '^' || c == '~'),
    ))
    .parse(pattern)?;

    match name {
        "@" | "HEAD" => Ok((input, Pattern::Head)),
        _ => Ok((input, Pattern::Branch(name))),
    }
}

fn parent(pattern: &str) -> IResult<&str, Pattern<'_>> {
    let (input, (prefix, _, n)) = (prefix, tag("~"), i32).parse(pattern)?;
    Ok((input, Pattern::Parent((n as usize, Box::new(prefix)))))
}

impl<'a> Pattern<'a> {
    pub fn parse(pattern: &'a str) -> IResult<&'a str, Self> {
        let (input, name) = alt((parent, prefix)).parse(pattern)?;
        Ok((input, name))
    }

    pub fn resolve(&self, repo: &Repo) -> Result<Option<git2::Oid>, git2::Error> {
        match self {
            Pattern::Head => Ok(Some(repo.head()?.target()?)),
            Pattern::Branch(name) => repo.find_branch(name).and_then(|b| b.target()).optional(),
            Pattern::Parent((n, pat)) => match pat.resolve(repo)? {
                Some(oid) => Ok(repo.find_commit(oid)?.parent_n(*n)?.map(|c| c.id())),
                None => Ok(None),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head() {
        let pattern = "HEAD";
        let (input, pattern) = Pattern::parse(pattern).unwrap();
        assert_eq!(input, "");
        assert_eq!(pattern, Pattern::Head);
    }

    #[test]
    fn test_parent() {
        let pattern = "main~2";
        let (input, pattern) = Pattern::parse(pattern).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            pattern,
            Pattern::Parent((2, Box::new(Pattern::Branch("main"))))
        );
    }
}
