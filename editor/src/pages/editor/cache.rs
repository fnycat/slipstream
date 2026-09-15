use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(usize);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

pub trait FileContent {}

pub struct FileCache {
    next_id: usize,
    map: HashMap<ResourceId, Rc<dyn FileContent>>,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            map: HashMap::new(),
        }
    }

    pub fn next_id(&mut self) -> ResourceId {
        self.next_id += 1;
        ResourceId(self.next_id - 1)
    }
}
