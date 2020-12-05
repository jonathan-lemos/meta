use fancy_regex::Regex;
use std::collections::HashMap;
use crate::collections::mrucache::MRUCache;
use std::sync::Mutex;
use std::borrow::BorrowMut;

const RE_CACHE_LEN: usize = 100;

static RE_CACHE: Mutex<MRUCache<String, Regex>> = Mutex::new(MRUCache::new(RE_CACHE_LEN));

pub fn regex_expect(regex: &str) -> &'static Regex {
    let mut lck = RE_CACHE.lock().expect("Regex cache lock is poisoned. This should never happen.");
    let cache = lck.borrow_mut();

    match cache.get(&regex.to_owned()) {
        Some(r) => return r,
        None => {}
    }

    match Regex::new(regex) {
        Ok(r) => cache.insert(regex.to_owned(), r),
        Err(e) => panic!(format!("Invalid regex '{}': {}", regex, e))
    }
}
