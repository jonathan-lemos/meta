pub trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;
}

impl<T, I: Iterator<Item=T>> IntoVec<T> for I {
    fn into_vec(self) -> Vec<T> {
        self.collect()
    }
}