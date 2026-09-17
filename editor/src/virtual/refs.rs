use crate::r#virtual::node::{VirtualNode, VirtualNodeRef};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroUsize;
use std::rc::{Rc, Weak};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VirtualNodeId(NonZeroUsize);

impl fmt::Display for VirtualNodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Maps between node IDs and the nodes that the IDs refer to.
///
/// This allows the editor to quickly access referenced nodes without
/// having to traverse the entire node tree.
pub struct VirtualRefCacheMap {
    next_id: usize,
    refs: HashMap<VirtualNodeId, Weak<RefCell<VirtualNode>>>,
}

pub type VirtualRefCache = Rc<RefCell<VirtualRefCacheMap>>;

/// Implements the [`VirtualRefCacheMap`] methods on `Rc<RefCell<VirtualRefCacheMap>>`.
///
/// These cannot be implemented on a foreign type directly, so a trait is used instead.
pub trait VirtualRefCacheExt {
    fn next_id(&self) -> VirtualNodeId;
    fn get(&self, id: VirtualNodeId) -> Option<VirtualNodeRef>;
    fn insert(&self, id: VirtualNodeId, node: Weak<RefCell<VirtualNode>>);
    fn prune_stale(&self);
}

impl VirtualRefCacheExt for Rc<RefCell<VirtualRefCacheMap>> {
    fn next_id(&self) -> VirtualNodeId {
        self.borrow_mut().next_id()
    }

    fn get(&self, id: VirtualNodeId) -> Option<VirtualNodeRef> {
        self.borrow().get(id)
    }

    fn insert(&self, id: VirtualNodeId, node: Weak<RefCell<VirtualNode>>) {
        self.borrow_mut().insert(id, node);
    }

    fn prune_stale(&self) {
        self.borrow_mut().prune_stale();
    }
}

impl VirtualRefCacheMap {
    pub fn new() -> VirtualRefCache {
        Rc::new(RefCell::new(Self {
            next_id: 1,
            refs: HashMap::new(),
        }))
    }

    pub fn next_id(&mut self) -> VirtualNodeId {
        self.next_id += 1;
        VirtualNodeId(NonZeroUsize::new(self.next_id - 1).unwrap())
    }

    /// Returns the node with the given ID.
    ///
    /// If the node does not exist (or became stale), `None` is returned.
    pub fn get(&self, id: VirtualNodeId) -> Option<VirtualNodeRef> {
        let weak = self.refs.get(&id)?;
        weak.upgrade()
    }

    /// Removes all stale nodes from the cache.
    pub fn prune_stale(&mut self) {
        self.refs.retain(|_, node| node.strong_count() > 0)
    }

    pub fn insert(&mut self, id: VirtualNodeId, node: Weak<RefCell<VirtualNode>>) {
        self.refs.insert(id, node);
    }
}
