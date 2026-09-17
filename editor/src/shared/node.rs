use std::{collections::HashMap, io::Cursor, path::PathBuf, rc::Rc};

use crate::{
    format::{
        arc,
        yaz0::{self, YAZ0_MAGIC},
    },
    shared::{
        util::RefCursor,
        r#virtual::{CacheStore, VirtualNode},
    },
};

/// Deserializes a possibly YAZ0-compressed file.
///
/// After decompressing, this forwards the call to [`deserialize_unknown_root`]
pub fn deserialize_maybe_compressed(
    mut reader: RefCursor<[u8]>,
    res_cache: &mut CacheStore,
    name: String,
) -> eyre::Result<VirtualNode> {
    // Is this file compressed?
    if &reader.get_ref()[..4] == YAZ0_MAGIC {
        // then decompress it.
        reader = RefCursor::new(Rc::from(yaz0::decompress(&mut reader)?));
    }

    deserialize_unknown_root(&mut reader, res_cache, name)
}

/// Deserializes an uncompressed file.
///
/// If the file may be compressed, call [`deserialize_maybe_compressed`].
///
/// This function works with OS level files, not files within archives.
pub fn deserialize_unknown_root(
    reader: &mut RefCursor<[u8]>,
    res_cache: &mut CacheStore,
    name: String,
) -> eyre::Result<VirtualNode> {
    let magic: &[u8; 4] = reader.get_ref()[..4]
        .try_into()
        .expect("array of size 4 does not have size 4?");

    let contents = match magic {
        &arc::ARC_MAGIC => arc::deserialize_virtual(reader, res_cache, name)?,
        // &brres::BRRES_MAGIC => deserialize_virtual_root_brres(reader, file_cache, name)?,
        _ => eyre::bail!(
            "unknown or unsupported file magic: `{}`",
            String::from_utf8_lossy(magic)
        ),
    };

    Ok(contents)
}
