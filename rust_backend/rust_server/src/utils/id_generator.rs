// Stolen from https://rocket.rs/v0.4/guide/pastebin/
use std::borrow::Cow;
use std::fmt;

use rand::{self, Rng};

/// Table to retrieve not base62 values from.
const BASE62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// A _probably_ unique paste ID.
pub struct UniqueId<'a>(Cow<'a, str>);

impl<'a> UniqueId<'a> {
    /// Generate a _probably_ unique ID with `size` characters. For readability,
    /// the characters used are from the sets [0-9], [A-Z], [a-z]. The
    /// probability of a collision depends on the value of `size` and the number
    /// of IDs generated thus far.
    pub fn new(size: usize) -> UniqueId<'static> {
        let mut id = String::with_capacity(size);
        let mut rng = rand::thread_rng();
        for _ in 0..size {
            id.push(BASE62[rng.gen::<usize>() % 36] as char);
        }
        debug!("Generated id: {}", &id);
        UniqueId(Cow::Owned(id))
    }
}

impl<'a> fmt::Display for UniqueId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
