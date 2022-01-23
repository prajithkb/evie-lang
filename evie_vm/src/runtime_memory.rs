//! Stores the runtime values of objects (used for storing global variables)
use std::collections::HashMap;

use evie_memory::objects::Value;
pub type Values = Objects<Box<str>, Value>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Objects<K, V>
where
    K: std::hash::Hash + std::cmp::Eq,
{
    objects: HashMap<K, V>,
}

#[allow(dead_code)]
impl<K: std::hash::Hash + std::cmp::Eq, V> Objects<K, V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Objects {
            objects: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.objects.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.objects.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.objects.get_mut(key)
    }

    pub fn remove(&mut self, key: &K) {
        self.objects.remove(key);
    }
}
