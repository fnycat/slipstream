//! Translates between Wii models and wgpu ones.

use crate::viewer::translation::{IntermediateModel, ModelContents, VertexBoneData};
use slipstream_ir::gx::GxOpCode;
use slipstream_ir::gx::draw::{
    ColorData, DrawOpCode, InlineNormal, InlinePosition, NormalData, NormalIndex, OpVertex,
    PositionData,
};
use slipstream_ir::gx::load_indexed::IndexedLoad;
use slipstream_ir::mdl0::{ColorBuffer, MatrixId, NormalBuffer, Polygon, VertexBuffer};
use slipstream_shared::{SlipstreamResult, try_unwrap, verify};
use std::collections::HashMap;

/// A key that completely describes a vertex.
///
/// On GX hardware, all data is put into separate buffers for positions, normals, etc.
/// Modern-day GPUs prefer all this vertex data being in a single buffer with all data
/// interleaved instead of being stored separately.
///
/// In other words:
///
/// ```ignore
/// GX:
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct VertexKey {
    /// The matrix that transforms this vertex.
    pub mtx_id: Option<MatrixId>,
    pub position: VertexAttrKey,
    pub normal: VertexAttrKey,
    pub color0: VertexAttrKey,
}

/// Describes how the vertex data should be retrieved.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum VertexAttrKey {
    /// The data is not present. The translator just replaces the data with a default
    /// value in this case.
    ///
    /// It corresponds to the `NotPresent` vertex data in the polygon draw commands.
    #[default]
    NotPresent,
    /// The data is stored in a different buffer and should loaded using the index.
    ///
    /// This corresponds directly to *both* the `Index8` and `Index16` variants of data
    /// in the polygon draw commands.
    ///
    /// The buffer that the data is stored in is given by the `*_array_id` fields of the [`Polygon`].
    Indexed(u16),
    /// The data is stored in an inline buffer and should be loaded using the index.
    ///
    /// Some polygon draw commands might have their vertex data stored in the command itself
    /// instead of through an index. The translator keeps track of these using separate buffers,
    /// this is an index into those buffers.
    Inline(u16),
}

type VertexIndex = u16;

pub const MAX_BONE_INFLUENCES: usize = 4;

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct TranslatedVertex {
    /// The position of the vertex.
    pub position: [f32; 3],
    /// The normal of the vertex.
    pub normal: [f32; 3],
    pub color0: [f32; 4],
    pub bone_indices: [u32; MAX_BONE_INFLUENCES],
    pub bone_weights: [f32; MAX_BONE_INFLUENCES],
}

const XF_SLOT_COUNT: usize = 10;

#[derive(Default, Debug)]
pub struct XfRegisters {
    pub positions: [MatrixId; XF_SLOT_COUNT],
    pub normals: [MatrixId; XF_SLOT_COUNT],
}

#[derive(Default, Debug)]
pub struct InlineBuffers {
    /// Buffer of positions that are stored inline in the draw command.
    positions: Vec<glam::Vec3>,
    /// Buffer of normals that are stored inline in the draw command.
    normals: Vec<glam::Vec3>,
    /// Buffer of colors that are stored inline in the draw command.
    colors: Vec<glam::U8Vec4>,
}

impl InlineBuffers {
    pub fn positions(&self) -> &[glam::Vec3] {
        &self.positions
    }

    pub fn normals(&self) -> &[glam::Vec3] {
        &self.normals
    }

    pub fn colors(&self) -> &[glam::U8Vec4] {
        &self.colors
    }

    pub fn insert_position(&mut self, position: glam::Vec3) -> VertexAttrKey {
        self.positions.push(position);
        VertexAttrKey::Inline(self.positions.len() as u16 - 1)
    }

    pub fn insert_normal(&mut self, normal: glam::Vec3) -> VertexAttrKey {
        self.normals.push(normal);
        VertexAttrKey::Inline(self.normals.len() as u16 - 1)
    }

    pub fn insert_color(&mut self, color: glam::U8Vec4) -> VertexAttrKey {
        self.colors.push(color);
        VertexAttrKey::Inline(self.colors.len() as u16 - 1)
    }
}

#[derive(Default, Debug)]
pub struct IntermediatePolygon {
    /// Maps a vertex index to a location in `vertices`.
    pub map: HashMap<VertexKey, VertexIndex>,
    /// This will become the new index buffer.
    pub indices: Vec<VertexIndex>,
    /// This will become the new vertex buffer.
    pub vertices: Vec<TranslatedVertex>,
    /// Stores all inline data of polygon draw commands.
    pub inline: InlineBuffers,
}

