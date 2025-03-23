use std::fmt::{self, Arguments};
use std::io::Write;

use colored::{Color, Colorize};

use crate::term::node::Status;

use super::node::{Attribute, Icon, Indicator, Node};

const HEADER: Color = Color::TrueColor {
    r: 225,
    g: 190,
    b: 120,
};

pub struct WriteFmt<T: Write>(pub T);

impl<T: Write> fmt::Write for WriteFmt<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub trait Render {
    fn render(&mut self, node: &Node) -> fmt::Result;
    fn renderln(&mut self, node: &Node) -> fmt::Result {
        self.render(node)?;
        self.render(&Node::Text("\n".into()))
    }
}

pub struct TermRenderer<W: fmt::Write> {
    writer: W,
    color: Option<Color>,
}

impl<W: fmt::Write> TermRenderer<W> {
    pub fn new(w: W) -> Self {
        Self {
            writer: w,
            color: None,
        }
    }

    pub fn render_with(&mut self, node: &Node, color: Color) -> fmt::Result {
        let state = self.color.take();
        self.color = Some(color);
        self.render(node)?;
        self.color = state;

        Ok(())
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> fmt::Result {
        match self.color {
            Some(color) => self
                .writer
                .write_str(&fmt::format(args).color(color).to_string()),
            None => self.writer.write_fmt(args),
        }
    }
}

impl Default for TermRenderer<WriteFmt<std::io::Stdout>> {
    fn default() -> Self {
        Self::new(WriteFmt(std::io::stdout()))
    }
}

macro_rules! write {
    ($dst:expr, $($arg:tt)*) => {
        $dst.write_fmt(format_args!($($arg)*))
    };
}

impl<W: fmt::Write> Render for TermRenderer<W> {
    fn render(&mut self, node: &Node) -> fmt::Result {
        match node {
            Node::Dimmed(node) => self.render_with(node, Color::BrightBlack),
            Node::Text(text) => write!(self, "{text}"),
            Node::Block(children) => {
                for node in children {
                    self.render(node)?;
                }

                Ok(())
            }
            Node::Continued(node) => {
                write!(self, "↪ ")?;
                self.render(node)
            }
            Node::Breadcrumb(children) => {
                for (i, node) in children.iter().enumerate() {
                    if i > 0 {
                        write!(self, " › ")?;
                    }

                    self.render(node)?;
                }

                Ok(())
            }
            Node::Attribute(attr) => match attr {
                Attribute::CommitShort(oid) => write!(
                    self,
                    "{}",
                    format!("{oid}")
                        .chars()
                        .take(7)
                        .collect::<String>()
                        .yellow()
                ),
                Attribute::Commit(oid) => write!(self.writer, "{}", oid.to_string().yellow()),
                Attribute::Tag(name) => write!(self, "{}", format!("#{name}").blue()),
                Attribute::Branch(name) => write!(self, "{}", format!(" {name}").blue()),
                Attribute::Remote(name) => write!(self, "{}", format!("⬡ {name}").cyan()),
                Attribute::Operation(name) => write!(self, "{}", format!("↻ {name}").cyan()),
            },
            Node::Group(heading, count, node) => {
                write!(self, "\n{}", format!("{heading}").color(HEADER).bold())?;

                if let Some(count) = count {
                    write!(self, " {}", format!("({})", count).dimmed())?;
                }

                writeln!(self)?;
                self.render(node)
            }
            Node::MultiLine(children) => {
                for (i, node) in children.iter().enumerate() {
                    if i > 0 {
                        writeln!(self)?;
                    }

                    self.render(node)?;
                }

                Ok(())
            }
            Node::Icon(icon) => match icon {
                Icon::ArrowUp => write!(self.writer, "↑"),
                Icon::ArrowDown => write!(self, "↓"),
                Icon::Lock => write!(self, "⚿"),
                Icon::Check => write!(self, "✓"),
            },
            Node::Indicator(indicator) => match indicator {
                Indicator::Unknown => write!(self, "{}", "?".bright_black()),
                Indicator::Conflict => write!(self, "{}", "⚠".yellow()),
                Indicator::New => write!(self, "{}", "✚".green()),
                Indicator::Modified => write!(self, "{}", "~".yellow()),
                Indicator::Renamed => write!(self, "{}", "➜".yellow()),
                Indicator::Deleted => write!(self, "{}", "✖".red()),
            },
            Node::Status(status, node) => match status {
                Status::Error => self.render_with(node, Color::Red),
                Status::Warning => self.render_with(node, Color::Yellow),
                Status::Success => self.render_with(node, Color::Green),
            },
            Node::Label(node) => {
                write!(self, "{}", "(".dimmed())?;
                self.render(node)?;
                write!(self, "{}", ")".dimmed())
            }
            Node::Column(left, right) => {
                self.render(left)?;
                write!(self, ": ")?;
                self.render(right)
            }
            Node::Empty => Ok(()),
        }
    }
}
