#![allow(clippy::needless_lifetimes)]
// See the crate level documentation for comments
use lifetime_abstractions::*;
use std::fmt::Write;
pub trait StreamingIterator {
    type Item: LtAbs;

    fn next<'a>(&'a mut self) -> Option<LtApply<'a, Self::Item>>;
}

struct Countdown {
    buf: String,
    count: usize,
}

impl StreamingIterator for Countdown {
    type Item = Lt!(for<'a> &'a str);

    fn next<'a>(&'a mut self) -> Option<&'a str> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        self.buf.clear();
        write!(&mut self.buf, "{}", self.count).unwrap();
        Some(&self.buf)
    }
}

fn main() {
    let mut countdown = Countdown {
        buf: String::new(),
        count: 20,
    };
    while let Some(item) = countdown.next() {
        println!("item: {:?}", item);
    }
}
