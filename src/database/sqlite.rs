use super::models::*;
use super::database::Database;

use std::iter::FromIterator;
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
    pub fn new(file_path: &str) -> Result<Self, ConnectionError> {
        let conn = SqliteConnection::establish(file_path)?;

        embedded_migrations::run(&conn);

        Ok(SqliteDatabase { conn })
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::Directories::dsl::*;

        Ok(Files.inner_join(Directories)
            .first::<(File, Directory)>(&self.conn).to_db_err()?
            .1)
    }

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        Ok(FileKeyValuePair::belonging_to(f)
            .select((key, value))
            .load::<(String, String)>(&self.conn).to_db_err()?
            .iter()
            .map(|x| *x)
            .collect())
    }

    fn file_metadata_get(&self, f: &File, k: &str) -> Result<Option<String>, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

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

    fn file_metadata_set(&self, f: &File, key: &str, value: Option<&str>) -> Result<Option<String>, SqliteError> {
    }
    */
}