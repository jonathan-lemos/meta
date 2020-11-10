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