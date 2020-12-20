use edit_distance::edit_distance;
use crate::linq::collectors::IntoVec;

pub fn typos<'a, I: Iterator<Item=&'a str>>(expected: &str, actual: I) -> Vec<(&'a str, usize)> {
    let mut v = actual.map(|x| (x, edit_distance(expected, x))).into_vec();
    v.sort_by_key(|x| -x.1);
    v
}

pub fn typos_threshold<'a, I: Iterator<Item=&'a str>>(expected: &str, actual: I, threshold: f64, max_count: usize) -> Vec<(&'a str, usize)> {
    let nchars = f64::round(expected.chars().count() * threshold) as usize;
    let mut v = typos(expected, actual);
    v.truncate(max_count);
    v.into_iter().filter(|x| x.1 >= nchars).into_vec()
}