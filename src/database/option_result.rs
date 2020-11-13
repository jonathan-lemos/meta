pub trait OptionResult<T, E> {
    fn map_value<R, F: FnOnce(T) -> R>(self, func: F) -> Result<Option<R>, E>;
}

impl<T, E> OptionResult<T, E> for Result<Option<T>, E> {
    fn map_value<R, F: FnOnce(T) -> R>(self, func: F) -> Result<Option<R>, E> {
        match self {
            Err(e) => Err(e),
            Ok(o) => match o {
                Some(s) => Ok(Some(func(s))),
                None => Ok(None)
            }
        }
    }
}