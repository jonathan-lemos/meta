use std::ops::{Add, AddAssign, Div, DivAssign};
use std::cmp::max;

#[derive(Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
pub struct Path {
    pat: String
}

impl Path {
    pub fn new(s: &str) -> Self {
        let s = s.trim_end_matches("/");
        let s = s.trim_start_matches("//");

        let st = if !s.starts_with("/") {
            let mut tmp = "/".to_owned();
            tmp += s;
            tmp
        }
        else {
            s.to_owned()
        };

        Path { pat: st }
    }

    pub fn str(&self) -> &str {
        &self.pat
    }

    pub fn filename(&self) -> &str {
        &self.pat[self.pat.trim_end_matches(|c| c != '/').trim_end_matches('/').len()..]
    }

    pub fn parent_str(s: &str) -> &str {
        let r = s.trim_end_matches(|c| c != '/').trim_end_matches('/');

        if r.len() > 0 {
            r
        }
        else {
            "/"
        }
    }

    pub fn parent(&self) -> &str {
        Self::parent_str(&self.pat)
    }

    pub fn pop(&mut self) -> String {
        let f = self.filename().to_owned();
        self.pat.truncate(self.pat.trim_end_matches(|c| c != '/').trim_end_matches('/').len());
        f
    }
}

impl Add<&str> for Path {
    type Output = Path;

    fn add(self, rhs: &str) -> Self {
        let mut n = self.pat + rhs;
        n.truncate(n.trim_end_matches("/").len());

        Self {
            pat: n
        }
    }
}

impl AddAssign<&str> for Path {
    fn add_assign(&mut self, rhs: &str) {
        self.pat += rhs;
        self.pat.truncate(self.pat.trim_end_matches("/").len());
    }
}

impl Div<&str> for Path {
    type Output = Path;

    fn div(self, rhs: &str) -> Self {
        let mut n = self.pat + "/" + rhs;
        n.truncate(n.trim_end_matches("/").len());

        Self {
            pat: n
        }
    }
}

impl DivAssign<&str> for Path {
    fn div_assign(&mut self, rhs: &str) {
        self.pat += "/";
        self.pat += rhs;
        self.pat.truncate(self.pat.trim_end_matches("/").len());
    }
}

mod PathTests {
    use super::Path;

    #[test]
    fn test_filename() {
        let cases = [
            ("/foo", "foo"),
            ("/foo/bar", "bar"),
            ("/", "")
        ];

        let paths = cases.iter().map(|x| Path::new(x.0)).collect::<Vec<Path>>();

        for (path, exp) in paths.iter().zip(cases.iter().map(|x| x.1)) {
            assert_eq!(path.filename(), exp);
        }
    }

    #[test]
    fn test_pop() {
        let cases = [
            ("/foo", "/"),
            ("/foo/bar", "/foo"),
            ("/", "/")
        ];

        let paths = cases.iter().map(|x| Path::new(x.0)).collect::<Vec<Path>>();

        for (path, exp) in paths.iter().zip(cases.iter().map(|x| x.1)) {
            assert_eq!(path.parent(), exp);
        }
    }
}
