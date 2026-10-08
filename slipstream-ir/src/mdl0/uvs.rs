use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_shared::{
    cursor::RefCursor,
    error::{CorruptionError, SlipstreamResult},
};

use crate::mdl0::{SectionHeader, section::DeserializeContents};
use crate::node::node::IrNode;
use crate::visitor::{
    VisitorContext, VisitorContextMut, VisitorContextNode, VisitorContextNodeMut,
};
use crate::{
    encoding::ReadArrayExt,
    node::node::IrNodeType,
    util::{VectorDivisor, VertexFormat, deserialize_scalar_data, deserialize_vector_data},
    visitor::{Visitable, Visitor},
};

const COMPONENTS_S: u32 = 0x00;
const COMPONENTS_ST: u32 = 0x01;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UvDataType {
    S,
    St,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UvBufData {
    S(Vec<f32>),
    St(Vec<glam::Vec2>),
}

impl UvBufData {
    pub fn ty(&self) -> UvDataType {
        match self {
            Self::S(_) => UvDataType::S,
            Self::St(_) => UvDataType::St,
        }
    }
}

/// Stores UVs (texture coordinates).
#[derive(Debug, Clone, PartialEq)]
pub struct UvBuffer {
    pub header: SectionHeader,
    /// The format of a single component in the buffer.
    ///
    /// The deserializer always converts the data to floats, but the original
    /// format is kept for reference.
    pub format: VertexFormat,
    /// The divisor is used to scale vectors at lower quality formats.
    ///
    /// For example if the format is [`Uint8`], then naively converting the
    /// UVs to floats would only give a range of 0-255 with whole integer intervals.
    ///
    /// The divisor is the power of 2 that is divided by the vertices to produce floats.
    /// I.e `float = uint8 / 2^divisor`.
    ///
    /// [`Uint8`]: VertexFormat::Uint8
    ///
    /// The deserializer always converts the data to floats, but the original
    /// format is kept for reference.
    pub divisor: u8,
    /// The amount of bytes between successive entries in the buffer.
    pub stride: u8,
    /// The raw UV data.
    pub uvs: UvBufData,
    /// First corner of the UV buffer's AABB.
    pub bounding_volume_min: glam::Vec2,
    /// Second corner of the UV buffer's AABB.
    pub bounding_volume_max: glam::Vec2,
}

impl UvBuffer {
    pub fn get_st(&self, index: usize) -> Option<glam::Vec2> {
        match &self.uvs {
            UvBufData::S(x) => x.get(index).map(|x| glam::vec2(*x, 0.0)),
            UvBufData::St(x) => x.get(index).copied(),
        }
    }
}

impl Visitable for UvBuffer {
    fn accept(&self, node: VisitorContextNode<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_uvs(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut<'_>,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_uvs_mut(VisitorContextMut::new(node, self))
    }
}

impl DeserializeContents for UvBuffer {
    const NAME: &str = "UVs";
    const KIND: IrNodeType = IrNodeType::UvBuffer;

    #[tracing::instrument(skip_all)]
    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let header = SectionHeader::deserialize(reader)?;
        let component_count = reader.read_u32::<BigEndian>()?;
        let format = VertexFormat::deserialize(reader)?;
        let divisor = reader.read_u8()?;
        let stride = reader.read_u8()?;

        let uv_count = reader.read_u16::<BigEndian>()?;
        let bounding_volume_min = glam::Vec2::from_array(reader.read_f32_array::<2, BigEndian>()?);
        let bounding_volume_max = glam::Vec2::from_array(reader.read_f32_array::<2, BigEndian>()?);

        reader.set_position(header.get_data_start());

        let uvs = match component_count {
            COMPONENTS_S => UvBufData::S(deserialize_scalar_data(
                reader,
                uv_count as usize,
                format,
                VectorDivisor::Custom(divisor),
            )?),
            COMPONENTS_ST => UvBufData::St(deserialize_vector_data::<2, glam::Vec2>(
                reader,
                uv_count as usize,
                format,
                VectorDivisor::Custom(divisor),
            )?),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid UV format: {component_count} (expected 0, 1)"),
                    ..Default::default()
                }
                .into());
            }
        };

        Ok(Self {
            header,
            format,
            stride,
            divisor,
            uvs,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}
