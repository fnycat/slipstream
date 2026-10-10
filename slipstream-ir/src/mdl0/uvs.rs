use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{
    cursor::RefCursor,
    error::{CorruptionError, SlipstreamResult},
};

use crate::{
    encoding::ReadArrayExt,
    node::node::IrNodeType,
    util::{VectorDivisor, VertexFormat, deserialize_scalar_data, deserialize_vector_data},
    visitor::{Visitable, Visitor},
};
use crate::{gx::load_cp::FORMAT_DIVISOR_TOOLTIP, node::node::IrNode};
use crate::{
    mdl0::{SectionHeader, section::DeserializeContents},
    util::Box3,
};
use crate::{
    util::Box2,
    visitor::{VisitorContext, VisitorContextMut, VisitorContextNode, VisitorContextNodeMut},
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
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct UvBuffer {
    pub header: SectionHeader,
    /// The format of a single component in the buffer.
    ///
    /// The deserializer always converts the data to floats, but the original
    /// format is kept for reference.
    #[inspect(tooltip = "The format used for each component in a texture coordinate.")]
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
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    pub divisor: u8,
    /// The raw UV data.
    #[inspect(ignore)]
    pub uvs: UvBufData,
    pub bounding_volume: Box2,
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
        // Can be recomputed.
        let _stride = reader.read_u8()?;

        let uv_count = reader.read_u16::<BigEndian>()?;
        let bounding_volume = Box2::deserialize(reader)?;

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
            divisor,
            uvs,
            bounding_volume,
        })
    }
}
