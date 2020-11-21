pub trait Collect<T> {
    fn into_vec(self) -> Vec<T>;
}

impl<T, I: Iterator<Item=T>> Collect<T> for I {
    fn into_vec(self) -> Vec<T> {
        self.collect()
    }
}