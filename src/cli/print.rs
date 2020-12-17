use std::cmp::max;
use std::sync::{Mutex, MutexGuard};

struct PrintingContext {
    indent_level: usize,
    x_index: usize,
}

static PRINTING_CTX: Mutex<PrintingContext> = Mutex::new(PrintingContext { indent_level: 0, x_index: 0 });
static WIDTH: Option<usize> = term_size::dimensions_stdout().map(|x| x.1);

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