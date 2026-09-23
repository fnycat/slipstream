//! Translates between Wii models and wgpu ones.

use std::borrow::Cow;

use crate::{
    error::EditorResult,
    format::mdl0::{
        gx::{GxOpCode, draw::DrawOpCode},
        normals::{NormalBuf, NormalBufType},
        uvs::{UvBuf, UvDataType},
        vertices::{VertexBuf, VertexPositionType},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct WiiModel<'a> {
    pub draw_calls: Cow<'a, [GxOpCode]>,
    pub vertex_buf: Cow<'a, VertexBuf>,
    pub normal_buf: Cow<'a, NormalBuf>,
    pub uv_buf: Cow<'a, UvBuf>,
}

impl<'a> WiiModel<'a> {
    /// Converts a Wii model into a WGPU one.
    ///
    /// All primitive topologies are converted to triangle lists. The Wii switches between types
    /// many times and also uses topologies that are not supported on modern hardware.
    pub fn translate(&self) -> EditorResult<WgpuModel<'static>> {
        let mut attributes = Vec::new();
        let mut offset = 0;
        let mut shader_location = 0;

        // Positions
        attributes.push(wgpu::VertexAttribute {
            offset,
            format: match self.vertex_buf.vertices.ty() {
                VertexPositionType::Xy => {
                    offset += 2 * size_of::<f32>() as u64;
                    wgpu::VertexFormat::Float32x2
                }
                VertexPositionType::Xyz => {
                    offset += 3 * size_of::<f32>() as u64;
                    wgpu::VertexFormat::Float32x3
                }
            },
            shader_location: 0,
        });
        shader_location += 1;

        // Either one normal or all three.
        attributes.push(wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset,
            shader_location: 1,
        });
        offset += 3 * size_of::<f32>() as u64;
        shader_location += 1;

        // Optional binormal + tangent
        if self.normal_buf.normals.ty() == NormalBufType::All {
            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset,
                shader_location,
            });
            offset += 3 * size_of::<f32>() as u64;
            shader_location += 1;

            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset,
                shader_location,
            });
            offset += 3 * size_of::<f32>() as u64;
            shader_location += 1;
        }

        // Color0
        attributes.push(wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint8x4,
            offset,
            shader_location,
        });
        offset += 4;
        shader_location += 1;

        // Color1
        attributes.push(wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint8x4,
            offset,
            shader_location,
        });
        offset += 4;
        shader_location += 1;

        attributes.push(wgpu::VertexAttribute {
            offset,
            format: match self.uv_buf.uvs.ty() {
                UvDataType::S => {
                    offset += size_of::<f32>() as u64;
                    wgpu::VertexFormat::Float32
                }
                UvDataType::St => {
                    offset += size_of::<f32>() as u64;
                    wgpu::VertexFormat::Float32x2
                }
            },
            shader_location,
        });

        let layout = wgpu::VertexBufferLayout {
            array_stride: offset,
            attributes: &attributes,
            step_mode: wgpu::VertexStepMode::Vertex,
        };

        dbg!(layout);

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
