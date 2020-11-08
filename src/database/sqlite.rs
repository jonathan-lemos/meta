use std::iter::FromIterator;
use crate::database::database::File;
use crate::database::database::Directory;
use crate::database::database::Database;
use std::path::Path;
use rusqlite::{Connection, params};
use crate::database::sqlite::SqliteError::*;

pub enum SqliteError {
    DbError(rusqlite::Error),
    ApplicationError(String)
}

trait ToSqlite<T> {
    fn to_db_err(self) -> Result<T, SqliteError>;
}

impl<T> ToSqlite<T> for rusqlite::Result<T> {
    fn to_db_err(self) -> Result<T, SqliteError> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(DbError(e))
        }
    }
}

pub struct SqliteDatabase {
    conn: Connection
}

impl SqliteDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, SqliteError> {
        let conn = Connection::open(path).to_db_err()?;

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
                let mut prep = conn.prepare(stmt).to_db_err()?;
                prep.execute(params![]).to_db_err()?;
            }
        }

        Ok(SqliteDatabase { conn })
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        let prep = self.conn.prepare("SELECT id, path FROM Directories d INNER JOIN Files f ON d.path = ").to_db_err()?;
        let rows = prep.query_map(params![f.filename], |row| Ok(Directory {
            id: row.get(0)?,
            path: row.get(1)?,

        })).to_db_err()?;

        for dir in rows {
            return Ok(dir.unwrap());
        }

        Err(ApplicationError(format!("The file '{}' does not have a parent directory in the database.", f.filename)))
    }

    fn file_metadata<B: FromIterator<(&'a str, &'a str)>>(&self, f: &File) -> Result<B, SqliteError> {
        let prep = self.conn.prepare("SELECT id, path FROM Directories d INNER JOIN Files f ON f.directoryId = d.Id").to_db_err()?;
        let rows = prep.query_map(params![f.filename], |row| Ok(Directory {
            id: row.get(0)?,
            path: row.get(1)?,

        })).to_db_err()?;


    }
}