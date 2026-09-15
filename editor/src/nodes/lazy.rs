use std::rc::Rc;

use szslib::{error::EncodingResult, lazy::Lazy};

use crate::pages::editor::VirtualNode;

struct LazyParserWrapper {
    raw: Vec<u8>,
    original: fn(&mut Vec<u8>) -> EncodingResult<T>,
}

/// This node initially contains only raw bytes
///
/// Upon user interaction, it will parse the bytes into usable content.
#[derive(Debug)]
pub struct LazyVirtualNode {
    pub inner: Lazy<Rc<dyn VirtualNode>, Vec<u8>>,
}

impl LazyVirtualNode {
    fn lazy_parser_wrapper(raw: Vec<u8>) -> EncodingResult<Rc<dyn VirtualNode>> {}

    pub fn from_lazy<T: VirtualNode>(lazy: Lazy<T, Vec<u8>>) -> Self {
        Self { inner }
    }
}

impl VirtualNode for LazyVirtualNode {
    fn label(&self) -> &str {
        &self.name
    }

    fn is_directory(&self) -> bool {
        true
    }

    fn children(&self) -> &[std::rc::Rc<dyn VirtualNode>] {
        todo!("children requested");
    }
}
