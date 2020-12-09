use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Borrow;

struct Node<K: Hash + Eq, V> {
    key: Arc<K>,
    value: Arc<V>,
    next: Option<Arc<Node<K, V>>>,
    prev: Option<Arc<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    pub fn new(key: Arc<K>, value: Arc<V>) -> Self {
        Node {
            key,
            value,
            next: None,
            prev: None
        }
    }
}

pub struct NodeIter<K: Hash + Eq, V> {
    cur: Option<Arc<Node<K, V>>>
}

impl<K, V> NodeIter<K, V> {
    pub fn new(head: Option<Arc<Node<K, V>>>) -> Self {
        NodeIter { cur: head }
    }
}

impl<'a, K, V> Iterator for NodeIter<K, V> {
    type Item = (&'a K, &'a V);

    fn next(&'a mut self) -> Option<Self::Item> {
        match &self.cur {
            Some(s) => {
                let (k, v) = (s.key.clone(), s.value.clone());
                self.cur = s.next.clone();
                Some((k.borrow(), v.borrow()))
            }
            None => None
        }
    }
}

pub struct MRUCache<K: Hash + Eq, V> {
    head: Option<Arc<Node<K, V>>>,
    tail: Option<Arc<Node<K, V>>>,
    hm: HashMap<Arc<K>, Arc<Node<K, V>>>,
    capacity: usize
}

impl<K: Hash + Eq, V> MRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self { head: None, tail: None, hm: HashMap::new(), capacity }
    }

    pub fn insert(&mut self, key: K, value: V) -> &V {
        let rck = Arc::new(key);
        let rcv = Arc::new(value);

        let new_node = Node::new(rck.clone(), rcv.clone());
        let rc_new_node = Arc::new(new_node);

        self.hm.insert(rck.clone(), rc_new_node.clone());

        if let Some(tail) = &self.tail {
            tail.next = Some(rc_new_node.clone());
            rc_new_node.prev = Some(tail.clone());
            self.tail = Some(rc_new_node.clone());
        }
        else {
            self.head = Some(rc_new_node.clone());
            self.tail = Some(rc_new_node.clone());
        }

        if self.len() > self.capacity {
            self.head = match &self.head {
                Some(s) => s.next.clone(),
                None => None
            };

            if let Some(s) = &self.head {
                s.prev = None;
            }
        }

        rcv.borrow()
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
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

    pub fn iter(&self) -> NodeIter<K, V> {
        NodeIter::new(self.head.clone())
    }

    pub fn len(&self) -> usize {
        self.hm.len()
    }
}

#[test]
fn test_insert() {
    let mut mruc = MRUCache::<i32, usize>::new(16);
    mruc.insert(1, 1);
    mruc.insert(2, 0);

    let result = mruc.iter().map(|x| (x.0.borrow().clone(), x.1.borrow().clone())).into_vec();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], (1, 1));
    assert_eq!(result[1], (2, 0));
}

#[test]
fn test_get() {
    let mut mruc = MRUCache::<i32, usize>::new(16);
    mruc.insert(1, 1);
    mruc.insert(2, 0);
    mruc.insert(3, 5);

    let v = mruc.get(&2);
    assert_eq!(v, Some(0));

    let result = mruc.iter().map(|x| (x.0.borrow().clone(), x.1.borrow().clone())).into_vec();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], (1, 1));
    assert_eq!(result[2], (3, 5));
    assert_eq!(result[1], (2, 0));
}
