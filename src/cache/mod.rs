use std::collections::HashMap;

use lru_cache::LruCache;

pub trait Cache {
    type Key: std::hash::Hash + Eq;
    type Value;

    fn insert(&mut self, k: Self::Key, v: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, k: &Self::Key) -> Option<Self::Value>;
    fn get(&mut self, k: &Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, k: &Self::Key) -> Option<&mut Self::Value>;

    fn contains_key(&mut self, k: &Self::Key) -> bool;
    // fn peek(&self, k: K) -> Option<&V>;
    // fn peek_contains_key(&self, k: K) -> bool;
}

impl<K, V> Cache for LruCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.insert(k, v)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        self.remove(k)
    }

    fn get(&mut self, k: &K) -> Option<&V> {
        self.get_mut(k).map(|v| &*v)
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.get_mut(k)
    }

    fn contains_key(&mut self, k: &K) -> bool {
        self.contains_key(k)
    }
}

impl<K, V> Cache for HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.insert(k, v)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        self.remove(k)
    }

    fn get(&mut self, k: &K) -> Option<&V> {
        self.get_mut(k).map(|v| &*v)
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.get_mut(k)
    }

    fn contains_key(&mut self, k: &K) -> bool {
        (&*self).contains_key(k)
    }
}
