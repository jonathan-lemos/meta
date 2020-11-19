use std::collections::BTreeMap;
use crate::functional::either::Either;
use std::iter::FromIterator;
use crate::database::models::{Directory, File};

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, E>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, E>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn files_metadata<'b, B: FromIterator<(File, BTreeMap<String, String>)>, I: Iterator<Item=&'b File>>(&self, files: I) -> Result<B, E>;
    fn files_metadata_get<'b, B: FromIterator<(File, Option<String>)>, I: Iterator<Item=&'b File>>(&self, files: I, key: &str) -> Result<B, E>;
    fn files_metadata_set<'b, I: Iterator<Item=&'b File>>(&self, files: I, key: &str, value: &str) -> Result<usize, E>;
    fn files_with_key<'b, B: FromIterator<File>, I: Iterator<Item=&'b File>>(&self, files: I, key: &str) -> Result<B, E>;
    fn files_with_key_and_value<'b, B: FromIterator<File>, I: Iterator<Item=&'b File>>(&self, files: I, key: &str, value: &str) -> Result<B, E>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>, E>;
    fn directory_subdir(&self, d: &Directory, filename: &str) -> Result<Option<Directory>, E>;
    fn directory_entries<F: FromIterator<File>, D: FromIterator<Directory>>(&self, d: &Directory) -> Result<(F, D), E>;

    fn directory_metadata<B: FromIterator<(Directory, BTreeMap<String, String>)>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_metadata_get(&self, d: &Directory, key: &str) -> Result<Option<String>, E>;
    fn directory_metadata_set(&self, d: &Directory, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn directories_metadata<'b, B: FromIterator<(Directory, Vec<String>)>, I: Iterator<Item=&'b Directory>>(&self, files: I) -> Result<B, E>;
    fn directories_metadata_get<'b, B: FromIterator<(Directory, Option<String>)>, I: Iterator<Item=&'b Directory>>(&self, files: I, key: &str) -> Result<B, E>;
    fn directories_metadata_set<'b, I: Iterator<Item=&'b Directory>>(&self, files: I, key: &str, value: &str) -> Result<usize, E>;
    fn directories_with_key<B: FromIterator<File>>(&self, d: &Directory, key: &str) -> Result<B, E>;
    fn directories_with_key_and_value<B: FromIterator<File>>(&self, d: &Directory, key: &str, value: &str) -> Result<B, E>;

    fn get_directory(&self, path: &str) -> Result<Option<Directory>, E>;
    fn get_directories<B: FromIterator<Directory>>(&self) -> Result<B, E>;
    fn get_directories_with_key<B: FromIterator<Directory>>(&self, key: &str) -> Result<B, E>;
    fn get_directories_with_key_and_value<B: FromIterator<Directory>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn get_file(&self, path: &str) -> Result<Option<File>, E>;
    fn get_files<B: FromIterator<File>>(&self) -> Result<B, E>;
    fn get_files_with_key<B: FromIterator<File>>(&self, key: &str) -> Result<B, E>;
    fn get_files_with_key_and_value<B: FromIterator<File>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn add_directory(&self, path: &str) -> Result<(Directory, bool), E>;
    fn add_directories<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<usize, E>;
    fn add_file(&self, path: &str, hash: &[u8]) -> Result<(File, bool), E>;
    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, E>;
}
