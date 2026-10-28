use std::collections::HashMap;
use std::num::NonZeroUsize;

use crate::viewer::pipeline::ViewerPipelines;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ViewerId(NonZeroUsize);

/// Enables using multiple viewer panes.
pub struct ViewerRegistry {
    next_id: NonZeroUsize,
    viewers: HashMap<ViewerId, ViewerPipelines>
}

impl ViewerRegistry {
    pub fn new() -> Self {
        Self {
            next_id: NonZeroUsize::MIN,
            viewers: HashMap::new()
        }
    }

    pub fn insert(&mut self, pipeline: ViewerPipelines) -> ViewerId {
        let id = ViewerId(self.next_id);
        self.viewers.insert(id, pipeline);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    pub fn get(&self, id: ViewerId) -> Option<&ViewerPipelines> {
        self.viewers.get(&id)
    }

    pub fn get_mut(&mut self, id: ViewerId) -> Option<&mut ViewerPipelines> {
        self.viewers.get_mut(&id)
    }
}

impl Default for ViewerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
