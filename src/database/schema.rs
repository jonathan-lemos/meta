table! {
    Directories (id) {
        id -> Integer,
        path -> Text,
    }
}

table! {
    DirectoryMetadata (id) {
        id -> Integer,
        directoryId -> Integer,
        key -> Text,
        value -> Text,
    }
}

table! {
    FileMetadata (id) {
        id -> Integer,
        fileId -> Integer,
        key -> Text,
        value -> Text,
    }
}

table! {
    Files (id) {
        id -> Integer,
        directoryId -> Integer,
        filename -> Text,
        hash -> Text,
    }
}

joinable!(DirectoryMetadata -> Directories (directoryId));
joinable!(FileMetadata -> Files (fileId));
joinable!(Files -> Directories (directoryId));

allow_tables_to_appear_in_same_query!(
    Directories,
    DirectoryMetadata,
    FileMetadata,
    Files,
);
