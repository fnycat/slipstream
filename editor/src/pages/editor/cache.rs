use std::{collections::HashMap, rc::Rc};

use crate::pages::editor::VirtualNode;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(usize);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

pub enum CacheEntry {
    Raw(Vec<u8>),
    Cached(Rc<dyn VirtualNode>),
}

pub struct FileCache {
    next_id: usize,
    map: HashMap<ResourceId, CacheEntry>,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            map: HashMap::new(),
        }
    }

    /// Allocates a new cache ID
    pub fn next_id(&mut self) -> ResourceId {
        self.next_id += 1;
        ResourceId(self.next_id - 1)
    }

    pub fn insert(&mut self, data: Vec<u8>) -> ResourceId {
        let id = self.next_id();
        self.map.insert(id, CacheEntry::Raw(data));

        id
    }
}
