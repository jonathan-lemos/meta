use crate::filesystem::xattr::XattrFunctions;
use std::path::Path;
use std::iter::{FromIterator, Map};
use std::io::{Error, Result, ErrorKind};
use xattr::XAttrs;
use std::ffi::OsString;

pub struct UnixXattr();

type Iter = Map<XAttrs, fn(OsString) -> core::result::Result<String, Error>>;

impl XattrFunctions<Iter> for UnixXattr {
    fn list_keys(p: &Path) -> Result<Iter> {
        Ok(xattr::list(p)?.map(|x| {
            x.into_string().map_err(
                |e| Error::new(ErrorKind::InvalidData, format!("The OS-string '{:?}' cannot be converted to valid UTF-8. What OS are you using?", e))
            )
        }))
    }

    fn get(p: &Path, key: &str) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn set(p: &Path, key: &str, value: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn remove(p: &Path, key: &str) -> Result<()> {
        unimplemented!()
    }
}

