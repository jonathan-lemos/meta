use crate::linq::collectors::IntoVec;

pub trait PrettyPaths {
    fn pretty_pathify(&self) -> String;
}

impl PrettyPaths for Vec<&str> {
    fn pretty_pathify(&self) -> String {
        self.iter().map(|x| "'".to_owned() + x + "'").into_vec().join(", ")
    }
}

impl PrettyPaths for Vec<String> {
    fn pretty_pathify(&self) -> String {
        self.iter().map(|x| "'".to_owned() + x + "'").into_vec().join(", ")
    }
}