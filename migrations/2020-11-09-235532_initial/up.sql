-- Your SQL goes here
CREATE TABLE IF NOT EXISTS Directories (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS Files (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directory_id INTEGER NOT NULL,
    filename TEXT NOT NULL UNIQUE,
    hash VARCHAR(64) NOT NULL,
    FOREIGN KEY (directory_id) REFERENCES Directories(id)
);

CREATE INDEX idx_files_hash ON Files(hash);

CREATE TABLE IF NOT EXISTS DirectoryMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directory_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(directory_id, key),
    FOREIGN KEY (directory_id) REFERENCES Directories(id)
);

CREATE TABLE IF NOT EXISTS FileMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(file_id, key),
    FOREIGN KEY (file_id) REFERENCES Files(id)
);

