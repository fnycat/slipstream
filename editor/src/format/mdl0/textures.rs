use byteorder::{BigEndian, ReadBytesExt};
use crate::error::EditorResult;
use crate::format::brres::IndexGroup;
use crate::format::encoding::{Deserialize, ReadArrayExt};
use crate::format::mdl0::gx::GxBytecode;
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap};
use crate::shared::util::RefCursor;

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    
}

impl Deserialize for Texture {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        todo!()
    }
}

#[tracing::instrument(skip_all, fields(parent_id))]
pub fn deserialize_virtual(reader: &mut RefCursor<[u8]>, parent_id: VirtualNodeId, node_map: &VirtualNodeMap) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut textures = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);

        let texture = Texture::deserialize(reader)?;
        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            parent: Some(parent_id),
            kind: VirtualNodeKind::Tevs,
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(texture))
            })
        };

        node_map.insert(id, node);
        textures.push(id);
    }

    Ok(VirtualNodeBody {
        children: textures,
        inspectable: None
    })
}