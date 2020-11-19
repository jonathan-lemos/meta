use std::collections::BTreeMap;
use super::models::*;
use super::database::Database;
use super::path::Path;
use super::option_result::OptionResult;

use std::sync::{RwLock, Mutex, RwLockReadGuard, RwLockWriteGuard};
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

struct UnsynchronizedSqliteDatabase {
    conn: SqliteConnection,
}

impl UnsynchronizedSqliteDatabase {
    pub fn new(file_path: &str) -> Result<Self, SqliteError> {
        let conn = match SqliteConnection::establish(file_path) {
            Ok(c) => c,
            Err(e) => return Err(ApplicationError(format!("Failed to establish database connection: {:?}", e)))
        };

        match embedded_migrations::run(&conn) {
            Ok(_) => {},
            Err(e) => return Err(ApplicationError(format!("Failed to run migrations: {:?}", e)))
        }

        Ok(UnsynchronizedSqliteDatabase { conn })
    }
}

impl<'a> Database<'a, SqliteError> for UnsynchronizedSqliteDatabase {
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

    fn files_metadata<'b, B: FromIterator<(File, BTreeMap<String, String>)>, I: Iterator<Item=&'b File>>(&self, files: I) -> Result<B, E> {
        let f = files.map(|x| x.clone()).collect::<Vec<File>>();

        let res = FileKeyValuePair::belonging_to(&f)
            .load::<FileKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&f);

        let it = res.into_iter()
            .zip(f)
            .map(|x| (
                x.1,
                x.0.into_iter()
                    .map(|y| (y.key, y.value))
                        .collect::<BTreeMap<String, String>>()))
            .collect();

