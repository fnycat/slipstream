//! Translates between Wii models and wgpu ones.

use std::{any::Any, borrow::Cow};

use parking_lot::{
    ArcRwLockReadGuard, MappedMutexGuard, MappedRwLockReadGuard, MutexGuard, RwLockReadGuard,
};
use wgpu::util::DeviceExt;

use crate::{
    error::{EditorError, EditorResult, InvalidInputError},
    format::mdl0::{
        gx::{GxOpCode, draw::DrawOpCode},
        normals::{NormalBuf, NormalBufType},
        uvs::{UvBuf, UvDataType},
        vertices::{VertexBuf, VertexPositionType},
    },
    node::{
        node::{Inspectable, InspectableReadGuard, VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
};

#[derive(Clone)]
pub struct ModelBuffers {
    pub map: VirtualNodeMap,
    pub vertices: Vec<VirtualNodeId>,
    pub normals: Vec<VirtualNodeId>,
    pub colors: Vec<VirtualNodeId>,
    pub uvs: Vec<VirtualNodeId>,
    pub polygons: Vec<VirtualNodeId>,
}

impl ModelBuffers {
    pub fn new(map: VirtualNodeMap) -> Self {
        Self {
            map,
            vertices: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            polygons: Vec::new(),
        }
    }

    pub fn from_root(node: VirtualNodeId, map: VirtualNodeMap) -> EditorResult<Self> {
        let mut bufs = Self::new(map.clone());

        let node = map.get

        todo!()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TranslatedVertex {
    pub position: [f32; 3],
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TranslatedModel {
    pub vertices: Vec<TranslatedVertex>,
}

impl TranslatedModel {
    /// Resolves all indices in the shape's draw commands and constructs a new vertex buffer with
    /// interleaved positions/normals/etc that is compatible with wgpu.
    #[tracing::instrument(skip_all, fields(node_id))]
    fn resolve_shape(
        &mut self,
        device: &wgpu::Device,
        shape_node_id: VirtualNodeId,
        node_map: &VirtualNodeMap,
    ) -> EditorResult<()> {
        let node = node_map.get(shape_node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: String::from("did not find node {node_id}"),
                ..Default::default()
            })
        })?;

        tracing::trace!("Read locking node {shape_node_id}");
        let node_lock = node.read();

        tracing::trace!(
            "Translating node `{}` of type `{:?}`",
            node_lock.label,
            node_lock.kind
        );

        match node_lock.kind {
            VirtualNodeKind::Vertices => {
                self.vertex_buffers.push(Self::translate_buf(
                    device,
                    node_id,
                    VirtualNodeKind::Vertices,
                    node_map,
                    Self::translate_vertex_buf,
                )?);
            }
            VirtualNodeKind::Polygon => {
                self.polygons.push(Self::translate_buf(
                    device,
                    node_id,
                    VirtualNodeKind::Polygon,
                    node_map,
                    Self::translate_polygon_buf,
                )?);
            }
            k => tracing::warn!("unable to assemble {k:?}"),
        };

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(node_id))]
    pub fn from_node(
        device: &wgpu::Device,
        node_id: VirtualNodeId,
        node_map: &VirtualNodeMap,
    ) -> EditorResult<Self> {
        let mut model = Self::default();

        let root = node_map.get(node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: String::from("did not find root of MDL0 node"),
                ..Default::default()
            })
        })?;

        let root_lock = root.read();

        // Verify that this is an MDL0 node.
        if root_lock.kind != VirtualNodeKind::Mdl0Root {
            return Err(InvalidInputError {
                reason: format!(
                    "model translator expected Mdl0Root node, received {:?}",
                    root_lock.kind
                ),
                ..Default::default()
            }
            .into());
        }

        let root_body = root_lock.body.get().expect("MDL0 root was deferred");
        for &subdir in &root_body.children {
            let Some(subdir_node) = node_map.get(subdir) else {
                return Err(InvalidInputError {
                    reason: format!("MDL0 subdirectory node {subdir} did not exist"),
                    ..Default::default()
                }
                .into());
            };

            let subdir_lock = subdir_node.read();

            let subdir_body = subdir_lock
                .body
                .get()
                .expect("MDL0 subdirectory node was deferred");

            for &child in &subdir_body.children {
                model.resolve_shape(device, child, node_map)?;
            }
        }

        dbg!(&model);

        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgpuDrawCall {
    format: wgpu::VertexBufferLayout<'static>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgpuModel<'a> {
    pub draw_calls: Cow<'a, WgpuDrawCall>,
}
