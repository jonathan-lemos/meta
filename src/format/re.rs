use regex::Regex;
use std::collections::HashMap;

const RE_CACHE_LEN: usize = 100;
static RE_CACHE: HashMap<String, Regex> = HashMap::<String, Regex>::new();

pub fn regex_expect(regex: &str) -> &'static Regex {
    match RE_CACHE.get(regex) {
        Some(r) => return r,
        None => {}
    }

    match Regex::new(regex) {
        Ok(r) => {

        },
        Err(e) => panic!(format!("Invalid regex '{}': {}", regex, e))
    }
}
