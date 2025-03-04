use std::{borrow::Cow, error::Error};

use crate::git::{Optional, Ref};

macro_rules! dimmed {
    ($content: expr) => {
        Node::Dimmed(Box::new($content))
    };
}

macro_rules! column {
    ($left: expr, $right: expr) => {
        Node::Column(Box::new($left), Box::new($right))
    };
}

macro_rules! text {
    ($text: expr) => {
        Node::Text($text.into())
    };
}

macro_rules! icon {
    ($name: ident) => {
        Node::Icon(Icon::$name)
    };
}

macro_rules! block {
    ($($content: expr),+) => {
        Node::Block(vec![$($content),+])
    };
}

macro_rules! breadcrumb {
    ($($content: expr),+) => {
        Node::Breadcrumb(vec![$($content),+])
    };
}

macro_rules! multi_line {
    ($($content: expr),+) => {
        Node::MultiLine(vec![$($content),+])
    };
}

macro_rules! label {
    ($content: expr) => {
        Node::Label(Box::new($content))
    };
}

macro_rules! continued {
    ($content: expr) => {
        Node::Continued(Box::new($content))
    };
}

macro_rules! spacer {
    () => {
        Node::spacer()
    };
}

pub(crate) use block;
pub(crate) use breadcrumb;
pub(crate) use column;
pub(crate) use continued;
pub(crate) use dimmed;
pub(crate) use icon;
pub(crate) use label;
pub(crate) use multi_line;
pub(crate) use spacer;
pub(crate) use text;

pub mod prelude {
    pub(crate) use super::{
        block, breadcrumb, continued, dimmed, icon, label, multi_line, spacer, text,
    };
    pub use super::{message_with_icon, Attribute, Icon, Indicator, Node, Status};
}

pub fn message_with_icon(icon: Icon, message: impl Into<Cow<'static, str>>) -> Node {
    Node::Block(vec![
        Node::Icon(icon),
        Node::spacer(),
        Node::Text(message.into()),
    ])
}

#[derive(Debug)]
pub enum Attribute {
    Commit(git2::Oid),
    CommitShort(git2::Oid),
    Branch(Cow<'static, str>),
    Remote(Cow<'static, str>),
    Operation(Cow<'static, str>),
}

impl Attribute {
    pub fn from_ref(reference: &Ref<'_>) -> Result<Attribute, Box<dyn Error>> {
        if reference.is_branch() || reference.is_tag() {
            Ok(Attribute::Branch(reference.shorthand()?.to_string().into()))
        } else {
            match reference.target().optional()? {
                Some(oid) => Ok(Attribute::CommitShort(oid)),
                None => Ok(Attribute::Remote(reference.shorthand()?.to_string().into())),
            }
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Error,
    Warning,
    Success,
}

#[derive(Debug)]
pub enum Icon {
    ArrowUp,
    ArrowDown,
    Lock,
    Check,
}

#[derive(Debug)]
pub enum Indicator {
    Unknown,
    New,
    Modified,
    Renamed,
    Deleted,
}

#[derive(Debug)]
pub enum Node {
    Empty,
    Icon(Icon),
    Label(Box<Node>),
    Block(Vec<Node>),
    Dimmed(Box<Node>),
    MultiLine(Vec<Node>),
    Indicator(Indicator),
    Continued(Box<Node>),
    Breadcrumb(Vec<Node>),
    Text(Cow<'static, str>),
    Attribute(Attribute),
    Status(Status, Box<Node>),
    Column(Box<Node>, Box<Node>),
    Group(Cow<'static, str>, Option<usize>, Box<Node>),
}

impl Node {
    pub fn spacer() -> Node {
        Node::Text(" ".into())
    }

    pub fn text_head_1(text: impl ToString) -> Node {
        Node::Text(
            text.to_string()
                .split('\n')
                .next()
                .unwrap_or_default()
                .to_string()
                .into(),
        )
    }

    pub fn text_capped(text: impl ToString, cap: usize) -> Node {
        let text = text.to_string();

        if text.len() > cap {
            Node::Text(format!("{}...", &text[..cap - 3]).into())
        } else {
            Node::Text(text.into())
        }
    }

    pub fn with_status(self, status: Status) -> Self {
        Node::Status(status, Box::new(self))
    }
}
