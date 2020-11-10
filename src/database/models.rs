use super::schema::*;
use super::path::parent_dir;

#[derive(Queryable)]
pub struct Directory {
    pub id: u32,
    pub path: String,
}

#[derive(Insertable)]
#[table_name="Directories"]
pub struct NewDirectory<'a> {
    pub path: &'a str
}

impl Directory {
    pub fn parent_path(&self) -> Option<&str> {
        parent_dir(&self.path)
    }
}

#[derive(Queryable)]
pub struct File {
    pub id: u32,
    pub filename: String,
    pub hash: String,
}

#[derive(Insertable)]
#[table_name = "Files"]
pub struct NewFile<'a> {
    pub filename: &'a str,
    pub hash: &'a str 
}

impl File {
    pub fn parent_path(&self) -> Option<&str> {
        parent_dir(&self.filename)
    }
}

#[derive(Queryable, PartialEq, Associations(foreign_key="fileId"), Debug)]
#[table_name = "FileMetadata"]
#[belongs_to(File, foreign_key="fileId")]
pub struct FileMetadata {
    pub id: u32,
    pub fileId: u32,
    pub key: String,
    pub value: String
}

#[derive(Insertable, Debug)]
pub struct NewFileMetadata {
    pub id: u32,
    pub fileId: u32,
    pub key: String,
    pub value: String
}