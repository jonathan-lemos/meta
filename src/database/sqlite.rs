use std::collections::BTreeMap;
use super::models::*;
use super::database::{Database, Entry};
use super::path::Path;
use super::option_result::OptionResult;

use std::sync::{RwLock, Mutex, RwLockReadGuard, RwLockWriteGuard};
use std::iter::{Chain, FromIterator};
use std::collections::{HashMap, HashSet};
use diesel::{insert_into, insert_or_ignore_into, update, delete};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::database::sqlite::SqliteError::*;
use crate::linq::collectors::Collect;
use crate::format::prettify::PrettyPaths;

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

impl From<diesel::result::Error> for SqliteError {
    fn from(d: diesel::result::Error) -> SqliteError {
        DbError(d)
    }
}

impl From<SqliteError> for diesel::result::Error {
    fn from(s: SqliteError) -> diesel::result::Error {
        match s {
            SqliteError::DbError(d) => d,
            ApplicationError(_) => diesel::result::Error::NotFound
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

    fn entry_metadata<B: FromIterator<(String, String)>>(&self, entry: &Entry) -> Result<B, SqliteError> {
        match entry {
            Entry::File(f) => {
                use super::schema::FileMetadata::dsl::*;

                Ok(FileKeyValuePair::belonging_to(f)
                    .select((key, value))
                    .load::<(String, String)>(&self.conn).to_db_err()?
                    .into_iter()
                    .collect())
            }
            Entry::Directory(d) => {
                use super::schema::DirectoryMetadata::dsl::*;

                Ok(DirectoryKeyValuePair::belonging_to(d)
                    .select((key, value))
                    .load::<(String, String)>(&self.conn).to_db_err()?
                    .into_iter()
                    .collect())
            }
        }
    }

    fn entry_metadata_get(&self, entry: &Entry, k: &str) -> Result<Option<String>, SqliteError> {
        match entry {
            Entry::File(f) => {
                use super::schema::FileMetadata::dsl::*;

                FileKeyValuePair::belonging_to(f)
                    .filter(key.eq(k))
                    .select(value)
                    .first::<String>(&self.conn)
                    .optional().to_db_err()
            }
            Entry::Directory(d) => {
                use super::schema::DirectoryMetadata::dsl::*;

                DirectoryKeyValuePair::belonging_to(d)
                    .filter(key.eq(k))
                    .select(value)
                    .first::<String>(&self.conn)
                    .optional().to_db_err()
            }
        }
    }

    fn entry_metadata_set(&self, entry: &Entry, k: &str, v: Option<&str>) -> Result<Option<String>, SqliteError> {
        let existing = self.entry_metadata_get(entry, k)?;

        match v {
            None => {
                match entry {
                    Entry::File(f) => {
                        use super::schema::FileMetadata::dsl::*;

                        delete(
                            FileKeyValuePair::belonging_to(f)
                            .filter(key.eq(k))
                        ).execute(&self.conn).to_db_err()?;
                    }
                    Entry::Directory(d) => {
                        use super::schema::DirectoryMetadata::dsl::*;

                        delete(
                            DirectoryKeyValuePair::belonging_to(d)
                            .filter(key.eq(k))
                        ).execute(&self.conn).to_db_err()?;
                    }
                }
            },
            Some(val) => {
                match &existing {
                    None => {
                        match entry {
                            Entry::File(f) => {
                                use super::schema::FileMetadata::dsl::*;

                                insert_into(FileMetadata)
                                    .values(&NewFileKeyValuePair {
                                        file_id: f.id,
                                        key: k,
                                        value: val
                                    })
                                    .execute(&self.conn).to_db_err()?;
                            }
                            Entry::Directory(d) => {
                                use super::schema::DirectoryMetadata::dsl::*;

                                insert_into(DirectoryMetadata)
                                    .values(&NewDirectoryKeyValuePair {
                                        directory_id: d.id,
                                        key: k,
                                        value: val
                                    })
                                    .execute(&self.conn).to_db_err()?;
                            }
                        }
                    }
                    Some(_s2) => {
                        match entry {
                            Entry::File(f) => {
                                use super::schema::FileMetadata::dsl::*;

                                update(FileMetadata)
                                    .filter(key.eq(k))
                                    .set(value.eq(val))
                                    .execute(&self.conn).to_db_err()?;
                            },
                            Entry::Directory(f) => {
                                use super::schema::DirectoryMetadata::dsl::*;

                                update(DirectoryMetadata)
                                    .filter(key.eq(k))
                                    .set(value.eq(val))
                                    .execute(&self.conn).to_db_err()?;
                            }
                        }
                    }
                }
            }
        }

        Ok(existing)
    }

    fn entry_metadata_clear(&self, entry: &Entry) -> Result<usize, SqliteError> {
        use super::schema::FileMetadata::dsl::*;
        use super::schema::DirectoryMetadata::dsl::*;

        Ok(match entry {
            Entry::File(f) => {
                delete(FileMetadata.filter(file_id.eq(f.id)))
                    .execute(&self.conn)?
            }
            Entry::Directory(d) => {
                delete(DirectoryMetadata.filter(directory_id.eq(d.id)))
                    .execute(&self.conn)?
            }
        })
    }

    fn entries_metadata<'b, B: FromIterator<(Entry, Vec<(String, String)>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<B, SqliteError> {
        let mut f = Vec::<File>::new();
        let mut d = Vec::<Directory>::new();

        for e in entries {
            match e {
                Entry::File(file) => f.push(file.clone()),
                Entry::Directory(dir) => d.push(dir.clone())
            }
        };

        let fres = FileKeyValuePair::belonging_to(&f)
            .load::<FileKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&f)
            .into_iter()
            .zip(&f);

        let dres = DirectoryKeyValuePair::belonging_to(&d)
            .load::<DirectoryKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&d)
            .into_iter()
            .zip(&d);