        Ok(it)
    }

    fn files_metadata_get<'b, B: FromIterator<(File, Option<String>)>, I: Iterator<Item=&'b File>>(&self, files: I, k: &str) -> Result<B, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata::dsl::*;

        let f = files.map(|x| x.clone()).collect::<Vec<File>>();
        f.sort_unstable_by_key(|f| f.id);

        let ids = f.iter().map(|x| x.id).collect::<Vec<i32>>();

        let res = Files.left_join(FileMetadata)
            .filter(super::schema::Files::dsl::id.eq_any(ids))
            .filter(key.eq(k))
            .load::<(File, Option<FileKeyValuePair>)>(&self.conn).to_db_err()?;

        Ok(res.into_iter()
            .map(|x| (x.0, x.1.map(|y| y.value)))
            .collect())
    }

    fn files_metadata_set<'b, I: Iterator<Item=&'b File>>(&self, files: I, k: &str, v: &str) -> Result<usize, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        self.conn.immediate_transaction(|| {
            let mut num = 0usize;

            for file in files {
                let res = update(FileMetadata)
                    .filter(file_id.eq(file.id))
                    .set(value.eq(v))
                    .execute(&self.conn)?;

                num += res;
            }

            Ok(num)
        }).to_db_err()
    }

    fn files_with_key<'b, B: FromIterator<File>, I: Iterator<Item=&'b File>>(&self, files: I, k: &str) -> Result<B, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let files = files.map(|x| x.clone()).collect::<Vec<File>>();
        files.sort_unstable_by_key(|x| x.id);

        let res = FileKeyValuePair::belonging_to(&files)
            .filter(key.eq(k))
            .load::<FileKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&files);

        let it = res.into_iter()
            .zip(&files)
            .filter(|x| x.0.len() > 0)
            .map(|x| x.1)
            .collect();
    }

    fn files_with_key_and_value<'b, B: FromIterator<File>, I: Iterator<Item=&'b File>>(&self, files: I, k: &str, v: &str) -> Result<B, SqliteError> {
        use super::schema::FileMetadata::dsl::*;

        let files = files.map(|x| x.clone()).collect::<Vec<File>>();
        files.sort_unstable_by_key(|x| x.id);

        let res = FileKeyValuePair::belonging_to(&files)
            .filter(key.eq(k).and(value.eq(v)))
            .load::<FileKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&files);

        let it = res.into_iter()
            .zip(&files)
            .filter(|x| x.0.len() > 0)
            .map(|x| x.1)
            .collect();
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

    fn directory_subdir(&self, d: &Directory, filename: &str) -> Result<Option<Directory>, SqliteError> {
        use super::schema::Directories::dsl::*;
        
        let d = Path::new(&d.path) / filename;

        Directories.filter(path.eq(d.str()))
            .first::<Directory>(&self.conn)
            .optional().to_db_err()
    }

    fn directory_entries<F: FromIterator<File>, D: FromIterator<Directory>>(&self, d: &Directory) -> Result<(F, D), SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        let f = F
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

        let new_dirs = pat_refs.into_iter().map(|p| NewDirectory {
                path: p
            }).collect::<Vec<NewDirectory>>();

        let res = insert_or_ignore_into(Directories)
            .values(new_dirs)
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

    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, SqliteError> {
        use super::schema::Files::dsl::*;
        use crate::linq::group_by::GroupBy;

        let it = paths.into_iter().map(|e| (parent_dir(&e.0), preprocess(e.0), e.1)).collect::<Vec<(Option<&str>, String, &[u8])>>();

        let errors = it.iter().filter(|e| e.0.is_none()).collect::<Vec<&(Option<&str>, String, &[u8])>>();
        if errors.len() > 0 {
            return Err(ApplicationError(
                format!("The following paths cannot have a parent directory: {}",
                    errors.into_iter().map(|e| e.1.clone()).collect::<Vec<String>>().join(", "))
            ));
        }

        let groups = it.iter().group_by(|e| e.0.unwrap());

        for (parent, tuples) in groups {
            let dir = match self.get_directory(parent)? {
                Some(d) => d,
                None => self.add_directory(parent)?.0
            };

            let new_files = tuples.into_iter().map(|t| NewFile {
                directory_id: dir.id,
                filename: super::path::filename(&t.1),
                hash: t.2
            }).collect::<Vec<NewFile>>();

            insert_or_ignore_into(Files)
                .values(new_files)
                .execute(&self.conn).to_db_err()?;
        }

        return Err(ApplicationError("".to_owned()));
    }

}

pub struct SqliteDatabase {
    usd: UnsynchronizedSqliteDatabase,
    lock_mtx: Mutex<i32>,
    file_lock: RwLock<i32>,
    file_meta_lock: RwLock<i32>,
    dir_lock: RwLock<i32>,
    dir_meta_lock: RwLock<i32>,
}

enum LockGuard<'a> {
    Empty,
    Read(RwLockReadGuard<'a, i32>),
    Write(RwLockWriteGuard<'a, i32>)
}

struct SqliteDatabaseLockContext<'a> {
    file: LockGuard<'a>,
    file_meta: LockGuard<'a>,
    dir: LockGuard<'a>,
    dir_meta: LockGuard<'a>
}

enum Lock {
    File,
    FileMeta,
    Dir,
    DirMeta
}

enum LockMode {
    Read,
    Write
}

impl SqliteDatabase {
    pub fn new(file_path: &str) -> Result<Self, SqliteError> {
        Ok(SqliteDatabase {
            usd: UnsynchronizedSqliteDatabase::new(file_path)?,
            lock_mtx: Mutex::new(0),
            file_lock: RwLock::new(0),
            file_meta_lock: RwLock::new(0),
            dir_lock: RwLock::new(0),
            dir_meta_lock: RwLock::new(0),
        })
    }

    pub fn ctx(&self, request: &[(Lock, LockMode)]) -> SqliteDatabaseLockContext<'_> {
        use self::Lock::*;
        use self::LockMode::*;

        let ret = SqliteDatabaseLockContext {
            file: LockGuard::Empty,
            file_meta: LockGuard::Empty,
            dir: LockGuard::Empty,
            dir_meta: LockGuard::Empty,
        };

