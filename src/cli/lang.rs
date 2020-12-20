fn combine<'a, I: Iterator<Item=&'a str>>(mut iter: I, word: &str) -> String {
    let mut cur = iter.next();
    let mut next = iter.next();
    let mut s = "".to_owned();

    while let Some(n) = next {
        s += cur.unwrap();
        s += ", ";
        cur = next
        next = iter.next();
    }

    if let Some(n) = cur {
        if s == "" {
            s = n.to_owned();
        }
        else {
            if s.contains(", ") {
                s += ", ";
            }
            s += word;
            s += " ";
            s += n;
        }
    }

    s
}

pub fn and<'a, I: Iterator<Item=&'a str>>(mut iter: I) -> String {
    combine(iter, "and")
}

pub fn or<'a, I: Iterator<Item=&'a str>>(mut iter: I) -> String {
    combine(iter, "or")
}