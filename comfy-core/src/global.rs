#![allow(dead_code)]
use crate::*;

pub struct Global<T> {
    value: Lazy<AtomicRefCell<T>>,
}
