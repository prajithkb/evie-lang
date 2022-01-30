//! Cache module for caching expensive lookup (e.g global variables)

// use std::collections::HashMap;

use crate::objects::GCObjectOf;

/// A cache for values.
/// This is [Vec] based cache instead of a hashmap based one. The logic is to avoid hashing and random memory lookups
/// Mostly used for properties methods, and global variables
pub struct Cache<V: Copy> {
    cached_values: Vec<(GCObjectOf<Box<str>>, V)>,
    // values: HashMap<GCObjectOf<Box<str>>, Value>,
}

impl<V: Copy> Cache<V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Cache {
            cached_values: Vec::new(),
            // values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: GCObjectOf<Box<str>>, value: V) {
        let v = self.cached_values.iter_mut().find(|(k, _)| *k == key);
        if let Some((_, v)) = v {
            *v = value
        } else {
            self.cached_values.push((key, value))
        }
    }

    pub fn get(&self, key: GCObjectOf<Box<str>>) -> Option<V> {
        let r = self.cached_values.iter().find(|(k, _)| *k == key);
        r.map(|(_, v)| *v)
    }
}
