use std::iter::Fuse;
use std::str::Chars;

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
            return 0
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
}

