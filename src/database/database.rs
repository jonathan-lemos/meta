use std::iter::FromIterator;
use crate::database::models::{Directory, File};

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;
    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, E>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, E>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>, E>;
    fn directory_files<B: FromIterator<&'a File>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_files_with_key<B: FromIterator<&'a File>>(&self, d: &Directory, key: &str) -> Result<B, E>;
    fn directory_files_with_key_and_value<B: FromIterator<&'a File>>(&self, d: &Directory, key: &str, value: &str) -> Result<B, E>;
    fn directory_metadata<B: FromIterator<(String, String)>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_metadata_get(&self, d: &Directory, key: &str) -> Result<Option<String>, E>;
    fn directory_metadata_set(&self, d: &Directory, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn get_directory(&self, path: &str) -> Result<Option<Directory>, E>;
    fn get_directories<B: FromIterator<&'a Directory>>(&self) -> Result<B, E>;
    fn get_directories_with_key<B: FromIterator<&'a Directory>>(&self, key: &str) -> Result<B, E>;
    fn get_directories_with_key_and_value<B: FromIterator<&'a Directory>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn get_file(&self, path: &str) -> Result<Option<(File, Directory)>, E>;
    fn get_files<B: FromIterator<&'a File>>(&self) -> Result<B, E>;
    fn get_files_with_key<B: FromIterator<&'a File>>(&self, key: &str) -> Result<B, E>;
    fn get_files_with_key_and_value<B: FromIterator<&'a File>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn add_directory(&self, path: &str) -> Result<(), E>;
    fn add_directory_rec(&self, path: &str) -> Result<(), E>;
    fn add_file(&self, path: &str) -> Result<(), E>;
}
