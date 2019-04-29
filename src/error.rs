use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

#[macro_export]
macro_rules! err {
    ($e:expr) => {{
        Err(format!($e).into())
    }};

    ($e:expr, $($es:expr),+) => {{
        Err(format!($e, $($es)+).into())
    }};
}
