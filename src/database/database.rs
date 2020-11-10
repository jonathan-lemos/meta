use crate::database::path::parent_dir;
use crate::functional::lazy::Lazy;
use std::iter::FromIterator;

pub struct Directory {
    id: u32,
    path: String,
}

impl Directory {
    pub fn parent_path(&self) -> Option<&str> {
        parent_dir(&self.path)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct File<'a> {
    id: u32,
    filename: String,
    hash: [u8; 32],
    directory: Lazy<'a, Option<Directory>>
}

impl<'a> File<'a> {
    pub fn parent_path(&self) -> Option<&str> {
        parent_dir(&self.filename)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }

    pub fn directory(&self) -> Option<&Directory> {
        match self.directory.get() {
            Some(s) => Some(s),
            None => None
        }
    }
}

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;
    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, E>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, E>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>, E>;
    fn directory_files<B: FromIterator<&'a File<'a>>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_files_with_key<B: FromIterator<&'a File<'a>>>(&self, key: &str) -> Result<B, E>;
    fn directory_files_with_key_and_value<B: FromIterator<File<'a>>>(&self, key: &str, value: &str) -> Result<B, E>;
    fn directory_metadata<B: FromIterator<(String, String)>>(&self, d: &Directory) -> Result<B, E>;
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
