use std::fmt::Display;

use colored::{Color, Colorize};

pub struct FmtString {
    bold: bool,
    content: String,
    color: Option<Color>,
}

impl FmtString {
    pub fn new(content: String) -> Self {
        Self {
            content,
            bold: false,
            color: None,
        }
    }

    pub fn with_color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_bold(mut self) -> Self {
        self.bold = true;
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
                            .map(|line| {
                                let line = line.color(color);

                                if self.bold {
                                    line.bold()
                                } else {
                                    line
                                }
                            }
                            .to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else if self.bold {
                    write!(f, "{}", self.content.color(color).bold())
                } else {
                    write!(f, "{}", self.content.color(color))
                }
            }
            None if !self.bold => write!(f, "{}", self.content),
            None => write!(f, "{}", self.content.bold()),
        }
    }
}
