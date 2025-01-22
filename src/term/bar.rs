use std::{sync::RwLock, time::Duration};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Bar {
    inner: ProgressBar,
    message: RwLock<String>,
}

impl Default for Bar {
    fn default() -> Self {
        let bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
        bar.enable_steady_tick(Duration::from_millis(50));

        Self {
            message: RwLock::new(String::new()),
            inner: bar,
        }
    }
}

impl Bar {
    pub fn with_message(msg: impl ToString) -> Self {
        let inner = Self::default();

        *inner.message.write().unwrap() = msg.to_string();
        inner.redraw();
        inner
    }

    pub fn set_message(&self, msg: impl ToString) {
        *self.message.write().unwrap() = msg.to_string();
        self.redraw();
    }

    pub fn update(&self, current: usize, total: usize) {
        self.inner.set_position(current as u64);
        self.inner.set_length(total as u64);
        self.redraw();
    }

    pub fn writeln(&self, msg: impl ToString) {
        self.inner.println(msg.to_string());
    }

    fn redraw(&self) {
        let current = self.inner.position();
        let total = self.inner.length().unwrap_or(0);

        if total > 0 {
            let max = 20;
            let len = ((current as f64 / total as f64) * max as f64) as usize;
            let arrows = format!("{}>", "=".repeat(len));
            let ws = " ".repeat(max - len);

            self.inner.set_message(format!(
                "{}{}{} {}",
                "[".blue().bold(),
                format!("{arrows}{ws}").bright_black(),
                "]".blue().bold(),
                self.message.read().unwrap(),
            ));
        } else {
            self.inner.set_message(self.message.read().unwrap().clone());
        }
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        self.inner.finish_and_clear();
    }
}
