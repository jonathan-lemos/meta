#![allow(non_snake_case)]

table! {
    Directories (id) {
        id -> Integer,
        path -> Text,
    }
}

table! {
    DirectoryMetadata (id) {
        id -> Integer,
        directory_id -> Integer,
        key -> Text,
        value -> Text,
    }
}

table! {
    FileMetadata (id) {
        id -> Integer,
        file_id -> Integer,
        key -> Text,
        value -> Text,
    }
}

table! {
    Files (id) {
        id -> Integer,
        directory_id -> Integer,
        filename -> Text,
        hash -> Binary,
    }
}

joinable!(DirectoryMetadata -> Directories (directory_id));
joinable!(FileMetadata -> Files (file_id));
joinable!(Files -> Directories (directory_id));

allow_tables_to_appear_in_same_query!(
    Directories,
    DirectoryMetadata,
    FileMetadata,
    Files,
);
