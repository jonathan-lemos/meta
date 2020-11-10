-- Your SQL goes here
CREATE TABLE IF NOT EXISTS Directories (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS Files (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directoryId INTEGER NOT NULL,
    filename TEXT NOT NULL UNIQUE,
    hash VARCHAR(64) NOT NULL,
    FOREIGN KEY (directoryId) REFERENCES Directories(id)
);

CREATE INDEX idx_files_hash ON Files(hash);

CREATE TABLE IF NOT EXISTS DirectoryMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    directoryId INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(directoryId, key),
    FOREIGN KEY (directoryId) REFERENCES Directories(id)
);

CREATE TABLE IF NOT EXISTS FileMetadata (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    fileId INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(fileId, key),
    FOREIGN KEY (fileId) REFERENCES Files(id)
);

