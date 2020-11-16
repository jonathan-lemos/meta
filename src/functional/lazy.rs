pub enum LazyOnce<'a, T: ?Sized> {
    Loaded(Box<T>),
    NotLoaded(Option<Box<dyn FnOnce() -> T + 'a>>)
}

impl<'a, T> LazyOnce<'a, T> {
    pub fn new<F: FnOnce() -> T + 'a>(f: F) -> Self {
        LazyOnce::NotLoaded(Some(Box::new(f)))
    }

    pub fn new_loaded(v: T) -> Self {
        LazyOnce::Loaded(Box::new(v))
    }

    pub fn get(&mut self) -> &T {
        if let LazyOnce::NotLoaded(f) = self {
            let val = f.take().unwrap()();
            *self = LazyOnce::Loaded(Box::new(val));
        }

        if let LazyOnce::Loaded(v) = self {
            return v;
        }

        panic!("This Lazy<T> is not loaded even though it should be at this point.");
    }

    pub fn get_if_loaded(&self) -> Option<&T> {
        match self {
            LazyOnce::NotLoaded(_) => None,
            LazyOnce::Loaded(v) => Some(v)
        }
    }

    pub fn loaded(&self) -> bool {
        match self {
            LazyOnce::NotLoaded(_) => false,
            LazyOnce::Loaded(_) => true
        }
    }
}

pub enum Lazy<'a, T: ?Sized> {
    Loaded(Box<T>),
    NotLoaded(Box<dyn Fn() -> T + 'a>)
}

impl<'a, T> Lazy<'a, T> {
    pub fn new<F: Fn() -> T + 'a>(f: F) -> Self {
        Lazy::NotLoaded(Box::new(f))
    }

    pub fn new_loaded(v: T) -> Self {
        Lazy::Loaded(Box::new(v))
    }

    pub fn get(&self) -> &T {
        if let Lazy::NotLoaded(f) = self {
            let val = f();
            *self = Lazy::Loaded(Box::new(val));
        }

        if let Lazy::Loaded(v) = self {
            return v;
        }

        panic!("This Lazy<T> is not loaded even though it should be at this point.");
    }

    pub fn get_if_loaded(&self) -> Option<&T> {
        match self {
            Lazy::NotLoaded(_) => None,
            Lazy::Loaded(v) => Some(v)
        }
    }

    pub fn loaded(&self) -> bool {
        match self {
            Lazy::NotLoaded(_) => false,
            Lazy::Loaded(_) => true
        }
    }
}