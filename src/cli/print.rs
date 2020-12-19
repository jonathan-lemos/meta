use std::cmp::max;
use std::sync::{Mutex, MutexGuard};
use colored::Colorize;
use crate::linq::collectors::IntoVec;
use std::collections::VecDeque;

struct PrintingContext {
    indent_level: usize,
    x_index: usize,
}

static PRINTING_CTX: Mutex<PrintingContext> = Mutex::new(PrintingContext { indent_level: 0, x_index: 0 });
static WIDTH: Option<usize> = term_size::dimensions_stdout().map(|x| x.1);

pub fn width() -> Option<usize> {
    WIDTH
}

fn exceeds_width(n: usize) -> bool {
    WIDTH.map(|e| n >= e).unwrap_or(false)
}

pub fn print() -> MutexGuard<'static, PrintingContext> {
    PRINTING_CTX.lock().expect("PrintingContext mutex is poisoned. This should never happen.")
}

impl PrintingContext {
    pub fn indent_level(&self) -> usize {
        self.indent_level
    }

    pub fn indent(&mut self, count: usize) {
        if self.x_index == 0 {
            if exceeds_width(self.x_index + count) {
                self.newline();
            }

            self.space(count);
            self.x_index += count;
        }
        self.indent_level += count;
    }

    pub fn unindent(&mut self, count: usize) {
        self.indent_level = max(0, self.indent_level - count);
    }

    pub fn set_indent(&mut self, count: usize) {
        self.indent_level = count;
    }

    pub fn newline(&mut self) {
        println!();
        self.x_index = 0;

        if !exceeds_width(self.indent_level) {
            print!("{}", " ".repeat(self.indent_level));
            self.x_index += self.indent_level
        }
    }

    pub fn line(&mut self, s: &str) {
        self.str(s);
        self.newline();
    }

    pub fn space(&mut self, count: usize) {
        if !exceeds_width(self.indent_level + count) {
            print!("{}", " ".repeat(count));
            self.x_index += count;
        }
        else {
            self.newline()
        }
    }

    pub fn str(&mut self, s: &str) {
        for chunk in s.split_whitespace() {
            let cl = chunk.chars().count();

            if exceeds_width(self.x_index + cl) {
                self.newline()
            }

            print!("{}", chunk);
            self.x_index += cl;
        }
    }
}

struct LoggingContext();
static LOGGING_CTX: Mutex<LoggingContext> = Mutex::new(LoggingContext());

pub fn log() -> MutexGuard<'static, LoggingContext> {
    LOGGING_CTX.lock().expect("LoggingContext mutex is poisoned. This should never happen.")
}

pub trait Logger {
    fn info(&mut self, s: &str);
    fn debug(&mut self, s: &str);
    fn warn(&mut self, s: &str);
    fn error(&mut self, s: &str);
    fn cmdline(&mut self, cmdline: &str, index: usize, len: usize);
}

impl Logger for LoggingContext {
    fn info(&mut self, s: &str) {
        eprintln!("{} {}", "[info]".bold().green(), s);
    }

    fn debug(&mut self, s: &str) {
        eprintln!("{} {}", "[debug]".bold().cyan(), s);
    }

    fn warn(&mut self, s: &str) {
        eprintln!("{} {}", "[warn]".bold().yellow(), s);
    }

    fn error(&mut self, s: &str) {
        eprintln!("{} {}", "[error]".bold().red(), s);
    }

    fn cmdline(&mut self, cmdline: &str, mut index: usize, len: usize) {
        let mut char_deque = cmdline.chars().collect::<VecDeque<_>>();
        let end = || index + len;

        let print_top = || for (i, c) in char_deque.into_iter().enumerate() {
            if i < index {
                eprint!("{}", c);
            }
            else {
                eprint!("{}", c.to_string().bold().red());
            }
            eprintln!();
        };

        let print_bottom = || for (i, _) in char_deque.into_iter().enumerate() {
            if i < index {
                eprint!(" ");
            }
            else {
                eprint!("^");
            }
            eprintln!();
        };

        eprintln!();
        if let Some(w) = WIDTH {
            if len < w {
                while char_deque.len() > w {
                    if end() > char_deque.len() {
                        char_deque.pop_front();
                        index -= 1;
                    }
                    else {
                        char_deque.pop_back();
                    }
                }
            }
            else {
                print_top();
                return;
            }
        }

        print_top();
        print_bottom();
    }
}