        return Ok(
            fres.into_iter().map(|x| (Entry::File(*x.1), x.0.into_iter().map(|x| (x.key, x.value)).collect()))
            .chain(
                dres.into_iter().map(|x| (Entry::Directory(*x.1), x.0.into_iter().map(|x| (x.key, x.value)).collect()))
            )
            .collect()
        )
    }

    fn entries_metadata_get<'b, B: FromIterator<(Entry, String)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, k: &str) -> Result<B, SqliteError> {
        let mut f = Vec::<File>::new();
        let mut d = Vec::<Directory>::new();

        for e in entries {
            match e {
                Entry::File(file) => f.push(file.clone()),
                Entry::Directory(dir) => d.push(dir.clone())
            }
        };

        let fres = FileKeyValuePair::belonging_to(&f)
            .filter(super::schema::FileMetadata::dsl::key.eq(k))
            .load::<FileKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&f)
            .into_iter()
            .zip(&f);

        let dres = DirectoryKeyValuePair::belonging_to(&d)
            .filter(super::schema::DirectoryMetadata::dsl::key.eq(k))
            .load::<DirectoryKeyValuePair>(&self.conn).to_db_err()?
            .grouped_by(&d)
            .into_iter()
            .zip(&d);

        return Ok(
            fres.into_iter()
                .map(|x| 
                    (Entry::File(*x.1), x.0.into_iter().next().map(|y| y.value)))
            .chain(
                dres.into_iter()
                    .map(|x|
                        (Entry::Directory(*x.1), x.0.into_iter().next().map(|y| y.value)))
            )
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap()))
            .collect()
        )
    }

    fn entries_metadata_set<'b, B: FromIterator<(Entry, Option<String>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, k: &str, v: Option<&str>) -> Result<B, SqliteError> {
        self.conn.immediate_transaction(|| {
            let ret = Vec::<(Entry, Option<String>)>::new();

            for entry in entries {
                ret.push((*entry, self.entry_metadata_set(entry, k, v)?));
            }

            Ok(ret.into_iter().collect())
        }).to_db_err()
    }

    fn entries_metadata_clear<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, SqliteError> {
        use super::schema::FileMetadata::dsl::*;
        use super::schema::DirectoryMetadata::dsl::*;

        let mut f = Vec::<File>::new();
        let mut d = Vec::<Directory>::new();

        for entry in entries {
            match entry {
                Entry::File(ff) => f.push(*ff),
                Entry::Directory(dd) => d.push(*dd)
            }
        }

        let mut sz = delete(FileMetadata.filter(file_id.eq_any(f.iter().map(|x| x.id).into_vec())))
                    .execute(&self.conn)?;

        sz += delete(DirectoryMetadata.filter(directory_id.eq_any(d.iter().map(|x| x.id).into_vec())))
                    .execute(&self.conn)?;

        Ok(sz)
    }

    fn directory_entry(&self, d: &Directory, fname: &str) -> Result<Option<Entry>, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        let r1 = Directories.find(d.id)
                    .inner_join(Files)
                    .filter(filename.eq(fname))
                    .first::<(Directory, File)>(&self.conn)
                    .map(|x| x.1)
                    .optional().to_db_err()?;
        
        if let Some(r) = r1 {
            return Ok(Some(Entry::File(r)))
        }

        let target = Path::new(&d.path) / fname;

        Ok(Directories.filter(path.eq(target.str()))
            .first::<Directory>(&self.conn)
            .optional()
            .to_db_err()?
            .map(|x| Entry::Directory(x)))
    }

    fn directory_entries<B: FromIterator<Entry>>(&self, d: &Directory) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        let dirs = Directories.filter(path.like(&(d.path + "%")))
                    .load::<Directory>(&self.conn).to_db_err()?;

        let ids = dirs.iter().map(|x| x.id).into_vec();

        let files = Files.filter(directory_id.eq_any(ids))
                    .load::<File>(&self.conn).to_db_err()?;

        Ok (
            dirs.into_iter()
                .map(Entry::Directory)
            .chain(files.into_iter()
                    .map(Entry::File)
            ).collect()
        )
    }

    fn directory_entries_with_key<'b, B: FromIterator<Entry>>(&self, d: &Directory, k: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata;
        use super::schema::DirectoryMetadata;

        let dirs = Directories.filter(path.like(&(d.path + "%")))
                    .inner_join(DirectoryMetadata::table)
                    .filter(DirectoryMetadata::key.eq(k))
                    .load::<(Directory, DirectoryKeyValuePair)>(&self.conn).to_db_err()?
                    .into_iter()
                    .map(|x| x.0)
                    .into_vec();

        let ids = dirs.iter().map(|x| x.id).into_vec();

        let files = Files.filter(directory_id.eq_any(ids))
                     .inner_join(FileMetadata::table)
                     .filter(FileMetadata::key.eq(k))
                     .load::<(File, FileKeyValuePair)>(&self.conn).to_db_err()?
                     .into_iter()
                     .map(|x| x.0)
                     .into_vec();

        Ok (
            dirs.into_iter()
                .map(Entry::Directory)
            .chain(files.into_iter()
                    .map(Entry::File)
            ).collect()
        )
    }

    fn directory_entries_with_key_and_value<'b, B: FromIterator<Entry>>(&self, d: &Directory, k: &str, v: &str) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;
        use super::schema::FileMetadata;
        use super::schema::DirectoryMetadata;

        let dirs = Directories.filter(path.like(&(d.path + "%")))
                    .inner_join(DirectoryMetadata::table)
                    .filter(DirectoryMetadata::key.eq(k).and(DirectoryMetadata::value.eq(v)))
                    .load::<(Directory, DirectoryKeyValuePair)>(&self.conn).to_db_err()?
                    .into_iter()
                    .map(|x| x.0)
                    .into_vec();

        let ids = dirs.iter().map(|x| x.id).into_vec();

        let files = Files.filter(directory_id.eq_any(ids))
                     .inner_join(FileMetadata::table)
                     .filter(FileMetadata::key.eq(k).and(FileMetadata::value.eq(v)))
                     .load::<(File, FileKeyValuePair)>(&self.conn).to_db_err()?
                     .into_iter()
                     .map(|x| x.0)
                     .into_vec();

        Ok (
            dirs.into_iter()
                .map(Entry::Directory)
            .chain(files.into_iter()
                    .map(Entry::File)
            ).collect()
        )
    }

    fn get_entry(&self, p: &str) -> Result<Option<Entry>, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;
        
        let pat = Path::new(p);
        
        let d = Directories.filter(path.eq(pat.str()))
                    .first::<Directory>(&self.conn)
                    .optional()
                    .to_db_err()?;

        if let Some(d) = d {
            return Ok(Some(Entry::Directory(d)));
        }

        let (par, fnam) = (pat.parent(), pat.filename());

        Ok(Directories.filter(path.eq(par))
            .inner_join(Files)
            .filter(filename.eq(fnam))
            .first::<(Directory, File)>(&self.conn)
            .optional()
            .to_db_err()?
            .map(|x| Entry::File(x.1)))
    }

    fn get_entries<'b, B: FromIterator<Entry>, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<B, SqliteError> {
        use super::schema::Directories::dsl::*;
        use super::schema::Files::dsl::*;

        let pstrs = paths.into_vec();
        let parents = pstrs.iter().map(|x| Path::new(x).parent()).into_vec();
        let filenames = pstrs.iter().map(|x| Path::new(x).filename()).into_vec();
        let mut filename_parent_map = HashMap::<String, HashSet<String>>::new();

        for (f, d) in filenames.iter().zip(parents) {
            if let Some(set) = filename_parent_map.get_mut(f.to_owned()) {
                set.insert(d.to_owned());
            }
            else {
                let set = HashSet::new();
                set.insert(d.to_owned());
                filename_parent_map.insert((*f).to_owned(), set);
            }
        }

        let dirs = Directories.filter(path.eq_any(pstrs))
                    .load::<Directory>(&self.conn).to_db_err()?;

        let files = Directories.filter(path.eq_any(parents))
                    .inner_join(Files)
                    .filter(filename.eq_any(filenames))
                    .load::<(Directory, File)>(&self.conn).to_db_err()?
                    .into_iter()
                    .filter(|x| match filename_parent_map.get(&x.1.filename) {
                                    Some(set) => set.contains(&x.0.path),
                                    None => false
                                })
                    .map(|x| x.1)
                    .into_vec();
                        
        Ok(dirs.into_iter().map(Entry::Directory)
            .chain(files.into_iter().map(Entry::File))
            .collect())
    }

    fn add_directory(&self, p: &str) -> Result<(Directory, bool), SqliteError> {
        use super::schema::Directories::dsl::*;

        if let Some(s) = self.get_entry(p)? {
            match s {
                Entry::File(_) => return Err(ApplicationError(format!("A file with path '{}' already exists.", p))),
                Entry::Directory(d) => return Ok((d, false))
            }
        }

        let p = Path::new(p);

        let res = insert_into(Directories)
            .values(NewDirectory {
                path: p.str()
            })
            .execute(&self.conn).to_db_err()?;

        let dir = Directories.filter(path.eq(p.str()))
            .first::<Directory>(&self.conn).to_db_err()?;

        if res == 0 {
            return Ok((dir, false));
        }

        p.pop();
        while p.str().len() > 1 {
            let res = insert_into(Directories)
                .values(NewDirectory {
                    path: p.str()
                })
                .execute(&self.conn).to_db_err()?;
            
            if res == 0 {
                break;
            }

            p.pop();
        }

        return Ok((dir, true));
    }

    fn add_directories<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<usize, SqliteError> {
        use super::schema::Directories::dsl::*;

        let pat_cache = paths.map(Path::new).into_vec();
        let mut pat_refs = Vec::<&str>::new();

        for p in &pat_cache {
            let s = p.str();

            while s.len() > 1 {
                pat_refs.push(s);
                s = Path::parent_str(s);
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

        if let Some(s) = self.get_entry(p)? {
            match s {
                Entry::File(f) => return Ok((f, false)),
                Entry::Directory(_) => return Err(ApplicationError(format!("A directory with path '{}' already exists.", p))),
            }
        }

        let p = Path::new(p);

        let (dir, _) = self.add_directory(p.parent())?;

        let res = insert_or_ignore_into(Files)
            .values(NewFile {
                directory_id: dir.id,
                filename: p.filename(),
                hash: h
            })
            .execute(&self.conn).to_db_err()?;

        let file = Files.filter(filename.eq(p.filename()))
            .first::<File>(&self.conn).to_db_err()?;

        return Ok((file, res > 0));
    }

    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, SqliteError> {
        use super::schema::Files::dsl::*;
        use crate::linq::group_by::GroupBy;

        let paths = paths.into_iter().map(|e| (Path::new(e.0), e.1)).into_vec();

        let groups = paths.iter().group_by(|e| e.0.parent());

        let mut changes = 0;

        for (parent, tuples) in groups {
            let dir = match self.get_entry(parent)? {
                Some(e) => match e {
                    Entry::Directory(d) => d,
                    Entry::File(e) => return Err(ApplicationError(format!("The paths [{}] are invalid since their parent directory '{}' is listed as a file.", tuples.iter().map(|x| x.0.str()).into_vec().pretty_pathify(), parent)))
                },
                None => self.add_directory(parent)?.0
            };

            let new_files = tuples.into_iter().map(|t| NewFile {
                directory_id: dir.id,
                filename: t.0.filename(),
                hash: t.1
            }).collect::<Vec<NewFile>>();

            changes += insert_or_ignore_into(Files)
                .values(new_files)
                .execute(&self.conn).to_db_err()?;
        }

        return Ok(changes);
    }

    fn remove_entry(&self, entry: &Entry) -> Result<bool, SqliteError> {
        use super::schema::Files::dsl::*;
        use super::schema::Directories::dsl::*;

        Ok(match entry {
            Entry::File(f) => {
                delete(
                    Files.find(f.id)
                ).execute(&self.conn).to_db_err()?
            },
            Entry::Directory(d) => {
                if d.path == "/" {
                    0
                }
                else {
                    delete(
                        Directories.find(d.id)
                    ).execute(&self.conn).to_db_err()?
                }
            }
        } > 0)
    }

    fn remove_entries<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, SqliteError> {
        use super::schema::Files;
        use super::schema::Directories;

        let mut f = Vec::<File>::new();
        let mut d = Vec::<Directory>::new();

        for entry in entries {
            match entry {
                Entry::File(ff) => f.push(*ff),
                Entry::Directory(dd) => d.push(*dd)
            }
        }

        let mut sz = delete(
            Files::table.filter(Files::id.eq_any(f.iter().map(|x| x.id).into_vec()))
        )
        .execute(&self.conn)?;

        sz += delete(
            Directories::table.filter(Directories::id.eq_any(d.iter().map(|x| x.id).into_vec()))
        )
        .execute(&self.conn)?;

        Ok(sz)
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

    fn entry_metadata<B: FromIterator<(String, String)>>(&self, entry: &Entry) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.entry_metadata(entry)
    }

    fn entry_metadata_get(&self, entry: &Entry, key: &str) -> Result<Option<String>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.entry_metadata_get(entry, key)
    }

    fn entry_metadata_set(&self, entry: &Entry, key: &str, value: Option<&str>) -> Result<Option<String>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.entry_metadata_set(entry, key, value)
    }

    fn entry_metadata_clear(&self, entry: &Entry) -> Result<usize, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.entry_metadata_clear(entry)
    }

    fn entries_metadata<'b, B: FromIterator<(Entry, Vec<(String, String)>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.entries_metadata(entries)
    }

    fn entries_metadata_get<'b, B: FromIterator<(Entry, String)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, key: &str) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.entries_metadata_get(entries, key)
    }

    fn entries_metadata_set<'b, B: FromIterator<(Entry, Option<String>)>, I: Iterator<Item=&'b Entry>>(&self, entries: I, key: &str, value: Option<&str>) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.entries_metadata_set(entries, key, value)
    }

    fn entries_metadata_clear<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.entries_metadata_clear(entries)
    }

    fn directory_entry(&self, d: &Directory, filename: &str) -> Result<Option<Entry>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read)]);

        self.usd.directory_entry(d, filename)
    }

    fn directory_entries<B: FromIterator<Entry>>(&self, d: &Directory) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read)]);

        self.usd.directory_entries(d)
    }

    fn directory_entries_with_key<'b, B: FromIterator<Entry>>(&self, d: &Directory, key: &str) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.directory_entries_with_key(d, key)
    }

    fn directory_entries_with_key_and_value<'b, B: FromIterator<Entry>>(&self, d: &Directory, key: &str, value: &str) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read), (FileMeta, Read), (DirMeta, Read)]);

        self.usd.directory_entries_with_key_and_value(d, key, value)
    }

    fn get_entry(&self, path: &str) -> Result<Option<Entry>, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read)]);

        self.usd.get_entry(path)
    }

    fn get_entries<'b, B: FromIterator<Entry>, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<B, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Read), (Dir, Read)]);

        self.usd.get_entries(paths)
    }

    fn add_directory(&self, path: &str) -> Result<(Directory, bool), SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Write)]);

        self.usd.add_directory(path)
    }

    fn add_directories<'b, I: Iterator<Item=&'b str>>(&self, paths: I) -> Result<usize, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Write)]);

        self.usd.add_directories(paths)
    }

    fn add_file(&self, path: &str, hash: &[u8]) -> Result<(File, bool), SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Read)]);

        self.usd.add_file(path, hash)
    }

    fn add_files<'b, 'c, I: Iterator<Item=(&'b str, &'c [u8])>>(&self, paths: I) -> Result<usize, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Read)]);

        self.usd.add_files(paths)
    }

    fn remove_entry(&self, entry: &Entry) -> Result<bool, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Write), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.remove_entry(entry)
    }

    fn remove_entries<'b, I: Iterator<Item=&'b Entry>>(&self, entries: I) -> Result<usize, SqliteError> {
        use self::Lock::*;
        use self::LockMode::*;

        let _ = self.ctx(&[(File, Write), (Dir, Write), (FileMeta, Write), (DirMeta, Write)]);

        self.usd.remove_entries(entries)
    }
}
