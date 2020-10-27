use rusqlite::{Connection, params, Result};
use std::path::Path;
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::ops::{Index, IndexMut};
use std::hash::Hash;

enum Lazy<T> {
    NotLoaded,
    Loaded(Box<T>)
}

enum PartialMap<K: Hash + Eq, V> {
    Partial(Box<HashMap<K, V>>),
    Full(Box<HashMap<K, V>>)
}

impl<K: Hash + Eq, V> PartialMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        match self {
            PartialMap::Partial(p) => p.get(key),
            PartialMap::Full(p) => p.get(key)
        }
    }

    pub fn set(&mut self, key: &K, value: &V) {
        match self {
            PartialMap::Empty => {
                *self = PartialMap::Partial(Box::new(HashMap::new()));
                self.set(key, value)
            },
            PartialMap::Partial(p) => {
                p.insert(*key, *value);
            },
            PartialMap::Full(p) => {
                p.insert(*key, *value);
            }
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        match self {
            PartialMap::Empty => HashMap::new().iter(),
            PartialMap::Partial(p) => p.iter(),
            PartialMap::Full(p) => p.iter()
        }
    }
}

struct Directory {
    pub path: String,
    files: PartialMap<String, File>,
    metadata: Lazy<HashMap<String, String>>
}

impl Directory {
    pub fn files<D: Database>(&mut self, d: &D) -> Result<Vec<File>> {
        match self.files {
            PartialMap::Empty => {
                let vec = d.directory_files(self)?;
                let hm: HashMap<String, File> = vec.iter().map(|x| x.filename).zip(vec.iter().map(|x| *x)).collect();

                self.files = PartialMap::Full(Box::new(hm));
                Ok(self.files.iter().map(|t| *t.1).collect())
            }
            PartialMap::Partial(p) => {
                let vec = d.directory_files(self)?;
                let hm: HashMap<String, File> = vec.iter().map(|x| x.filename).zip(vec.iter().map(|x| *x)).collect();

                self.files = PartialMap::Full(Box::new(hm));
                Ok(self.files.iter().map(|t| *t.1).collect())
            }
            PartialMap::Full(p) => {
                Ok(self.files.iter().map(|t| *t.1).collect())
            }
        }
    }

    pub fn file<D: Database>(&self, d: &D, filename: &str) -> Result<Option<File>> {
        
    }
}

impl Index<String> for Directory {

}

struct File {
    pub filename: String,
    pub hash: [u8; 32],
    directory: Lazy<Directory>,
    metadata: Lazy<HashMap<String, String>>
}

trait Database {
    fn file_directory(&self, f: &File) -> Result<Directory>;
    fn file_metadata(&self, f: &File) -> Result<HashMap<String, String>>;
    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>>;
    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>>;

    fn directory_file(&self, d: &Directory, filename: &str) -> Result<Option<File>>;
    fn directory_files(&self, d: &Directory) -> Result<Vec<File>>;
    fn directory_files_with_key(&self, key: &str) -> Result<Vec<File>>;
    fn directory_files_with_key_and_value(&self, key: &str, value: &str) -> Result<Vec<File>>;
    fn directory_metadata(&self, d: &Directory) -> Result<HashMap<String, String>>;
    fn directory_metadata_get(&self, d: &Directory, key: &str) -> Result<Option<String>>;
    fn directory_metadata_set(&self, d: &Directory, key: &str, value: Option<&str>) -> Result<Option<String>>;

    fn get_directory(&self, path: &str) -> Option<Directory>;
    fn get_directories(&self, ) -> Result<Vec<Directory>>;
    fn get_directories_with_key(&self, key: &str) -> Result<Vec<Directory>>;
    fn get_directories_with_key_and_value(&self, key: &str, value: &str) -> Result<Vec<Directory>>;
    fn get_files_with_key(&self, key: &str) -> Result<Vec<File>>;
    fn get_files_with_key_and_value(&self, key: &str, value: &str) -> Result<Vec<File>>;

    fn add_directory(&self, path: &str) -> Result<()>;
    fn add_directory_rec(&self, path: &str) -> Result<()>;
    fn add_file(&self, path: &str) -> Result<()>;
}

struct SqliteDatabase {
    conn: Connection
}

impl SqliteDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        {
            let statements = [
                "
                CREATE TABLE IF NOT EXISTS Directories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL UNIQUE
                );
                ",
                "
                CREATE TABLE IF NOT EXISTS Files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    directoryId INTEGER NOT NULL,
                    filename TEXT NOT NULL UNIQUE,
                    hash VARCHAR(64) NOT NULL,
                    FOREIGN KEY (directoryId) REFERENCES Directories(id)
                );
                ",
                "CREATE INDEX idx_files_hash ON Files(hash)",
                "
                CREATE TABLE IF NOT EXISTS DirectoryMetadata (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    directoryId INTEGER NOT NULL
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    UNIQUE(fileId, key),
                    FOREIGN KEY directoryId REFERENCES Directories(id)
                );
                ",
                "
                CREATE TABLE IF NOT EXISTS FileMetadata (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    fileId INTEGER NOT NULL
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    UNIQUE(fileId, key),
                    FOREIGN KEY fileId REFERENCES Files(id)
                );
                "
            ];

            for stmt in &statements {
                let mut prep = conn.prepare(stmt)?;
                prep.execute(params![])?;
            }
        }

        Ok(SqliteDatabase { conn })
    }
}

impl Database for SqliteDatabase {

}