        let _ = self.lock_mtx.lock().expect("Meta-mutex was poisoned.");

        for (lock, mode) in request {
            let (r, l) = match lock {
                File => (&mut ret.file, self.file_lock),
                FileMeta => (&mut ret.file_meta, self.file_meta_lock),
                Dir => (&mut ret.dir, self.dir_lock),
                DirMeta => (&mut ret.dir_meta, self.dir_meta_lock)
            };

            *r = match mode {
                Read => LockGuard::Read(l.read().expect("Database lock was poisoned.")),
                Write => LockGuard::Write(l.write().expect("Database lock was poisoned."))
            };
        };

        ret
    }
}

impl<'a> Database<'a, SqliteError> for SqliteDatabase {
    fn file_directory(&self, f: &File) -> Result<Directory, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read)]);

        self.usd.file_directory(f)
    }

    fn file_metadata<B: FromIterator<(String, String)>>(&self, f: &File) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(FileMeta, Read), (File, Read)]);

        self.usd.file_metadata(f)
    }

    fn file_metadata_get(&self, f: &File, k: &str) -> Result<Option<String>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(FileMeta, Read), (File, Read)]);

        self.usd.file_metadata_get(f, k)
    }

    fn file_metadata_set(&self, f: &File, k: &str, v: Option<&str>) -> Result<Option<String>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(FileMeta, Write), (File, Read)]);

        self.usd.file_metadata_set(f, k, v)
    }

    fn directory_file(&self, d: &Directory, fname: &str) -> Result<Option<File>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(Dir, Read), (File, Read)]);

        self.usd.directory_file(d, fname)
    }

    fn directory_files<B: FromIterator<File>>(&self, d: &Directory) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(Dir, Read), (File, Read)]);

        self.usd.directory_files(d)
    }

    fn directory_files_with_key<B: FromIterator<File>>(&self, d: &Directory, k: &str) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(Dir, Read), (File, Read), (FileMeta, Read)]);

        self.usd.directory_files_with_key(d, k)
    }

    fn directory_files_with_key_and_value<B: FromIterator<File>>(&self, d: &Directory, k: &str, v: &str) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(Dir, Read), (File, Read), (FileMeta, Read)]);

        self.usd.directory_files_with_key_and_value(d, k, v)
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

        let new_dirs = pat_refs.into_iter().map(|p| NewDirectory {
                path: p
            }).collect::<Vec<NewDirectory>>();

        let res = insert_or_ignore_into(Directories)
            .values(new_dirs)
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

    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, SqliteError> {
        use super::schema::Files::dsl::*;
        use crate::linq::group_by::GroupBy;

        let it = paths.into_iter().map(|e| (parent_dir(&e.0), preprocess(e.0), e.1)).collect::<Vec<(Option<&str>, String, &[u8])>>();

        let errors = it.iter().filter(|e| e.0.is_none()).collect::<Vec<&(Option<&str>, String, &[u8])>>();
        if errors.len() > 0 {
            return Err(ApplicationError(
                format!("The following paths cannot have a parent directory: {}",
                    errors.into_iter().map(|e| e.1.clone()).collect::<Vec<String>>().join(", "))
            ));
        }

        let groups = it.iter().group_by(|e| e.0.unwrap());

        for (parent, tuples) in groups {
            let dir = match self.get_directory(parent)? {
                Some(d) => d,
                None => self.add_directory(parent)?.0
            };

            let new_files = tuples.into_iter().map(|t| NewFile {
                directory_id: dir.id,
                filename: super::path::filename(&t.1),
                hash: t.2
            }).collect::<Vec<NewFile>>();

            insert_or_ignore_into(Files)
                .values(new_files)
                .execute(&self.conn).to_db_err()?;
        }

        return Err(ApplicationError("".to_owned()));
    }

}