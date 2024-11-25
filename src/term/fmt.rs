use std::fmt::Display;

use colored::{Color, Colorize};

pub struct FmtString {
    content: String,
    color: Option<Color>,
}

impl FmtString {
    pub fn new(content: String) -> Self {
        Self {
            content,
            color: None,
        }
    }

    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl Display for FmtString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.color {
            Some(color) => {
                if self.content.contains('\n') {
                    write!(
                        f,
                        "{}",
                        self.content
                            .lines()
                            .map(|line| line.color(color).to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    write!(f, "{}", self.content.color(color))
                }
            }
            None => write!(f, "{}", self.content),
        }
    }
}
