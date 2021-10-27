#![warn(clippy::pedantic)]

mod authority;
mod coder;
mod err;
mod ip;
mod statics;
mod uri;

#[macro_use]
extern crate lazy_static;

pub use crate::{
    uri::Uri,
    authority::Authority,
    err::Error,
    
};

#[cfg(test)]
struct TestCase<T> {
    case: T,
    expected: T,
}
