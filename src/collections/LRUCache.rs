use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;
use std::borrow::Borrow;

struct Node<K: Hash + Eq, V> {
    key: Rc<K>,
    value: Rc<V>,
    next: Option<Rc<Node<K, V>>>,
    prev: Option<Rc<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    pub fn new(key: Rc<K>, value: Rc<V>) -> Self {
        Node {
            key,
            value,
            next: None,
            prev: None
        }
    }
}

pub struct LRUCache<K: Hash + Eq, V> {
    head: Option<Rc<Node<K, V>>>,
    tail: Option<Rc<Node<K, V>>>,
    hm: HashMap<Rc<K>, Rc<Node<K, V>>>,
    capacity: usize
}

impl<K: Hash + Eq, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LinkedHashMap { head: None, tail: None, hm: HashMap::new(), capacity }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let rck = Rc::new(key);
        let rcv = Rc::new(value);

        let new_node = Node::new(rck, rcv);
        let rcn = Rc::new(new_node);

        self.hm.insert(rck.clone(), rcn);

        if let Some(s) = &self.tail {
            let n = rcn.clone();

            s.next = Some(n.clone());
            n.prev = Some(s.clone());
            self.tail = Some(n);
        }
        else {
            self.head = Some(rcn.clone());
            self.tail = Some(rcn.clone());
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        match self.hm.get_mut(key).borrow() {
            Some(s) => {
                let prev = s.prev.clone();
                let next = s.next.clone();

                if let Some(ss) = prev {
                    ss.next = next.clone();
                }

                if let Some(ss) = next.clone() {
                    ss.prev = prev.clone();
                }

                s.prev = self.tail.clone();
                s.next = None;
                self.tail = Some((**s).clone());

                Some(s.borrow().value.borrow())
            },
            None => None
        }
    }

    pub fn len(&self) -> usize {
        self.hm.len()
    }
}