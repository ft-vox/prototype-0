use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

type Link<Key> = Arc<Mutex<Node<Key>>>;

struct Node<Key> {
    key: Key,
    prev: Option<Link<Key>>,
    next: Option<Link<Key>>,
}

pub struct LRUCache<Key: Eq + Hash + Clone, Value: Clone> {
    map: HashMap<Key, (Value, Link<Key>)>,
    head: Option<Link<Key>>, // Least recently used
    tail: Option<Link<Key>>, // Most recently used
    capacity: usize,
}

impl<Key: Eq + Hash + Clone, Value: Clone> LRUCache<Key, Value> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn has(&self, key: &Key) -> bool {
        self.map.contains_key(key)
    }

    pub fn put(&mut self, key: Key, value: Value) {
        if self.map.contains_key(&key) {
            self.update_recent_usage(key.clone());
            self.map.get_mut(&key).unwrap().0 = value;
        } else {
            if self.map.len() >= self.capacity {
                self.evict_lru();
            }

            let new_node = Arc::new(Mutex::new(Node {
                key: key.clone(),
                prev: None,
                next: None,
            }));
            self.add_to_tail(new_node.clone());

            self.map.insert(key, (value, new_node));
        }
    }

    pub fn get(&mut self, key: &Key) -> Option<Value> {
        if let Some((value, _)) = self.map.get(key) {
            let result = Some(value.clone());
            self.update_recent_usage(key.clone());
            result
        } else {
            None
        }
    }

    fn update_recent_usage(&mut self, key: Key) {
        if let Some((_, node)) = self.map.get(&key) {
            self.move_to_tail(node.clone());
        }
    }

    fn evict_lru(&mut self) {
        if let Some(lru_node) = self.head.take() {
            let mut node_ref = lru_node.lock().unwrap();

            if let Some(next_node) = node_ref.next.take() {
                next_node.lock().unwrap().prev = None;
                self.head = Some(next_node);
            } else {
                self.tail = None;
            }

            self.map.remove(&node_ref.key);
        }
    }

    fn add_to_tail(&mut self, new_node: Link<Key>) {
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.lock().unwrap().next = Some(new_node.clone());
                new_node.lock().unwrap().prev = Some(old_tail);
                self.tail = Some(new_node);
            }
            None => {
                self.head = Some(new_node.clone());
                self.tail = Some(new_node);
            }
        }
    }

    fn move_to_tail(&mut self, node: Link<Key>) {
        {
            let mut node_ref = node.lock().unwrap();

            if self
                .tail
                .as_ref()
                .map(|t| Arc::ptr_eq(t, &node))
                .unwrap_or(false)
            {
                return;
            }

            if let Some(prev_node) = node_ref.prev.take() {
                prev_node.lock().unwrap().next = node_ref.next.clone();
            } else {
                self.head = node_ref.next.clone();
            }

            if let Some(next_node) = node_ref.next.take() {
                next_node.lock().unwrap().prev = node_ref.prev.clone();
            }
        }

        self.add_to_tail(node.clone());
    }

    pub fn set_capacity(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
        while self.map.len() > self.capacity {
            self.evict_lru();
        }
    }
}

// Implement the Drop trait to manually clean up the linked list and break the Rc cycle.
impl<Key: Eq + Hash + Clone, Value: Clone> Drop for LRUCache<Key, Value> {
    fn drop(&mut self) {
        // Clean up the linked list by traversing from the head
        while let Some(node) = self.head.take() {
            let next_node = node.lock().unwrap().next.take(); // Take the next node
            self.head = next_node; // Move to the next node
        }
        // At this point, all Rc references should be dropped and the list should be cleaned up
    }
}
