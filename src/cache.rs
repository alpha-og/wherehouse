use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct CachedItem<V> {
    value: V,
    fetched_at: Instant,
    ttl: Duration,
}

pub struct Cache<V> {
    store: Mutex<HashMap<String, CachedItem<V>>>,
    max_entries: usize,
    default_ttl: Duration,
}

impl<V: Clone> Cache<V> {
    pub fn new(max_entries: usize, default_ttl: Duration) -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            max_entries,
            default_ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        let mut store = self.store.lock().unwrap();
        let expired = store.get(key).map_or(false, |item| item.fetched_at.elapsed() >= item.ttl);
        if expired {
            store.remove(key);
            return None;
        }
        store.get(key).map(|item| item.value.clone())
    }

    pub fn set(&self, key: String, value: V) {
        self.set_with_ttl(key, value, self.default_ttl);
    }

    pub fn set_with_ttl(&self, key: String, value: V, ttl: Duration) {
        let mut store = self.store.lock().unwrap();
        if store.len() >= self.max_entries && !store.contains_key(&key) {
            if let Some(oldest) = store.iter().min_by_key(|(_, v)| v.fetched_at).map(|(k, _)| k.clone()) {
                store.remove(&oldest);
            }
        }
        store.insert(key, CachedItem { value, fetched_at: Instant::now(), ttl });
    }

    pub fn invalidate(&self, key: &str) {
        self.store.lock().unwrap().remove(key);
    }

    pub fn invalidate_all(&self) {
        self.store.lock().unwrap().clear();
    }

    pub fn invalidate_prefix(&self, prefix: &str) {
        self.store.lock().unwrap().retain(|k, _| !k.starts_with(prefix));
    }
}
