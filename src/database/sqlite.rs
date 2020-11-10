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
use crate::database::sqlite::SqliteError::*;

pub enum SqliteError {
    DbError(rusqlite::Error),
    ApplicationError(String),
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

struct Locks {
    file_mtx: RwLock<i32>,
    file_meta_mtx: RwLock<i32>,
    dir_mtx: RwLock<i32>,
    dir_meta_mtx: RwLock<i32>
}

impl Locks {
    pub fn new() -> Self {
        Locks {
            file_mtx: RwLock::new(0),
            file_meta_mtx: RwLock::new(0),
            dir_mtx: RwLock::new(0),
            dir_meta_mtx: RwLock::new(0)
        }
    }

    pub fn new_context(&self) -> LockContext<'_> {
        LockContext {
            file_mtx: None,
            file_meta_mtx: None,
            dir_mtx: None,
            dir_meta_mtx: None,
            locks: self
        }
    }
}

enum LockGuard<'a> {
    Read(RwLockReadGuard<'a, i32>),
    Write(RwLockWriteGuard<'a, i32>)
}

struct LockContext<'a> {
    file_mtx: Option<LockGuard<'a>>,
    file_meta_mtx: Option<LockGuard<'a>>,
    dir_mtx: Option<LockGuard<'a>>,
    dir_meta_mtx: Option<LockGuard<'a>>,
    locks: &'a Locks
}

impl<'a> LockContext<'a> {
    pub fn read_file(&mut self) {
        match self.file_mtx {
            None => {
                self.file_mtx = Some(LockGuard::Read(self.locks.file_mtx.read().unwrap()));
            }
            Some(s) => {
                match s {
                    LockGuard::Read(r) => {}
                    LockGuard::Write(w) => {
                        {
                            self.file_mtx = None
                        }
                        self.file_mtx = Some(LockGuard::Write(self.locks.file_mtx.write().unwrap()))
                    }
                }
            }
        }
    }

    pub fn read_file_meta(&mut self) {
        match self.file_meta_mtx {
            None => {
                self.file_meta_mtx = Some(LockGuard::Read(self.locks.file_meta_mtx.read().unwrap()));
            }
            Some(s) => {
                match s {
                    LockGuard::Read(r) => {}
                    LockGuard::Write(w) => {
                        {
                            self.file_meta_mtx = None
                        }
                        self.file_meta_mtx = Some(LockGuard::Write(self.locks.file_meta_mtx.write().unwrap()))
                    }
                }
            }
        }
    }

}


pub struct SqliteDatabase {
    conn: Connection,
    lock_lock: Mutex<i32>,
}

impl Drop for SqliteDatabase {
    fn drop(&mut self) {
        self.conn.close();
    }
}

impl SqliteDatabase {
    fn prepare(&self, sql: &str) -> Result<(rusqlite::Statement, Vec<RwLockReadGuard<i32>>, Vec<RwLockWriteGuard<i32>>), SqliteError> {
        let _ = self.lock_lock.lock().unwrap();

        let res = self.conn.prepare(sql).to_db_err()?;
        let read_locks = Vec::new();
        let write_locks = Vec::new();

        if sql.trim_start().starts_with("SELECT") {
            if sql.contains("Files") {
                read_locks.push(self.file_mtx.read().unwrap());
            }
    
            if sql.contains("Directories") {
                read_locks.push(self.dir_mtx.read().unwrap());
            }
    
            if sql.contains("FileMetadata") {
                read_locks.push(self.file_meta_mtx.read().unwrap());
            }
    
            if sql.contains("DirectoryMetadata") {
                read_locks.push(self.dir_meta_mtx.read().unwrap());
            }
        }
        else {
            if sql.contains("Files") {
                write_locks.push(self.file_mtx.write().unwrap());
            }
    
            if sql.contains("Directories") {
                write_locks.push(self.dir_mtx.write().unwrap());
            }
    
            if sql.contains("FileMetadata") {
                write_locks.push(self.file_meta_mtx.write().unwrap());
            }
    
            if sql.contains("DirectoryMetadata") {
                write_locks.push(self.dir_meta_mtx.write().unwrap());
            }
        }

        Ok((res, read_locks, write_locks))
    }

    pub fn new<P: AsRef<Path>>(path: &str) -> Result<Self, SqliteError> {
        let conn = SqliteConnection::establish(path);
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