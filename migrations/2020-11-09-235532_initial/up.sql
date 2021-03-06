-- Your SQL goes here
CREATE TABLE IF NOT EXISTS Directories (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS Files (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directory_id INTEGER NOT NULL,
    filename TEXT NOT NULL UNIQUE,
    hash BLOB NOT NULL,
    FOREIGN KEY (directory_id) REFERENCES Directories(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX idx_dir_path ON Directories(path);
CREATE INDEX idx_file_name ON Directories(filename);
CREATE INDEX idx_files_hash ON Files(hash);

CREATE TABLE IF NOT EXISTS DirectoryMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directory_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(directory_id, key),
    FOREIGN KEY (directory_id) REFERENCES Directories(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS FileMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(file_id, key),
    FOREIGN KEY (file_id) REFERENCES Files(id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO Directories(path) VALUES ("/");

