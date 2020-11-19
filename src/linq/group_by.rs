use std::hash::Hash;
use std::collections::HashMap;

pub trait GroupBy<K: Hash + Eq, T> {
    fn group_by<F: Fn(&T) -> K>(self, f: F) -> HashMap<K, Vec<T>>;
}

impl<K: Hash + Eq, T, I: Iterator<Item=T>> GroupBy<K, T> for I {
    fn group_by<F: Fn(&T) -> K>(self, f: F) -> HashMap<K, Vec<T>> {
        let mut ret = HashMap::<K, Vec<T>>::new();

        for t in self {
            let key = f(&t);

            match ret.get_mut(&key) {
                Some(s) => {
                    s.push(t);
                }
                None => {
                    ret.insert(key, vec![t]);
                }
            }
        }

        ret
    }
}