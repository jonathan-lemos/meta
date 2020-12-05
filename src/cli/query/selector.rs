use crate::cli::query::parse::OrQuery;

pub enum Selector {
    Files(Vec<String>),
    Query(OrQuery)
}