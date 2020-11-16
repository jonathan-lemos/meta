use super::models::*;
use super::database::Database;
use super::path::{parent_dir, preprocess};
use super::option_result::OptionResult;

use std::iter::FromIterator;
use diesel::{insert_into, insert_or_ignore_into, update, delete};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

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
}


impl SqliteDatabase {
    pub fn new(file_path: &str) -> Result<Self, SqliteError> {
        let conn = match SqliteConnection::establish(file_path) {
            Ok(c) => c,
            Err(e) => return Err(ApplicationError(format!("Failed to establish database connection: {:?}", e)))
        };

        match embedded_migrations::run(&conn) {
            Ok(_) => {},
            Err(e) => return Err(ApplicationError(format!("Failed to run migrations: {:?}", e)))
        }

        Ok(SqliteDatabase { conn })
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::Directories::dsl::*;

        Ok(Files.find(f.id)
            .inner_join(Directories)
            .first::<(File, Directory)>(&self.conn).to_db_err()?
            .1)
    }

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        Ok(FileKeyValuePair::belonging_to(f)
            .select((key, value))
            .load::<(String, String)>(&self.conn).to_db_err()?
            .into_iter()
            .collect())
    }

    fn file_metadata_get(&self, f: &File, k: &str) -> Result<Option<String>, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        FileKeyValuePair::belonging_to(f)
            .filter(key.eq(k))
            .select(value)
            .first::<String>(&self.conn)
            .optional().to_db_err()
    }

    fn file_metadata_set(&self, f: &File, k: &str, v: Option<&str>) -> Result<Option<String>, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let existing = self.file_metadata_get(f, k)?;

        match v {
            None => {
                delete(
                    FileKeyValuePair::belonging_to(f)
                    .filter(key.eq(k))
                ).execute(&self.conn).to_db_err()?;
            },
            Some(s) => {
                match &existing {
                    None => {
                        insert_into(FileMetadata)
                            .values(&NewFileKeyValuePair {
                                file_id: f.id,
                                key: k,
                                value: s
                            })
                            .execute(&self.conn).to_db_err()?;
                    }
                    Some(_s2) => {
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

    fn directory_file(&self, d: &Directory, fname: &str) -> Result<Option<File>, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        Directories.find(d.id)
            .inner_join(Files)
            .filter(filename.eq(fname))
            .first::<(Directory, File)>(&self.conn)
            .map(|x| x.1)
            .optional().to_db_err()
    }

    fn directory_files<B: FromIterator<File>>(&self, d: &Directory) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        let res = Directories.find(d.id)
            .inner_join(Files)
            .load::<(Directory, File)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.1)
            .collect();

        Ok(res)
    }

    fn directory_files_with_key<B: FromIterator<File>>(&self, d: &Directory, k: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata::dsl::*;

        let res = Files.inner_join(FileMetadata)
            .filter(key.eq(k))
            .inner_join(Directories)
            .load::<(File, FileKeyValuePair, Directory)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.0)
            .collect();

        Ok(res)
    }

    fn directory_files_with_key_and_value<B: FromIterator<File>>(&self, d: &Directory, k: &str, v: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata::dsl::*;

        let res = Files.inner_join(FileMetadata)
            .filter(key.eq(k).and(value.eq(k)))
            .inner_join(Directories)
            .load::<(File, FileKeyValuePair, Directory)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.0)
            .collect();

        Ok(res)
   }

    fn directory_metadata<B: FromIterator<(String, String)>>(&self, d: &Directory) -> Result<B, SqliteError> {
        use super::schema::DirectoryMetadata::dsl::*;

        Ok(DirectoryKeyValuePair::belonging_to(d)
            .select((key, value))
            .load::<(String, String)>(&self.conn).to_db_err()?
            .into_iter()
            .collect())
    }

    fn directory_metadata_get(&self, d: &Directory, k: &str) -> Result<Option<String>, SqliteError> {
        use super::schema::DirectoryMetadata::dsl::*;

        DirectoryKeyValuePair::belonging_to(d)
            .filter(key.eq(k))
            .select(value)
            .first::<String>(&self.conn)
            .optional().to_db_err()
    }

    fn directory_metadata_set(&self, d: &Directory, k: &str, v: Option<&str>) -> Result<Option<String>, SqliteError> {
        use super::schema::DirectoryMetadata::dsl::*;

        let existing = self.directory_metadata_get(d, k)?;

        match v {
            None => {
                delete(
                    DirectoryKeyValuePair::belonging_to(d)
                    .filter(key.eq(k))
                ).execute(&self.conn).to_db_err()?;
            },
            Some(s) => {
                match &existing {
                    None => {
                        insert_into(DirectoryMetadata)
                            .values(&NewDirectoryKeyValuePair {
                                directory_id: d.id,
                                key: k,
                                value: s
                            })
                            .execute(&self.conn).to_db_err()?;
                    }
                    Some(_s2) => {
                        update(DirectoryMetadata)
                            .filter(key.eq(k))
                            .set(value.eq(s))
                            .execute(&self.conn).to_db_err()?;
                    }
                }                
            }
        }

        Ok(existing)
    }

    fn get_directory(&self, p: &str) -> Result<Option<Directory>, SqliteError> {
        use super::schema::Directories::dsl::*;

        let p = &preprocess(p);
        
        Directories.filter(path.eq(p).or(path.eq(p.trim_end_matches("/"))))
            .first::<Directory>(&self.conn)
            .optional().to_db_err()
    }

    fn get_directories<B: FromIterator<Directory>>(&self) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;

        let res = Directories.load::<Directory>(&self.conn).to_db_err()?
                    .into_iter()
                    .collect();

        Ok(res)
    }

    fn get_directories_with_key<B: FromIterator<Directory>>(&self, k: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::DirectoryMetadata::dsl::*;

        let res = DirectoryMetadata
            .filter(key.eq(k))
            .inner_join(Directories)
            .load::<(DirectoryKeyValuePair, Directory)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.1)
            .collect();

        Ok(res)
    }

    fn get_directories_with_key_and_value<B: FromIterator<Directory>>(&self, k: &str, v: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::DirectoryMetadata::dsl::*;

        let res = DirectoryMetadata
            .filter(key.eq(k).and(value.eq(v)))
            .inner_join(Directories)
            .load::<(DirectoryKeyValuePair, Directory)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.1)
            .collect();

        Ok(res)
    }

    fn get_file(&self, p: &str) -> Result<Option<(File, Directory)>, SqliteError> {
        use super::schema::Files::dsl::*;

        let p = &preprocess(p);

        let parent = match parent_dir(p) {
            Some(s) => s,
            None => return Err(ApplicationError("The path does not have a parent directory ('{}' was given).".to_owned()))
        };

        let fname = super::path::filename(p);
        
        let dir = match self.get_directory(parent)? {
            Some(s) => s,
            None => return Ok(None)
        };

        File::belonging_to(&dir)
            .filter(filename.eq(fname))
            .first::<File>(&self.conn)
            .optional()
            .map_value(|s| (s, dir))
            .to_db_err()
    }

    fn get_files<B: FromIterator<File>>(&self) -> Result<B, SqliteError> {
        use super::schema::Files::dsl::*;
        
        let res = Files.load::<File>(&self.conn).to_db_err()?
                    .into_iter()
                    .collect();

        Ok(res)
    }

    fn get_files_with_key<B: FromIterator<File>>(&self, k: &str) -> Result<B, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata::dsl::*;
        
        let res = FileMetadata
            .filter(key.eq(k))
            .inner_join(Files)
            .load::<(FileKeyValuePair, File)>(&self.conn).to_db_err()?
            .into_iter()
            .map(|x| x.1)
            .collect();

        Ok(res)
    }

    fn get_files_with_key_and_value<B: FromIterator<File>>(&self, k: &str, v: &str) -> Result<B, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata::dsl::*;

        let res = FileMetadata
            .filter(key.eq(k).and(value.eq(v)))
            .inner_join(Files)
            .load::<(FileKeyValuePair, File)>(&self.conn).to_db_err()?
            .iter()
            .map(|x| x.1.clone())
            .collect();

        Ok(res)
    }

    fn add_directory(&self, p: &str) -> Result<(Directory, bool), SqliteError> {
        use super::schema::Directories::dsl::*;

        let p = &preprocess(p);

        let res = insert_into(Directories)
            .values(NewDirectory {
                path: p
            })
            .execute(&self.conn).to_db_err()?;

        let dir = Directories.filter(path.eq(p))
            .first::<Directory>(&self.conn).to_db_err()?;

        if res == 0 {
            return Ok((dir, false));
        }

        while let Some(p) = parent_dir(p) {
            let res = insert_into(Directories)
                .values(NewDirectory {
                    path: p
                })
                .execute(&self.conn).to_db_err()?;
            
            if res == 0 {
                break;
            }
        }

        return Ok((dir, true));
    }

    fn add_directories<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<usize, SqliteError> {
        use super::schema::Directories::dsl::*;

        let mut pat_cache = Vec::<String>::new();
        let mut pat_refs = Vec::<&str>::new();

        for p in paths {
            pat_cache.push(preprocess(p));
        }

        for p in &pat_cache {
            pat_refs.push(p);

            while let Some(p) = parent_dir(p) {
                pat_refs.push(p);
            }
        }

        let newDirs = pat_refs.into_iter().map(|p| NewDirectory {
                path: p
            }).collect::<Vec<NewDirectory>>();

        let res = insert_or_ignore_into(Directories)
            .values(newDirs)
            .execute(&self.conn).to_db_err()?;

        return Ok(res);
    }

    fn add_file(&self, p: &str, h: &[u8]) -> Result<(File, bool), SqliteError> {
        use super::schema::Files::dsl::*;

        let p = &preprocess(p);

        let parent = match parent_dir(p) {
            Some(s) => s,
            None => return Err(ApplicationError("The path does not have a parent directory ('{}' was given).".to_owned()))
        };

        let (dir, _) = self.add_directory(parent)?;

        let res = insert_or_ignore_into(Files)
            .values(NewFile {
                directory_id: dir.id,
                filename: super::path::filename(p),
                hash: h
            })
            .execute(&self.conn).to_db_err()?;

        let file = Files.filter(filename.eq(p))
            .first::<File>(&self.conn).to_db_err()?;

        return Ok((file, res > 0));
    }

}