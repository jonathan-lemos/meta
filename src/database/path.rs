pub fn parent_dir(path: &str) -> Option<&str> {
    let count = 0;
    let it = path.chars().rev();

    match it.next() {
        Some(c) => {
            count += 1;
        }
        None => return None
    }

    for c in it {
        if c == '/' {
            return Some(&path[..path.len() - count]);
        }
        count += 1
    }

    None
}

#[test]
fn test_parent_dir() {
    assert_eq!(parent_dir("/foo/bar"), Some("/foo"));
    assert_eq!(parent_dir("/foo/bar/"), Some("/foo"));
    assert_eq!(parent_dir("/f/b"), Some("/f"));
    assert_eq!(parent_dir("/f/b/"), Some("/f"));
    assert_eq!(parent_dir("/foo/"), Some("/"));
    assert_eq!(parent_dir("/foo"), Some("/"));
    assert_eq!(parent_dir("/f/"), Some("/"));
    assert_eq!(parent_dir("/f"), Some("/"));
    assert_eq!(parent_dir("/"), None);
    assert_eq!(parent_dir(""), None);
}

pub fn filename(path: &str) -> &str {
    let count = 0;
    let it = path.chars().rev();

    match it.next() {
        Some(c) => {
            count += 1;
        }
        None => return ""
    }

    for c in it {
        if c == '/' {
            return &path[path.len() - count..];
        }
        count += 1
    }

    path
}

#[test]
fn test_filename() {
    assert_eq!(filename("/foo/bar"), "bar");
    assert_eq!(filename("/foo/bar/"), "");
    assert_eq!(filename("/f/b"), "b");
    assert_eq!(filename("/f/b/"), "");
    assert_eq!(filename("/foo/"), "");
    assert_eq!(filename("/foo"), "foo");
    assert_eq!(filename("/f/"), "");
    assert_eq!(filename("/f"), "f");
    assert_eq!(filename("/"), "");
    assert_eq!(filename(""), "");
}

pub fn preprocess(p: &str) -> &str {
    p.trim_end_matches("/");
    p.trim_start_matches("//");
    if !p.starts_with("/") {
        p = &("/".to_owned() + p);
    }
    p
}