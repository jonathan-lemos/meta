use super::schema::*;

/*
    notes to future self:
        the foreign key column must be named {entity.lower()}_id
        the table name cannot be the same as the struct name
*/

#[derive(Identifiable, Queryable, Clone, PartialEq, Eq, Debug)]
#[table_name = "Directories"]
pub struct Directory {
    pub id: i32,
    pub path: String,
}

#[derive(Insertable)]
#[table_name = "Directories"]
pub struct NewDirectory<'a> {
    pub path: &'a str
}

impl Directory {}

#[derive(Identifiable, Queryable, PartialEq, Eq, Associations, Debug, Clone)]
#[belongs_to(Directory)]
#[table_name = "Files"]
pub struct File {
    pub id: i32,
    pub directory_id: i32,
    pub filename: String,
    pub hash: Vec<u8>,
}

#[derive(Insertable, PartialEq, Eq, Associations, Debug)]
#[belongs_to(Directory)]
#[table_name = "Files"]
pub struct NewFile<'a> {
    pub directory_id: i32,
    pub filename: &'a str,
    pub hash: &'a [u8],
}

#[derive(Identifiable, Queryable, PartialEq, Eq, Associations, Debug, Clone)]
#[belongs_to(File)]
#[table_name = "FileMetadata"]
pub struct FileKeyValuePair {
    pub id: i32,
    pub file_id: i32,
    pub key: String,
    pub value: String,
}

#[derive(Insertable, Associations, PartialEq, Eq, Debug)]
#[belongs_to(File)]
#[table_name = "FileMetadata"]
pub struct NewFileKeyValuePair<'a> {
    pub file_id: i32,
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Identifiable, Queryable, PartialEq, Eq, Associations, Debug, Clone)]
#[belongs_to(Directory)]
#[table_name = "DirectoryMetadata"]
pub struct DirectoryKeyValuePair {
    pub id: i32,
    pub directory_id: i32,
    pub key: String,
    pub value: String,
}

#[derive(Insertable, Associations, PartialEq, Eq, Debug)]
#[belongs_to(Directory)]
#[table_name = "DirectoryMetadata"]
pub struct NewDirectoryKeyValuePair<'a> {
    pub directory_id: i32,
    pub key: &'a str,
    pub value: &'a str,
}