use std::io::Result;
use std::path::Path;
use std::iter::FromIterator;

pub trait XattrFunctions<I: Iterator<Item=String>> {
    fn list_keys(p: &Path) -> Result<I>;
    fn get(p: &Path, key: &str) -> Result<Option<Vec<u8>>>;
    fn set(p: &Path, key: &str, value: &[u8]) -> Result<()>;
    fn remove(p: &Path, key: &str) -> Result<()>;
}

#[cfg(target_family = "unix")]
pub type Xattr = crate::os::unix::xattr::UnixXattr;
