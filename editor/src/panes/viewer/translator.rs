//! Translates between Wii models and wgpu ones.

use std::borrow::Cow;

use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MutexGuard, RwLockReadGuard};
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
        node::{VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatedVertexBuf {
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::VertexBufferLayout<'static>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TranslatedModel {
    // pub draw_calls: Cow<'a, [GxOpCode]>,
    pub vertex_buffers: Vec<TranslatedVertexBuf>,
}

impl TranslatedModel {
    /// Translates a [`Vertexbuf`] into a [`TranslatedVertexBuf`].
    #[tracing::instrument(skip_all, fields(buf_id, node_kind))]
    fn translate_buf<T, U, F>(
        device: &wgpu::Device,
        buf_id: VirtualNodeId,
        node_kind: VirtualNodeKind,
        node_map: &VirtualNodeMap,
        translate_fn: F,
    ) -> EditorResult<U>
    where
        T: 'static,
        F: FnOnce(&wgpu::Device, MappedRwLockReadGuard<'_, T>) -> U,
    {
        let Some(node) = node_map.get(buf_id) else {
            return Err(InvalidInputError {
                reason: format!("virtual node {buf_id} was not found"),
                ..Default::default()
            }
            .into());
        };

        tracing::trace!("Locking node {buf_id}");
        let node_lock = node.read();
        if node_lock.kind != node_kind {
            return Err(InvalidInputError {
                reason: format!(
                    "expected virtual node type {:?}, got {:?}",
                    node_kind, node_lock.kind
                ),
                ..Default::default()
            }
            .into());
        }

        // Map the lock to the downcasted body.
        let body = RwLockReadGuard::try_map(node_lock, |lock| {
            let Some(VirtualNodeBody {
                inspectable: Some(body),
                ..
            }) = lock.body.get()
            else {
                return None;
            };

            body.as_any().downcast_ref::<T>()
        })
        .map_err(|_| {
            EditorError::from(InvalidInputError {
                reason: String::from("failed to obtain node content for translation"),
                ..Default::default()
            })
        })?;

        tracing::trace!("Unlocked node {buf_id}");
        Ok(translate_fn(device, body))
    }

    #[tracing::instrument(skip_all, fields(node_id))]
    fn assemble_buffer(
        &mut self,
        device: &wgpu::Device,
        node_id: VirtualNodeId,
        node_map: &VirtualNodeMap,
    ) -> EditorResult<()> {
        let node = node_map.get(node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: String::from("did not find node {node_id}"),
                ..Default::default()
            })
        })?;

        tracing::trace!("Read locking node {node_id}");
        let node_lock = node.read();

        tracing::trace!(
            "Translating node `{}` of type `{:?}`",
            node_lock.label,
            node_lock.kind
        );
        let node_body = node_lock.body.get().expect("node body was deferred");

        match node_lock.kind {
            VirtualNodeKind::Vertices => {
                let translate_fn =
                    |device: &wgpu::Device, body: MappedRwLockReadGuard<'_, VertexBuf>| {
                        let bytes = body.vertices.as_bytes();
                        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("translated vertex buffer"),
                            contents: bytes,
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                        // These need to be stored as compile constants to give them a static lifetime.
                        const ATTRIBUTE_XY: &[wgpu::VertexAttribute] = &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }];

                        const ATTRIBUTES_XYZ: &[wgpu::VertexAttribute] = &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }];

                        let layout = wgpu::VertexBufferLayout {
                            array_stride: match body.vertices.ty() {
                                VertexPositionType::Xy => 2 * size_of::<f32>() as u64,
                                VertexPositionType::Xyz => 3 * size_of::<f32>() as u64,
                            },
                            attributes: match body.vertices.ty() {
                                VertexPositionType::Xy => ATTRIBUTE_XY,
                                VertexPositionType::Xyz => ATTRIBUTES_XYZ,
                            },
                            step_mode: wgpu::VertexStepMode::Vertex,
                        };

                        tracing::trace!(
                            "Created wgpu vertex buffer of size {} with format {:?}",
                            bytes.len(),
                            layout.attributes[0].format
                        );

                        TranslatedVertexBuf { buffer, layout }
                    };

                self.vertex_buffers.push(Self::translate_buf(
                    device,
                    node_id,
                    VirtualNodeKind::Vertices,
                    node_map,
                    translate_fn,
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
            tracing::trace!("Translating subdirectory `{:?}`", subdir_lock.kind);

            let subdir_body = subdir_lock
                .body
                .get()
                .expect("MDL0 subdirectory node was deferred");

            for &child in &subdir_body.children {
                model.assemble_buffer(device, child, node_map)?;
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
