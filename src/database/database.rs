use std::iter::FromIterator;
use crate::functional::lazy::Lazy;

pub struct Directory {
    pub(crate) id: u32,
    pub path: String,
}

pub struct File<'a> {
    pub(crate) id: u32,
    pub filename: String,
    pub hash: [u8; 32],
    directory: Lazy<'a, Option<Directory>>
}

impl<'a> File<'a> {
    pub fn containing_dir(&self) -> Option<&str> {
        
    }
}

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;
    fn file_metadata<B: FromIterator<(&'a str, &'a str)>>(&self, f: &File) -> Result<B, E>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, E>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>, E>;
    fn directory_files<B: FromIterator<&'a File<'a>>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_files_with_key<B: FromIterator<&'a File<'a>>>(&self, key: &str) -> Result<B, E>;
    fn directory_files_with_key_and_value<B: FromIterator<File<'a>>>(&self, key: &str, value: &str) -> Result<B, E>;
    fn directory_metadata<B: FromIterator<(&'a str, &'a str)>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_metadata_get(&self, d: &Directory, key: &str) -> Result<Option<String>, E>;
    fn directory_metadata_set(&self, d: &Directory, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn get_directory(&self, path: &str) -> Option<Directory>;
    fn get_directories<B: FromIterator<&'a Directory>>(&self) -> Result<B, E>;
    fn get_directories_with_key<B: FromIterator<&'a Directory>>(&self, key: &str) -> Result<B, E>;
    fn get_directories_with_key_and_value<B: FromIterator<&'a Directory>>(&self, key: &str, value: &str) -> Result<B, E>;
    fn get_files_with_key<B: FromIterator<&'a File<'a>>>(&self, key: &str) -> Result<B, E>;
    fn get_files_with_key_and_value<B: FromIterator<&'a File<'a>>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn add_directory(&self, path: &str) -> Result<(), E>;
    fn add_directory_rec(&self, path: &str) -> Result<(), E>;
    fn add_file(&self, path: &str) -> Result<(), E>;
}
