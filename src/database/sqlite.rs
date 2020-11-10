

use std::sync::Mutex;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::sync::{RwLock, PoisonError};
use std::iter::FromIterator;
use crate::database::database::File;
use crate::database::database::Directory;
use crate::database::database::Database;
use std::path::Path;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::{ConnectionError};

use crate::database::sqlite::SqliteError::*;

pub enum SqliteError {
    DbError(diesel::result::Error),
    ApplicationError(String),
}

trait ToSqlite<T> {
    fn to_db_err(self) -> Result<T, SqliteError>;
}

impl<T> ToSqlite<T> for diesel::result::QueryResult<T> {
    fn to_db_err(self) -> Result<T, SqliteError> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(DbError(e))
        }
    }
}

pub struct SqliteDatabase {
    conn: SqliteConnection,
}

embed_migrations!();

impl SqliteDatabase {
    pub fn new<P: AsRef<Path>>(path: &str) -> Result<Self, ConnectionError> {
        let conn = SqliteConnection::establish(path)?;

        embedded_migrations::run(&conn);

        Ok(SqliteDatabase { conn })
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        let (prep, _, _) = self.prepare("SELECT d.id, d.path FROM Directories d INNER JOIN Files f ON f.directoryId = d.Id WHERE f.filename = ?")?;
        let rows = prep.query_map(params![f.filename()], |row| Ok(Directory {
            id: row.get(0)?,
            path: row.get(1)?,

        })).to_db_err()?;

        for dir in rows {
            return Ok(dir.unwrap());
        }

        Err(ApplicationError(format!("The file '{}' does not have a parent directory in the database.", f.filename())))
    }

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, SqliteError> {
        let (prep, _, _) = self.prepare("SELECT m.key, m.value FROM FileMetadata m INNER JOIN Files f ON m.fileId = f.id WHERE f.filename = ?").to_db_err()?;
        let rows = prep.query_map(params![f.filename()], |row| Ok((row.get(0)?, row.get(1)?))).to_db_err()?;

        let res = rows.map(|r| r.unwrap());
        
        Ok(res.collect())
    }

    fn file_metadata_get(&self, f: &File, key: &str) -> Result<Option<String>, SqliteError> {
        let (prep, _, _) = self.prepare("SELECT m.value FROM FileMetadata m INNER JOIN Files f ON m.fileId = f.id WHERE f.filename = ? AND m.key = ?").to_db_err()?;
        let rows = prep.query_map(params![f.filename(), key], |row| Ok(row.get(0)?)).to_db_err()?;

        let res = rows.map(|r| r.unwrap());
        
        for val in res {
            return Ok(Some(val));
        }

        Ok(None)
    }

    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, SqliteError> {
        if let val = Some(value) {
            let (prep, _, _) = self.prepare("INSERT OR IGNORE INTO FileMetadata VALUES (?, ?)").to_db_err()?;
            let rows = prep.query_map(params![key, value], |row| Ok(row.get(0)?)).to_db_err()?;

            let res = rows.map(|r| r.unwrap());
        
            for val in res {
                return Ok(Some(val));
            }

            Ok(None)
        }

        Ok(None)
    }
}