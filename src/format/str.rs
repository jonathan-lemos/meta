use std::iter::Fuse;
use std::str::Chars;
use std::ops::Mul;

pub struct CharIterator<'a> {
    iter: Fuse<Chars<'a>>,
    str: &'a str,
    count_chars: usize,
    count_bytes: usize
}

impl<'a> CharIterator<'a> {
    pub fn new(str: &'a str) -> Self {
        CharIterator {
            iter: str.chars().fuse(),
            str,
            count_chars: 0,
            count_bytes: 0
        }
    }

    pub fn position_chars(&self) -> usize {
        self.count_chars
    }

    pub fn position_bytes(&self) -> usize {
        self.count_bytes
    }
}

pub trait ToCharIterator {
    fn char_iterator(&self) -> CharIterator<'_>;
}

impl ToCharIterator for &str {
    fn char_iterator(&self) -> CharIterator<'_> {
        CharIterator::new(self)
    }
}

impl<'a> Iterator for CharIterator<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(c) => {
                self.count_chars += 1;
                self.count_bytes += c.len_utf8();
                Some(c)
            },
            None => None
        }
    }
}

pub trait StrExtensions {
    fn slice_until<F: FnMut(char) -> bool>(&self, until: F) -> &str;

    fn slice_from<F: FnMut(char) -> bool>(&self, until: F) -> &str;

    fn slice_offset_bytes(&self, inner: &str) -> Option<usize>;

    fn slice_offset_chars(&self, inner: &str) -> Option<usize>;

    fn col_begin_end(&self, begin: usize, end: usize) -> String;

    fn col_begin_end_indent(&self, begin: usize, end: Option<usize>) -> String;
}

impl Mul<usize> for &str {
    type Output = String;

    fn mul(self, rhs: usize) -> Self::Output {
        let mut ret = self.to_owned();
        for _ in 0..rhs {
            ret += self.clone();
        }
        ret
    }
}

impl StrExtensions for &str {
    fn slice_until<F: FnMut(char) -> bool>(&self, until: F) -> &str {
        let iter = self.char_iterator();

        for c in iter {
            if until(c) {
                return &self[..iter.position_bytes()]
            }
        }

        self
    }

    fn slice_from<F: FnMut(char) -> bool>(&self, until: F) -> &str {
        let iter = self.char_iterator();

        for c in iter {
            if until(c) {
                return &self[iter.position_bytes()..]
            }
        }

        self
    }

    fn slice_offset_bytes(&self, inner: &str) -> Option<usize> {
        let o1 = self.as_ptr() as usize;
        let o2 = inner.as_ptr() as usize;

        if o1 > o2 || o2 > o1.wrapping_add(self.len()) {
            None
        }
        else {
            Some(o2 - o1)
        }
    }

    fn slice_offset_chars(&self, inner: &str) -> Option<usize> {
        let ob = match self.slice_offset_bytes(inner) {
            Some(s) => s,
            None => return None
        };

        if ob == 0 {
            return Some(0)
        }

        let iter = self.char_iterator();

        for _ in iter {
            if iter.position_bytes() == ob {
                return Some(iter.position_chars())
            }
            if iter.position_bytes() > ob {
                return None
            }
        }

        None
    }

    fn col_begin_end(&self, begin: usize, end: Option<usize>) -> String {
        let spacer = " " * begin;
        let mut ret = String::new();
        let mut index = begin;

        if let Some(e) = end {
            // 'if let Some(e) = end && begin >= e' is "experimental"
            if begin >= e {
                return self.to_string();
            }
        }

        // todo?: hyphenator
        for line in self.split("\n") {
            if let Some(e) = end {
                for word in line.split_whitespace() {
                    if word.len() + index > e {
                        match index {
                            x if x == begin => {
                                ret += word;
                                ret += "\n";
                            }
                            _ => {
                                ret += "\n";
                                ret += &spacer;
                                ret += word;
                            }
                        }
                    }
                    else {
                        ret += word;
                    }
                }
            }
            else {
                ret += line;
            }

            ret += "\n";
            ret += &spacer;
        }
        ret.trim().to_owned()
    }

    fn col_begin_end_indent(&self, begin: usize, end: Option<usize>) -> String {
        let spacer = " " * begin;

        (spacer + self).col_begin_end(begin, end)
    }
}

