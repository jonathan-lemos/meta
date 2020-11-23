use std::iter::FromIterator;

use crate::database::models::{Directory, File};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

impl Entry {
    pub fn iter_split<I: Iterator<Item=Entry>>(iter: I) -> (Vec<File>, Vec<Directory>) {
        let mut f = Vec::<File>::new();
        let mut d = Vec::<Directory>::new();

        for entry in iter {
            match entry {
                Entry::File(ff) => f.push(ff),
                Entry::Directory(dd) => d.push(dd)
            }
        }

        (f, d)
    }
}

pub trait Database<'a, E> {
    fn file_directory(&self, f: &File) -> Result<Directory, E>;

    fn entry_metadata<B: FromIterator<(String, String)>>(&self, entry: &Entry) -> Result<B, E>;
    fn entry_metadata_get(&self, entry: &Entry, key: &str) -> Result<Option<String>, E>;
    fn entry_metadata_set(&self, entry: &Entry, key: &str, value: Option<&str>) -> Result<Option<String>, E>;
    fn entry_metadata_clear(&self, entry: &Entry) -> Result<usize, E>;

    fn entries_metadata<'b, B: FromIterator<(Entry, Vec<(String, String)>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<B, E>;
    fn entries_metadata_get<'b, B: FromIterator<(Entry, String)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, key: &str) -> Result<B, E>;
    fn entries_metadata_set<'b, B: FromIterator<(Entry, Option<String>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, key: &str, value: Option<&str>) -> Result<B, E>;
    fn entries_metadata_clear<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, E>;

    fn directory_entry(&self, d: &Directory, filename: &str) -> Result<Option<Entry>, E>;
    fn directory_entries<B: FromIterator<Entry>>(&self, d: &Directory) -> Result<B, E>;
    fn directory_entries_with_key<'b, B: FromIterator<Entry>>(&self, d: &Directory, key: &str) -> Result<B, E>;
    fn directory_entries_with_key_and_value<'b, B: FromIterator<Entry>>(&self, d: &Directory, key: &str, value: &str) -> Result<B, E>;

    fn get_entry(&self, path: &str) -> Result<Option<Entry>, E>;
    fn get_entries<'b, B: FromIterator<Entry>, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<B, E>;

    fn add_directory(&self, path: &str) -> Result<(Directory, bool), E>;
    fn add_directories<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<usize, E>;
    fn add_file(&self, path: &str, hash: &[u8]) -> Result<(File, bool), E>;
    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, E>;

    fn remove_entry(&self, entry: &Entry) -> Result<bool, E>;
    fn remove_entries<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, E>;
}
