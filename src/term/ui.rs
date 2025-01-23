use std::{borrow::Cow, error::Error, fmt::Display};

use colored::{Color, Colorize};

use crate::git::{Optional, Ref};

const HEADER: Color = Color::TrueColor {
    r: 225,
    g: 190,
    b: 120,
};

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

#[derive(Default)]
pub struct Stream(usize);

impl Stream {
    pub fn send(&mut self, node: impl Into<Option<Node>>) -> &mut Self {
        if let Some(node) = node.into() {
            if !matches!(&node, Node::MultiLine(nodes) | Node::Block(nodes) if nodes.is_empty()) {
                if self.0 > 0 {
                    println!("\n");
                }

                self.0 += 1;

                println!("{node}");
            }
        }

        self
    }
}

#[derive(Default)]
pub struct Builder {
    nodes: Vec<Node>,
}

impl Builder {
    pub fn and(mut self, node: impl Into<Option<Node>>) -> Self {
        if let Some(node) = node.into() {
            if !matches!(&node, Node::MultiLine(nodes) | Node::Block(nodes) if nodes.is_empty()) {
                self.nodes.push(node);
            }
        }

        self
    }

    pub fn build(self) -> Node {
        Node::MultiLine(self.nodes)
    }
}

#[derive(Debug)]
pub enum Node {
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

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Dimmed(node) => write!(f, "{}", format!("{node}").bright_black()),
            Node::Text(text) => write!(f, "{}", text),
            Node::Block(children) => {
                for node in children {
                    write!(f, "{}", node)?;
                }

                Ok(())
            }
            Node::Continued(node) => write!(f, "↪ {}", node),
            Node::Breadcrumb(children) => {
                for (i, node) in children.iter().enumerate() {
                    if i > 0 {
                        write!(f, " › ")?;
                    }

                    write!(f, "{}", node)?;
                }

                Ok(())
            }
            Node::Attribute(attr) => match attr {
                Attribute::CommitShort(oid) => write!(
                    f,
                    "{}",
                    format!("{oid}")
                        .chars()
                        .take(7)
                        .collect::<String>()
                        .yellow()
                ),
                Attribute::Commit(oid) => write!(f, "{}", oid.to_string().yellow()),
                Attribute::Branch(name) => write!(f, "{}", format!(" {name}").blue()),
                Attribute::Remote(name) => write!(f, "{}", format!("⬡ {name}").cyan()),
            },
            Node::Group(heading, count, node) => {
                write!(f, "\n{}", format!("{heading}").color(HEADER).bold())?;

                if let Some(count) = count {
                    write!(f, " {}", format!("({})", count).dimmed())?;
                }

                write!(f, "\n{}", node)
            }
            Node::MultiLine(children) => {
                for (i, node) in children.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }

                    write!(f, "{}", node)?;
                }

                Ok(())
            }
            Node::Icon(icon) => match icon {
                Icon::ArrowUp => write!(f, "↑"),
                Icon::ArrowDown => write!(f, "↓"),
                Icon::Lock => write!(f, "⚿"),
                Icon::Check => write!(f, "✓"),
            },
            Node::Indicator(indicator) => match indicator {
                Indicator::Unknown => write!(f, "{}", "⚠".bright_black()),
                Indicator::New => write!(f, "{}", "✚".green()),
                Indicator::Modified => write!(f, "{}", "~".yellow()),
                Indicator::Renamed => write!(f, "{}", "➜".yellow()),
                Indicator::Deleted => write!(f, "{}", "✖".red()),
            },
            Node::Status(status, node) => {
                write!(
                    f,
                    "{}",
                    match status {
                        Status::Error => format!("{node}").red(),
                        Status::Warning => format!("{node}").yellow(),
                        Status::Success => format!("{node}").green(),
                    }
                )
            }
            Node::Label(node) => write!(f, "{}{node}{}", "(".dimmed(), ")".dimmed()),
            Node::Column(left, right) => write!(f, "{}: {}", left, right),
        }
    }
}
