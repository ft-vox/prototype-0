use std::collections::{BTreeMap, VecDeque};

pub struct LRUCache<Key: Eq + Ord + Clone, Value: Clone> {
    pub map: BTreeMap<Key, Value>, // TODO: make it private
    queue: VecDeque<Key>,
    capacity: usize,
}

impl<Key: Eq + Ord + Clone, Value: Clone> LRUCache<Key, Value> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: BTreeMap::new(),
            queue: VecDeque::new(),
            capacity,
        }
    }

    pub fn has(&self, key: &Key) -> bool {
        self.map.contains_key(key)
    }

    pub fn put(&mut self, key: Key, value: Value) {
        self.map.insert(key.clone(), value);

        self.queue.push_back(key.clone());

        if self.queue.len() > self.capacity {
            if let Some(lru_key) = self.queue.pop_front() {
                self.map.remove(&lru_key);
            }
        }
        self.update_recent_usage(key.clone());
    }

    pub fn get(&mut self, key: &Key) -> Option<Value> {
        let result = self.map.get(key).cloned();
        if result.is_some() {
            self.update_recent_usage(key.clone());
        }
        result
    }

    fn update_recent_usage(&mut self, key: Key) {
        if let Some(pos) = self.queue.iter().position(|x| *x == key) {
            self.queue.remove(pos);
        }
        self.queue.push_back(key);
    }

    pub fn set_capacity(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
        while self.queue.len() > self.capacity {
            if let Some(lru_key) = self.queue.pop_front() {
                self.map.remove(&lru_key);
            }
        }
    }
}
