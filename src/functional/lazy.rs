pub enum Lazy<'a, T> {
    Loaded(Box<T>),
    NotLoaded(Box<dyn FnOnce() -> T + 'a>)
}

impl<'a, T> Lazy<'a, T> {
    pub fn new(f: Box<dyn FnOnce() -> T + 'a>) -> Self {
        Lazy::NotLoaded(f)
    }

    pub fn get(&mut self) -> Box<T> {
        match self {
            Lazy::NotLoaded(f) => {
                let val = f();
                *self = Lazy::Loaded(Box::new(val));
                self.get()
            }
            Lazy::Loaded(v) => *v
        }
    }

    pub fn get_if_loaded(&self) -> Option<Box<T>> {
        match self {
            Lazy::NotLoaded(_) => None,
            Lazy::Loaded(v) => Some(*v)
        }
    }

    pub fn loaded(&self) -> bool {
        match self {
            Lazy::NotLoaded(_) => false,
            Lazy::Loaded(_) => true
        }
    }
}