impl ModelContents<'_> {
    fn try_inspect_positions<F, T>(&self, index: usize, inspect_fn: F) -> SlipstreamResult<T>
    where
        F: FnOnce(&VertexBuffer) -> SlipstreamResult<T>,
    {
        let key = try_unwrap!(self.vertices.get(index), "vertex buffer index out of range")?;

        self.try_inspect_inner(*key, inspect_fn)
    }

    fn try_inspect_normals<F, T>(&self, index: usize, inspect_fn: F) -> SlipstreamResult<T>
    where
        F: FnOnce(&NormalBuffer) -> SlipstreamResult<T>,
    {
        let key = try_unwrap!(self.normals.get(index), "normal buffer index out of range")?;

        self.try_inspect_inner(*key, inspect_fn)
    }

    fn try_inspect_colors<F, T>(&self, index: usize, inspect_fn: F) -> SlipstreamResult<T>
    where
        F: FnOnce(&ColorBuffer) -> SlipstreamResult<T>,
    {
        let key = try_unwrap!(self.colors.get(index), "color buffer index out of range")?;

        self.try_inspect_inner(*key, inspect_fn)
    }

    fn translate_vertex(
        &self,
        model: &IntermediateModel,
        scratch: &IntermediatePolygon,
        polygon: &Polygon,
        vertex_key: &VertexKey,
    ) -> SlipstreamResult<TranslatedVertex> {
        const POSITION_DEFAULT: glam::Vec3 = glam::Vec3::ZERO;
        const NORMAL_DEFAULT: glam::Vec3 = glam::vec3(0.0, 1.0, 0.0);
        const COLOR_DEFAULT: glam::U8Vec4 = glam::u8vec4(0, 255, 0, 255);

        let VertexBoneData {
            ids: indices,
            weights,
        } = self.translate_bones(model, scratch, polygon, vertex_key)?;

        let position = match vertex_key.position {
            VertexAttrKey::NotPresent => POSITION_DEFAULT,
            VertexAttrKey::Indexed(idx) => {
                self.try_inspect_positions(polygon.vertex_array_id as usize, |buf| {
                    try_unwrap!(
                        buf.get_xyz(idx as usize),
                        "vertex {idx} did not exist in vertex buffer"
                    )
                })?
            }
            VertexAttrKey::Inline(idx) => *scratch
                .inline
                .positions()
                .get(idx as usize)
                .expect("inline position index out of range"),
        }
        .to_array();

        let normal = match vertex_key.normal {
            VertexAttrKey::NotPresent => NORMAL_DEFAULT,
            VertexAttrKey::Indexed(idx) => {
                self.try_inspect_normals(polygon.normal_array_id as usize, |buf| {
                    try_unwrap!(
                        buf.get_normal(idx as usize),
                        "normal {idx} did not exist in normal buffer"
                    )
                })?
            }
            VertexAttrKey::Inline(idx) => *scratch
                .inline
                .normals()
                .get(idx as usize)
                .expect("inline normal index out of range"),
        }
        .to_array();

        let color0 = match vertex_key.color0 {
            VertexAttrKey::NotPresent => COLOR_DEFAULT,
            VertexAttrKey::Indexed(idx) => {
                self.try_inspect_colors(polygon.color_array_ids[0] as usize, |buf| {
                    try_unwrap!(
                        buf.get_rgba(idx as usize),
                        "color {idx} did not exist in color buffer"
                    )
                })?
            }
            VertexAttrKey::Inline(idx) => *scratch
                .inline
                .colors()
                .get(idx as usize)
                .expect("inline color0 index out of range"),
        };

        // Divide by 255 to get the colour components into the [0, 1] range.
        let color0 = (color0.as_vec4() / 255.0).to_array();

        Ok(TranslatedVertex {
            position,
            normal,
            color0,
            bone_indices: indices,
            bone_weights: weights,
        })
    }

    fn resolve_vertex(
        &self,
        model: &IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
        vertex: &OpVertex,
    ) -> SlipstreamResult<crate::viewer::translation::vertex::VertexIndex> {
        let mut vertex_key = VertexKey::default();

        let matrix_id = vertex
            .pn_matrix_index
            .map(|id| model.xf_slots.positions[id as usize]);

        vertex_key.mtx_id = matrix_id;

        match &vertex.position {
            PositionData::NotPresent => vertex_key.position = VertexAttrKey::NotPresent,
            PositionData::Index8(idx) => vertex_key.position = VertexAttrKey::Indexed(*idx as u16),
            PositionData::Index16(idx) => vertex_key.position = VertexAttrKey::Indexed(*idx),
            PositionData::Direct(x) => {
                let position = x.to_vec3();
                vertex_key.position = scratch.inline.insert_position(position);
            }
        }

        match &vertex.normals {
            NormalData::NotPresent => vertex_key.normal = VertexAttrKey::NotPresent,
            NormalData::Index8(idx) => match idx {
                NormalIndex::Single(x) => vertex_key.normal = VertexAttrKey::Indexed(*x as u16),
                NormalIndex::Triple(x) => vertex_key.normal = VertexAttrKey::Indexed(x[0] as u16),
            },
            NormalData::Index16(idx) => match idx {
                NormalIndex::Single(x) => vertex_key.normal = VertexAttrKey::Indexed(*x),
                NormalIndex::Triple(x) => vertex_key.normal = VertexAttrKey::Indexed(x[0]),
            },
            NormalData::Direct(x) => {
                let normal = x.to_vec3();
                vertex_key.normal = scratch.inline.insert_normal(normal);
            }
        }

        match &vertex.color0 {
            ColorData::NotPresent => vertex_key.color0 = VertexAttrKey::NotPresent,
            ColorData::Index8(idx) => vertex_key.color0 = VertexAttrKey::Indexed(*idx as u16),
            ColorData::Index16(idx) => vertex_key.color0 = VertexAttrKey::Indexed(*idx),
            ColorData::Direct(x) => {
                let color = x.to_rgba();
                vertex_key.color0 = scratch.inline.insert_color(color);
            }
        }

        // Check if the index has already been seen before. In case it has not
        // a new index will be generated.
        let vertex_index = match scratch.map.get(&vertex_key) {
            Some(&x) => x,
            None => {
                let translated = self.translate_vertex(model, scratch, polygon, &vertex_key)?;
                scratch.vertices.push(translated);

                let index =
                    scratch.vertices.len() as crate::viewer::translation::vertex::VertexIndex - 1;
                scratch.map.insert(vertex_key, index);
                index
            }
        };

        Ok(vertex_index)
    }

    fn resolve_triangle_list(
        &self,
        model: &IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
        vertices: &[OpVertex],
    ) -> SlipstreamResult<()> {
        scratch.indices.reserve(vertices.len());
        for vertex in vertices {
            let resolved = self.resolve_vertex(model, scratch, polygon, vertex)?;
            scratch.indices.push(resolved);
        }

        Ok(())
    }

    fn resolve_triangle_strip(
        &self,
        model: &IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
        vertices: &[OpVertex],
    ) -> SlipstreamResult<()> {
        let expanded_len = (vertices.len() - 2) * 3;

        scratch.indices.reserve(expanded_len);
        for (i, [v1, v2, v3]) in vertices.array_windows().enumerate() {
            let r1 = self.resolve_vertex(model, scratch, polygon, v1)?;
            let r2 = self.resolve_vertex(model, scratch, polygon, v2)?;
            let r3 = self.resolve_vertex(model, scratch, polygon, v3)?;

            if i % 2 == 0 {
                scratch.indices.extend([r1, r2, r3]);
            } else {
                scratch.indices.extend([r2, r1, r3]);
            }
        }

        Ok(())
    }

    fn load_position_slot(
        &self,
        model: &mut IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
        load: &IndexedLoad,
    ) -> SlipstreamResult<()> {
        let address = load.address();
        verify!(
            address <= 108 && address % 12 == 0,
            "Invalid address for position LoadXF command: {address}. Expected an address divisible by 12 and in the range 0..=108"
        );

        let transfer_count = load.transfer_count();
        verify!(
            transfer_count == 11,
            "Expected position load transfer count to equal 11, got {transfer_count}"
        );

        let slot_index = address / 12;
        let matrix_index = MatrixId(load.index());

        tracing::debug!(
            slot_index = ?slot_index,
            matrix_index = ?matrix_index,
            transfer_count
        );

        model.xf_slots.positions[slot_index as usize] = matrix_index;

        Ok(())
    }

    fn load_normal_slot(
        &self,
        model: &mut IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
        load: &IndexedLoad,
    ) -> SlipstreamResult<()> {
        todo!("normal indexed load");

        Ok(())
    }

    pub fn translate_polygon(
        &self,
        model: &mut IntermediateModel,
        scratch: &mut IntermediatePolygon,
        polygon: &Polygon,
    ) -> SlipstreamResult<()> {
        for call in &polygon.vertex_data_gx.commands {
            match call {
                GxOpCode::DrawTriangles(DrawOpCode { vertices }) => {
                    self.resolve_triangle_list(model, scratch, polygon, vertices)?
                }
                GxOpCode::DrawTriangleStrip(DrawOpCode { vertices }) => {
                    self.resolve_triangle_strip(model, scratch, polygon, vertices)?
                }
                GxOpCode::LoadIndexedPosition(load) => {
                    self.load_position_slot(model, scratch, polygon, load)?;
                }
                GxOpCode::LoadIndexedNormal(load) => {
                    // tracing::trace!("{load:#?}");
                }
                _ => {}
            }
        }

        Ok(())
    }
}
