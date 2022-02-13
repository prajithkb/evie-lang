//! Stores the runtime values of objects (used for storing global variables)

#[cfg(feature = "nan_boxed")]
use evie_memory::objects::nan_boxed::Value;
#[cfg(not(feature = "nan_boxed"))]
use evie_memory::objects::non_nan_boxed::Value;
use evie_memory::{cache::Cache, objects::GCObjectOf};
use rustc_hash::FxHashMap;
pub type Values = Objects<Value>;

/// This is an arbitrary number for now.
const ITEM_COUNT: usize = 1024;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Objects<V>
where
    V: Copy,
{
    objects: FxHashMap<GCObjectOf<Box<str>>, V>,
    cached_values: Cache<V>,
}

#[allow(dead_code)]
impl<V: Copy> Objects<V> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Objects {
            objects: FxHashMap::default(),
            cached_values: Cache::new(),
        }
    }

    pub fn insert(&mut self, key: GCObjectOf<Box<str>>, value: V) {
        self.cached_values.insert(key, value);
        // When we exceed the item count threshold, we drain it into the hashmap.
        if self.cached_values.size() >= ITEM_COUNT {
            let items = self.cached_values.drain_first(ITEM_COUNT);
            items.into_iter().for_each(|(k, v)| {
                self.objects.insert(k, v);
            });
        }
    }

    pub fn get(&mut self, key: GCObjectOf<Box<str>>) -> Option<V> {
        // if it is in the cache return
        if let Some(v) = self.cached_values.get(key) {
            Some(v)
            // fetch from the map, add to the cache and return
        } else if let Some(v) = self.objects.get(&key).copied() {
            self.cached_values.insert(key, v);
            Some(v)
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: GCObjectOf<Box<str>>) -> bool {
        self.cached_values.contains_key(key) || self.objects.contains_key(&key)
    }
}
