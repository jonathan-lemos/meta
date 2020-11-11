use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use super::models::*;
use super::database::Database;

use std::iter::FromIterator;
use diesel::{insert_into, update, delete};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::{ConnectionError};

use crate::database::sqlite::SqliteError::*;

embed_migrations!();

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
    locks: SqliteLocks
}

struct SqliteLocks {
    file_lock: RwLock<i32>,
    dir_lock: RwLock<i32>,
    file_meta_lock: RwLock<i32>,
    dir_meta_lock: RwLock<i32>,
    mtx: Mutex<i32>
}

impl SqliteLocks {
    pub fn new() -> Self {
        SqliteLocks {
            file_lock: RwLock::new(0),
            dir_lock: RwLock::new(0),
            file_meta_lock: RwLock::new(0),
            dir_meta_lock: RwLock::new(0),
            mtx: Mutex::new(0)
        }        
    }

    pub fn context(&self) -> &SqliteLockContext {
        &SqliteLockContext {
            file_guard: LockGuard::None,
            dir_guard: LockGuard::None,
            file_meta_guard: LockGuard::None,
            dir_meta_guard: LockGuard::None,
            locks: self
        }
    }
}

enum LockGuard<'a> {
    None,
    Read(RwLockReadGuard<'a, i32>),
    Write(RwLockWriteGuard<'a, i32>)
}

struct SqliteLockContext<'a> {
    file_guard: LockGuard<'a>,
    dir_guard: LockGuard<'a>,
    file_meta_guard: LockGuard<'a>,
    dir_meta_guard: LockGuard<'a>,
    locks: &'a SqliteLocks
}

#[derive(PartialEq, Eq, Debug, Hash)]
enum Field {
    File,
    Dir,
    FileMeta,
    DirMeta
}

#[derive(PartialEq, Eq, Debug, Hash)]
enum Operation {
    Read,
    Write
}

impl<'a> SqliteLockContext<'a> {
    pub fn lock(&self, request: &[(Field, Operation)]) -> &Self {
        let _ = self.locks.mtx.lock().expect("Lock mutex is poisoned. This should never happen.");

        for (field, operation) in request {
            let guard = match field {
                Field::File => &mut self.file_guard,
                Field::Dir => &mut self.dir_guard,
                Field::FileMeta => &mut self.file_meta_guard,
                Field::DirMeta => &mut self.dir_meta_guard
            };

            let lock = match field {
                Field::File => &self.locks.file_lock,
                Field::Dir => &self.locks.dir_lock,
                Field::FileMeta => &self.locks.file_meta_lock,
                Field::DirMeta => &self.locks.dir_meta_lock
            };

            match guard {
                LockGuard::None => {
                    match operation {
                        Operation::Read => *guard = LockGuard::Read(lock.read().expect("Lock is poisoned. This should never happen.")),
                        Operation::Write => *guard = LockGuard::Write(lock.write().expect("Lock is poisoned. This should never happen."))
                    }
                },
                LockGuard::Read(r) => {
                    match operation {
                        Operation::Read => {},
                        Operation::Write => {
                            {
                                *guard = LockGuard::None
                            }
                            *guard = LockGuard::Write(lock.write().expect("Lock is poisoned. This should never happen."))
                        }
                    }
                }
                LockGuard::Write(w) => {
                    match operation {
                        Operation::Write => {},
                        Operation::Read => {
                            {
                                *guard = LockGuard::None
                            }
                            *guard = LockGuard::Read(lock.read().expect("Lock is poisoned. This should never happen."))
                        }
                    }
                }
            }
        }

        self
    }

    pub fn release(&self) -> &Self {
        for mut guard in &[self.file_guard, self.dir_guard, self.file_meta_guard, self.dir_meta_guard] {
            *guard = LockGuard::None;
        }

        self
    }
}

impl SqliteDatabase {
    pub fn new(file_path: &str) -> Result<Self, ConnectionError> {
        let conn = SqliteConnection::establish(file_path)?;

        embedded_migrations::run(&conn);

        Ok(SqliteDatabase { conn, locks: SqliteLocks::new() })
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::Directories::dsl::*;

        let _ = self.locks.context().lock(&[
            (Field::File, Operation::Read),
            (Field::Dir, Operation::Read)]
        );

        Ok(Files.inner_join(Directories)
            .first::<(File, Directory)>(&self.conn).to_db_err()?
            .1)
    }

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let _ = self.locks.context().lock(&[
            (Field::File, Operation::Read),
            (Field::FileMeta, Operation::Read)]
        );

        Ok(FileKeyValuePair::belonging_to(f)
            .select((key, value))
            .load::<(String, String)>(&self.conn).to_db_err()?
            .iter()
            .map(|x| *x)
            .collect())
    }

    fn file_metadata_get(&self, f: &File, k: &str) -> Result<Option<String>, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let _ = self.locks.context().lock(&[
            (Field::File, Operation::Read),
            (Field::FileMeta, Operation::Read)]
        );

        let res = FileKeyValuePair::belonging_to(f)
            .filter(key.eq(k))
            .select(value)
            .first::<String>(&self.conn);

        match res {
            Ok(s) => Ok(Some(s)),
            Err(e) => {
                if let NotFound = e {
                    Ok(None)
                }
                else {
                    Err(DbError(e))
                }
            }
        }
    }

    fn file_metadata_set(&self, f: &File, k: &str, v: Option<&str>) -> Result<Option<String>, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let _ = self.locks.context().lock(&[
            (Field::File, Operation::Read),
            (Field::FileMeta, Operation::Write)]
        );

        let existing = FileKeyValuePair::belonging_to(f)
            .filter(key.eq(k))
            .select(value)
            .load::<String>(&self.conn).to_db_err()?;

        let existing = if existing.len() == 0 {
            None
        } else {
            Some(existing[0])
        };

        match v {
            None => {
                delete(
                    FileKeyValuePair::belonging_to(f)
                    .filter(key.eq(k))
                ).execute(&self.conn).to_db_err()?;
            },
            Some(s) => {
                match existing {
                    None => {
                        insert_into(FileMetadata)
                            .values(&NewFileKeyValuePair {
                                file_id: f.id,
                                key: k,
                                value: s
                            })
                            .execute(&self.conn).to_db_err()?;
                    }
                    Some(s2) => {
                        update(FileMetadata)
                            .filter(key.eq(k))
                            .set(value.eq(s))
                            .execute(&self.conn).to_db_err()?;
                    }
                }                
            }
        }

        Ok(existing)
    }
}