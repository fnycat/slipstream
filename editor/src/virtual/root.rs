use std::rc::Rc;

use crate::error::{EditorResult, UnsupportedError};
use crate::r#virtual::node::VirtualNode;
use crate::r#virtual::refs::{VirtualNodeId, VirtualRefCache};
use crate::{
    format::{
        arc,
        yaz0::{self, YAZ0_MAGIC},
    },
    shared::util::RefCursor,
};

/// Deserializes a possibly YAZ0-compressed file.
///
/// After decompressing, this forwards the call to [`deserialize_unknown_root`]
pub fn deserialize_maybe_compressed(
    mut reader: RefCursor<[u8]>,
    ref_cache: &VirtualRefCache,
    name: String,
) -> EditorResult<VirtualNodeId> {
    // Is this file compressed?
    if &reader.as_remaining()[..4] == YAZ0_MAGIC {
        // then decompress it.
        reader = RefCursor::new(Rc::from(yaz0::decompress(&mut reader)?));
    }

    deserialize_unknown_root(&mut reader, ref_cache, name)
}

/// Deserializes an uncompressed file.
///
/// If the file may be compressed, call [`deserialize_maybe_compressed`].
///
/// This function works with OS level files, not files within archives.
pub fn deserialize_unknown_root(
    reader: &mut RefCursor<[u8]>,
    ref_cache: &VirtualRefCache,
    name: String,
) -> EditorResult<VirtualNodeId> {
    let magic: &[u8; 4] = reader.as_remaining()[..4]
        .try_into()
        .expect("array of size 4 does not have size 4?");

    let contents = match magic {
        &arc::ARC_MAGIC => arc::deserialize_virtual(reader, None, ref_cache, name)?,
        // &brres::BRRES_MAGIC => deserialize_virtual_root_brres(reader, file_cache, name)?,
        _ => {
            return Err(UnsupportedError {
                reason: format!(
                    "unknown or unsupported file magic: `{}`",
                    String::from_utf8_lossy(magic)
                ),
                location: Some(reader.position()),
            }
            .into());
        }
    };

    Ok(contents)
}
