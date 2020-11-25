use std::env::{current_dir, set_current_dir};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::fs::{DirEntry, ReadDir, read_dir};
use std::thread;
use crate::linq::collectors::IntoVec;

pub const DB_NAME: &'static str = ".meta.db";

pub fn reposition_to_db() -> Result<Option<String>> {
    let mut dir = current_dir()?;
    let dir_initial = dir.clone();

    loop {
        dir = match current_dir() {
            Ok(d) => d,
            Err(e) => {
                if let ErrorKind::PermissionDenied = e.kind() {
                    match dir.parent() {
                        Some(s) => set_current_dir(s)?,
                        None => {
                            set_current_dir(dir_initial)?;
                            return Ok(None);
                        }
                    };
                    continue;
                } else {
                    set_current_dir(dir_initial)?;
                    return Err(e);
                }
            }
        };

        let target = dir.join(DB_NAME).to_owned();

        if target.is_file() {
            return match target.to_str() {
                Some(s) => Ok(Some(s.to_owned())),
                None => {
                    set_current_dir(dir_initial)?;
                    return Err(Error::new(ErrorKind::InvalidData, format!("Found a database file at {:#?}, but it could not be converted to a UTF-8 string. What OS are you using?", dir)));
                }
            };
        }

        match dir.parent() {
            Some(s) => set_current_dir(s)?,
            None => {
                set_current_dir(dir_initial)?;
                return Ok(None);
            }
        };
    }
}
