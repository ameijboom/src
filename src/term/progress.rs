use std::{
    io::{stdout, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

use colored::Colorize;

const MOVE_BEGIN: &str = "\r";
const MOVE_UP: &str = "\x1B[1A";
const MOVE_DOWN: &str = "\x1B[1B";
const MOVE_RIGHT: &str = "\x1b[1C";
const DELETE_CHAR: &str = "\x08";
const CLEAR_LINE: &str = "\x1B[2K";

fn decode_chars(s: &str) -> Vec<char> {
    String::from_utf8_lossy(&strip_ansi_escapes::strip(s.as_bytes()))
        .chars()
        .collect()
}

fn move_begin() {
    print!("{MOVE_BEGIN}");
}

fn print_n(n: usize, s: &str) {
    for _ in 0..n {
        print!("{s}");
    }
}

fn move_up(n: usize) {
    print_n(n, MOVE_UP);
}

fn move_down(n: usize) {
    print_n(n, MOVE_DOWN);
}

fn move_right(n: usize) {
    print_n(n, MOVE_RIGHT);
}

fn delete_char() {
    print!("{DELETE_CHAR}");
}

fn erase_to_end() {
    print!("\x1B[0K");
}

struct Bar {
    name: String,
    current: AtomicUsize,
    total: AtomicUsize,
    message: Option<String>,
}

impl Bar {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            message: None,
            current: AtomicUsize::new(0),
            total: AtomicUsize::new(100),
        }
    }

    pub fn render(&self, prefix: usize, width: usize) -> String {
        let current = self.current.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);

        if current >= total {
            return format!(
                "{}{} {}",
                self.name,
                " ".repeat(prefix - self.name.len()),
                "✔".green().bold(),
            );
        }

        let left = (width as f64 * (current as f64 / total as f64)).floor() as usize;

        format!(
            "{}{} {}{}{}{}{} {}",
            self.name,
            " ".repeat(prefix - self.name.len()),
            "[".blue().bold(),
            if left > 0 {
                "=".repeat(left - 1)
            } else {
                String::new()
            }
            .bold(),
            if left > 0 { "❯" } else { "" }.bold(),
            if left > 0 { " " } else { "-" }.repeat(width - left),
            "]".blue().bold(),
            self.message.as_deref().unwrap_or_default()
        )
    }
}

pub struct ProgressBar {
    width: usize,
    bars: Vec<Bar>,
    previous_state: Option<Vec<String>>,
}

impl ProgressBar {
    pub fn with_multiple(names: Vec<impl ToString>) -> Self {
        Self {
            previous_state: None,
            width: 24,
            bars: names.into_iter().map(Bar::new).collect(),
        }
    }

    pub fn set_message(&mut self, idx: usize, message: impl ToString) {
        if idx < self.bars.len() {
            self.bars[idx].message = Some(message.to_string());
        }
    }

    pub fn set_progress(&mut self, idx: usize, current: usize, total: usize) {
        if idx < self.bars.len() {
            self.bars[idx].current.store(current, Ordering::Relaxed);
            self.bars[idx].total.store(total, Ordering::Relaxed);
        }
    }

    pub fn clear(&mut self) {
        if self.previous_state.is_none() {
            return;
        }

        self.previous_state = None;

        for _ in 0..self.bars.len() {
            print!("{MOVE_BEGIN}");
            print!("{MOVE_UP}");
            print!("{CLEAR_LINE}")
        }
    }

    pub fn draw(&mut self) {
        let prefix = self
            .bars
            .iter()
            .map(|bar| bar.name.len())
            .max()
            .unwrap_or(0);

        let mut new_state = vec![];

        match self.previous_state.as_ref() {
            Some(state) => {
                for (i, bar) in self.bars.iter().rev().enumerate() {
                    let current_line = &state[i];
                    let new_line = bar.render(prefix, self.width);

                    move_up(1);

                    if current_line == &new_line {
                        new_state.push(new_line);
                        continue;
                    }

                    let new = decode_chars(&new_line);
                    let current = decode_chars(current_line);

                    if new.len() < current.len() {
                        move_begin();
                        print!("{new_line}");
                        erase_to_end();
                    } else {
                        for (i, c) in new.iter().copied().enumerate() {
                            let current_char = current.get(i).copied().unwrap_or(' ');

                            if current_char == c {
                                continue;
                            }

                            move_begin();
                            move_right(i + 1);

                            if i <= current.len() {
                                delete_char();
                            }

                            print!("{c}");
                        }
                    }

                    new_state.push(new_line);
                }
            }
            None => {
                for bar in self.bars.iter() {
                    let new_line = bar.render(prefix, self.width);
                    println!("{new_line}");
                    new_state.push(new_line);
                }
            }
        }

        if self.previous_state.is_some() {
            move_down(self.bars.len());
            move_begin();
        }

        // Try to flush the output buffer
        let _ = stdout().flush();

        self.previous_state = Some(new_state);
    }
}
