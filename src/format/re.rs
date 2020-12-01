use regex::Regex;

pub fn regex_expect(regex: &str) -> Regex {
    match Regex::new(regex) {
        Ok(r) => r,
        Err(e) => panic!(format!("Invalid regex '{}': {}", regex, e))
    }
}
