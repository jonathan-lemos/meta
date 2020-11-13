use std::iter::FromIterator;
use crate::database::models::{Directory, File};

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;
    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, E>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, E>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>, E> {
        Ok(self.directory_files::<Vec<File>>(d)?
            .iter()
            .find(|f| f.filename == filename)
            .map(|x| *x))
    }
    fn directory_files<B: FromIterator<File>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_files_with_key<B: FromIterator<File>>(&self, d: &Directory, key: &str) -> Result<B, E>;
    fn directory_files_with_key_and_value<B: FromIterator<File>>(&self, d: &Directory, key: &str, value: &str) -> Result<B, E>;
    fn directory_metadata<B: FromIterator<(String, String)>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_metadata_get(&self, d: &Directory, key: &str) -> Result<Option<String>, E>;
    fn directory_metadata_set(&self, d: &Directory, key: &str, value: Option<&str>) -> Result<Option<String>, E>;

    fn get_directory(&self, path: &str) -> Result<Option<Directory>, E>;
    fn get_directories<B: FromIterator<Directory>>(&self) -> Result<B, E>;
    fn get_directories_with_key<B: FromIterator<Directory>>(&self, key: &str) -> Result<B, E>;
    fn get_directories_with_key_and_value<B: FromIterator<Directory>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn get_file(&self, path: &str) -> Result<Option<(File, Directory)>, E>;
    fn get_files<B: FromIterator<File>>(&self) -> Result<B, E>;
    fn get_files_with_key<B: FromIterator<File>>(&self, key: &str) -> Result<B, E>;
    fn get_files_with_key_and_value<B: FromIterator<File>>(&self, key: &str, value: &str) -> Result<B, E>;

    fn add_directory(&self, path: &str) -> Result<(Directory, bool), E>;
    fn add_directories<'b, B: FromIterator<(Directory, bool)>, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<B, E> {
        let res: Vec<(Directory, bool)> = Vec::new();

        for path in paths {
            res.push(self.add_directory(path)?);
        }

        Ok(res.iter().map(|x| *x).collect())
    }
    fn add_file(&self, path: &str, hash: &str) -> Result<(File, bool), E>;
    fn add_files<'b, B: FromIterator<(File, bool)>, I: Iterator<Item=(&'b str, &'b str)>>(&self, paths: I) -> Result<B, E> {
        let res: Vec<(File, bool)> = Vec::new();

        for (path, hash) in paths {
            res.push(self.add_file(path, hash)?);
        }

        Ok(res.iter().map(|x| *x).collect())
    }
}
