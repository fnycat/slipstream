//! Translates between Wii models and wgpu ones.

use std::{any::Any, borrow::Cow, collections::HashMap};

use bytemuck::Zeroable;
use parking_lot::{
    ArcRwLockReadGuard, MappedMutexGuard, MappedRwLockReadGuard, MutexGuard, RwLockReadGuard,
};
use wgpu::util::DeviceExt;

use crate::{
    error::{EditorError, EditorResult, InvalidInputError},
    format::mdl0::{
        gx::{
            GxOpCode,
            draw::{DirectNormal, DirectPosition, DrawOpCode, NormalData, OpVertex, PositionData},
        },
        normals::{NormalBuf, NormalBufType},
        shapes::Shape,
        uvs::{UvBuf, UvDataType},
        vertices::{VertexBuf, VertexPositionType},
    },
    node::{
        node::{Inspectable, InspectableReadGuard, VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
};
use crate::format::mdl0::gx::draw::NormalIndex;

/// A vertex with all data interleaved.
///
/// While the Wii stores every single buffer type separately and uses separate index buffer,
/// this is not actually possible on modern hardware. We solve this by creating wgpu compatible
/// vertex buffers with all data interleaved. All buffer indices are resolved and turned into a single index buffer.
///
/// Some data might be upgraded to higher quality formats automatically as this prevents having to support
/// many different formats in the shaders.
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InterleavedVertex {
    /// Position data can be stored in either 2 or 3 component format.
    /// 2 component formats are automatically upgraded to 3 components by setting the Z component to 0.
    position: [f32; 3],
    /// The MDL0 file format either stores a single normal or the normal + tangent + binormal.
    /// The translator discards the binormal and always stores the normal and tangent. If the
    /// normal buffer did not contain tangents, a dummy value will be used.
    ///
    /// Discarding the binormal reduces buffer size, since the binormal can easy be calculated from a cross
    /// product between the normal and tangent.
    normals: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct TranslatedBuffers {}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum IndexAttrKey {
    /// The attribute was stored as an index into a buffer.
    Physical(usize),
    /// The attribute was stored inline. It has been added to the direct map
    /// and a new synthetic index into the map has been generated.
    Synthetic(usize),
    /// The attribute was marked as not present.
    #[default]
    NotPresent,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct IndexKey {
    pub position: IndexAttrKey,
    pub normals: IndexAttrKey,
}

#[derive(Debug, Default, Clone)]
pub struct DirectScratchBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

impl DirectScratchBuffers {
    pub fn insert_position(&mut self, position: DirectPosition) -> IndexAttrKey {
        self.positions.push(position.to_xyz());
        IndexAttrKey::Synthetic(self.positions.len() - 1)
    }

    pub fn insert_normal(&mut self, normal: DirectNormal) -> IndexAttrKey {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VertexIndex(pub usize);

#[derive(Debug, Default, Clone)]
pub struct TranslationScratchData {
    /// Maps MDL0's multi-indices into a single index pointing into the `vertices` array.
    ///
    /// This map ensures that we do not duplicate vertices and allows for easy
    /// building of an index buffer later.
    pub vertex_map: HashMap<IndexKey, VertexIndex>,
    pub vertices: Vec<InterleavedVertex>,
    /// The actual new index buffer,
    /// These indices point into the `vertices` array.
    pub indices: Vec<VertexIndex>,
    /// Direct draw call values are stored in here. They are given a new separate ID
    /// that is used to refer to them in the vertex map.
    pub direct_data: DirectScratchBuffers,
}

impl TranslationScratchData {
    /// Inserts a translated vertex into the scratch buffer.
    pub fn insert_vertex(&mut self, vertex: InterleavedVertex) -> usize {
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }

    /// Translates the generated scratch data into usable wgpu buffers.
    pub fn generate_buffers(&self, device: &wgpu::Device) -> EditorResult<TranslatedBuffers> {
        dbg!(&self);

        todo!()
    }
}

#[derive(Clone)]
pub struct ModelBuffers {
    pub map: VirtualNodeMap,
    pub positions: Vec<VirtualNodeId>,
    pub normals: Vec<VirtualNodeId>,
    pub colors: Vec<VirtualNodeId>,
    pub uvs: Vec<VirtualNodeId>,
    pub shapes: Vec<VirtualNodeId>,
}

const POSITION_DEFAULT: [f32; 3] = [0.0; 3];
const NORMAL_DEFAULT: [f32; 3] = [0.0, 1.0, 0.0];

impl ModelBuffers {
    pub fn new(map: VirtualNodeMap) -> Self {
        Self {
            map,
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            shapes: Vec::new(),
        }
    }

    /// Creates a new interleaved vertex by resolving all indices in the key.
    fn generate_interleaved_vertex(
        &self,
        shape: &Shape,
        index_key: &IndexKey,
        scratch: &TranslationScratchData,
    ) -> EditorResult<InterleavedVertex> {
        let position = match index_key.position {
            IndexAttrKey::NotPresent => POSITION_DEFAULT,
            IndexAttrKey::Physical(idx) => {
                let position_buffer = self.get_vertices(shape.vertex_array_id as usize)?;
                position_buffer
                    .get_xyz(idx)
                    .expect("vertex did not exist in position buffer")
            }
            IndexAttrKey::Synthetic(idx) => *scratch
                .direct_data
                .positions
                .get(idx)
                .expect("position did not exist in direct data buffer"),
        };

        let normals = match index_key.normals {
            IndexAttrKey::NotPresent => NORMAL_DEFAULT,
            IndexAttrKey::Physical(idx) => {
                let normal_buffer = self.get_normals(shape.normal_array_id as usize)?;
                normal_buffer.get_normal(idx).expect("vertex did not exist in normal buffer")
            }
            IndexAttrKey::Synthetic(idx) => *scratch
                .direct_data
                .normals
                .get(idx)
                .expect("normal did not exist in direct data buffer"),
        };

        Ok(InterleavedVertex { position, normals })
    }

    /// Resolves the given vertex, returning its assigned index.
    fn resolve_vertex(
        &self,
        shape: &Shape,
        vertex: &OpVertex,
        scratch: &mut TranslationScratchData,
    ) -> EditorResult<VertexIndex> {
        let mut index_key = IndexKey::default();

        // Generate the index.

        match &vertex.position {
            PositionData::NotPresent => index_key.position = IndexAttrKey::NotPresent,
            PositionData::Index8(idx) => index_key.position = IndexAttrKey::Physical(*idx as usize),
            PositionData::Index16(idx) => {
                index_key.position = IndexAttrKey::Physical(*idx as usize)
            }
            PositionData::Direct(x) => {
                let sid = scratch.direct_data.insert_position(*x);
                index_key.position = sid;
            }
        }

        match &vertex.normals {
            NormalData::NotPresent => index_key.normals = IndexAttrKey::NotPresent,
            NormalData::Index8(idx) => {
                match idx {
                    NormalIndex::Single(idx) => index_key.normals = IndexAttrKey::Physical(*idx as usize),
                    NormalIndex::Triple(idxs) => todo!("multi normal indices")
                }
            },
            NormalData::Index16(idx) => {
                match idx {
                    NormalIndex::Single(idx) => index_key.normals = IndexAttrKey::Physical(*idx as usize),
                    NormalIndex::Triple(idxs) => todo!("multi normal indices")
                }
            },
            NormalData::Direct(x) => {
                let sid = scratch.direct_data.insert_normal(x.clone());
                index_key.normals = sid;
            }
        }

        // Check if the index has already been seen before. In case it hasn't
        // a new index will be generated.
        let translated_index = match scratch.vertex_map.get(&index_key) {
            // If the given multi index has already been seen before, load its translated
            // index.
            Some(&x) => x,
            // Otherwise insert the index and its vertex data into the map.
            None => {
                let interleaved = self.generate_interleaved_vertex(shape, &index_key, scratch)?;

                scratch.vertices.push(interleaved);
                let new_index = VertexIndex(scratch.vertices.len() - 1);

                scratch.vertex_map.insert(index_key.clone(), new_index);

                new_index
            }
        };

        Ok(translated_index)
    }

    /// Resolves a list of triangles.
    ///
    /// This is the most straightforward topology as it maps straight to what the editor's
    /// graphics pipeline uses.
    #[tracing::instrument(skip_all, fields(vertex_count = vertices.len()))]
    fn resolve_triangle_list(
        &self,
        shape: &Shape,
        vertices: &[OpVertex],
        scratch: &mut TranslationScratchData,
    ) -> EditorResult<()> {
        tracing::trace!("Resolving a {} triangle list", vertices.len());

        scratch.indices.reserve(vertices.len());
        for vertex in vertices {
            let resolved = self.resolve_vertex(shape, vertex, scratch)?;
            scratch.indices.push(resolved);
        }

        Ok(())
    }

    /// Resolves a triangle strip topology.
    ///
    /// While wgpu also supports triangle strip topologies, the topology is converted into a
    /// basic triangle list. Since a render pipeline is only capable of rendering a single toplogy, this
    /// reduces the amount of pipelines that have to be created.
    #[tracing::instrument(skip_all, fields(vertex_count = vertices.len()))]
    fn resolve_triangle_strip(
        &self,
        shape: &InspectableReadGuard<Shape>,
        vertices: &[OpVertex],
        scratch: &mut TranslationScratchData,
    ) -> EditorResult<()> {
        tracing::trace!("Resolving a {} triangle strip", vertices.len());

        let expanded_len = (vertices.len() - 2) * 3;
        scratch.indices.reserve(expanded_len);

        for (i, [v1, v2, v3]) in vertices.array_windows().enumerate() {
            let r1 = self.resolve_vertex(shape, v1, scratch)?;
            let r2 = self.resolve_vertex(shape, v2, scratch)?;
            let r3 = self.resolve_vertex(shape, v3, scratch)?;

            if i % 2 == 0 {
                // Even triangles should keep their original winding order.
                scratch.indices.extend([r1, r2, r3]);
            } else {
                // Odd triangle should have their first two vertices reversed.
                scratch.indices.extend([r2, r1, r3]);
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(shape = shape.label(), id = shape.id().into_inner()))]
    fn resolve_shape(
        &self,
        shape: InspectableReadGuard<Shape>,
        out: &mut TranslationScratchData,
    ) -> EditorResult<()> {
        tracing::trace!(
            "Resolving {} draw calls",
            shape.vertex_data_gx.commands.len(),
        );

        for call in &shape.vertex_data_gx.commands {
            match call {
                GxOpCode::DrawTriangles(DrawOpCode { vertices }) => {
                    self.resolve_triangle_list(&shape, vertices, out)?;
                }
                GxOpCode::DrawTriangleStrip(DrawOpCode { vertices }) => {
                    self.resolve_triangle_strip(&shape, vertices, out)?;
                }
                _ => tracing::error!("unsupported opcode: {call:?}"),
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn resolve_shapes(&self) -> EditorResult<TranslationScratchData> {
        let mut translated = TranslationScratchData::default();

        tracing::trace!("Resolving {} shapes", self.shapes.len());
        for &shape_id in &self.shapes {
            let shape = self.map.get_inspectable::<Shape>(shape_id).ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("virtual node {shape_id} did not exist"),
                    ..Default::default()
                })
            })?;

            self.resolve_shape(shape, &mut translated)?;
        }

        Ok(translated)
    }

    #[tracing::instrument(skip_all, fields(node))]
    pub fn from_root(node: VirtualNodeId, map: VirtualNodeMap) -> EditorResult<Self> {
        tracing::trace!("Constructing model buffer block from MDL0 file");

        let mut bufs = Self::new(map.clone());

        let root_children = map.get_children(node).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!("virtual node {node} did not exist"),
                ..Default::default()
            })
        })?;

        for &section_id in &root_children {
            let children = map
                .get_children(section_id)
                .expect("unable to find children of node");

            for &child in &children {
                let handle = map.get(child).expect("did not find child node");
                let guard = handle.read();

                match guard.kind {
                    VirtualNodeKind::Vertices => bufs.positions.push(child),
                    VirtualNodeKind::Normals => bufs.normals.push(child),
                    VirtualNodeKind::Colors => bufs.colors.push(child),
                    VirtualNodeKind::Uvs => bufs.uvs.push(child),
                    VirtualNodeKind::Shape => bufs.shapes.push(child),
                    _ => {}
                }
            }
        }

        tracing::trace!("Constructed model buffer block successfully");
        Ok(bufs)
    }

    pub fn get_vertices(&self, index: usize) -> EditorResult<InspectableReadGuard<VertexBuf>> {
        let node_id = *self.positions.get(index).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!("vertex buffer {index} does not exist"),
                ..Default::default()
            })
        })?;

        self.map
            .get_inspectable::<VertexBuf>(node_id)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("vertex buffer {index} was not found"),
                    ..Default::default()
                })
            })
    }

    pub fn get_normals(&self, index: usize) -> EditorResult<InspectableReadGuard<NormalBuf>> {
        let node_id = *self.normals.get(index).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!("normal buffer {index} does not exist"),
                ..Default::default()
            })
        })?;

        self.map
            .get_inspectable::<NormalBuf>(node_id)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("normal buffer {index} was not found"),
                    ..Default::default()
                })
            })
    }
}
