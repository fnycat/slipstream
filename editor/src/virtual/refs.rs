use crate::r#virtual::node::VirtualNode;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VirtualNodeId(NonZeroUsize);

impl fmt::Display for VirtualNodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type VirtualRefCache = Rc<RefCell<VirtualRefCacheMap>>;
pub type VirtualNodeRef = Rc<RefCell<VirtualNode>>;

/// Implements the [`VirtualRefCacheMap`] methods on `Rc<RefCell<VirtualRefCacheMap>>`.
///
/// These cannot be implemented on a foreign type directly, so a trait is used instead.
pub trait VirtualRefCacheExt {
    fn next_id(&self) -> VirtualNodeId;
    fn get(&self, id: VirtualNodeId) -> Option<Ref<'_, VirtualNodeRef>>;
    fn insert(&self, id: VirtualNodeId, node: VirtualNode);
}

impl VirtualRefCacheExt for Rc<RefCell<VirtualRefCacheMap>> {
    fn next_id(&self) -> VirtualNodeId {
        self.borrow_mut().next_id()
    }

    fn get(&self, id: VirtualNodeId) -> Option<Ref<'_, VirtualNodeRef>> {
        let borrow = self.borrow();
        Ref::filter_map(borrow, |map| map.get(id)).ok()
    }

    fn insert(&self, id: VirtualNodeId, node: VirtualNode) {
        self.borrow_mut().insert(id, node);
    }
}

/// Maps between node IDs and the nodes that the IDs refer to.
///
/// This allows the editor to quickly access referenced nodes without
/// having to traverse the entire node tree.
pub struct VirtualRefCacheMap {
    next_id: AtomicUsize,
    /// Nodes are stored in refcells to enable interior mutability.
    /// This ensures that nodes can be inserted into the cache while other nodes are being used.
    refs: HashMap<VirtualNodeId, Rc<RefCell<VirtualNode>>>,
}

impl VirtualRefCacheMap {
    pub fn new() -> VirtualRefCacheMap {
        Self {
            next_id: AtomicUsize::new(1),
            refs: HashMap::new(),
        }
    }

    pub fn next_id(&self) -> VirtualNodeId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        VirtualNodeId(NonZeroUsize::new(id).unwrap())
    }

    /// Returns the node with the given ID.
    ///
    /// If the node does not exist (or became stale), `None` is returned.
    pub fn get(&self, id: VirtualNodeId) -> Option<&VirtualNodeRef> {
        self.refs.get(&id)
    }

    pub fn insert(&mut self, id: VirtualNodeId, node: VirtualNode) {
        self.refs.insert(id, Rc::new(RefCell::new(node)));
    }
}
