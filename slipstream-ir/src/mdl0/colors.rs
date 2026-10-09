use std::ops::ControlFlow;

use bitfield_struct::bitenum;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::Inspect;
use slipstream_shared::{
    cursor::{MutCursor, RefCursor},
    error::{CorruptionError, InvalidInputError, SlipstreamError, SlipstreamResult},
    verify,
};

use crate::node::node::IrNode;
use crate::visitor::{
    VisitorContext, VisitorContextMut, VisitorContextNode, VisitorContextNodeMut,
};
use crate::{
    encoding::ReadArrayExt,
    node::node::IrNodeType,
    visitor::{Visitable, Visitor},
};
use crate::{
    img::deserialize_color,
    mdl0::{SectionHeader, section::DeserializeContents},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorComponents {
    Rgb = 0x00,
    Rgba = 0x01,
}

impl TryFrom<u32> for ColorComponents {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Rgb,
            1 => Self::Rgba,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid color format: {value} (expected 0, 1)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl ColorComponents {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

/// Describes the format of the color.
#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum ColorFormat {
    /// 16 total bits.
    ///
    /// Red: 5 bits;
    /// Green: 6 bits;
    /// Blue: 5 bits;
    /// Alpha: N/A
    Rgb565 = 0x00,
    /// 24 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: N/A
    Rgb24 = 0x01,
    /// 32 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: 8 bits (discarded)
    Rgbx32 = 0x02,
    /// 16 total bits.
    ///
    /// Red: 4 bits;
    /// Green: 4 bits;
    /// Blue: 4 bits;
    /// Alpha: 4 bits
    Rgba16 = 0x03,
    /// 24 total bits.
    ///
    /// Red: 6 bits;
    /// Green: 6 bits;
    /// Blue: 6 bits;
    /// Alpha: 6 bits
    Rgba24 = 0x04,
    /// 32 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: 8 bits
    Rgba32 = 0x05,
    /// This is a fallback for `bitenum`, it should never be used and is not visible to users.
    #[inspect(ignore)]
    #[fallback]
    Invalid,
}

impl ColorFormat {
    /// The stride in bytes of the format.
    #[inline]
    pub const fn stride(&self) -> u32 {
        match self {
            Self::Rgb565 => 2,
            Self::Rgb24 => 3,
            Self::Rgbx32 => 4,
            Self::Rgba16 => 2,
            Self::Rgba24 => 3,
            Self::Rgba32 => 4,
            Self::Invalid => 0,
        }
    }
}

impl TryFrom<u32> for ColorFormat {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Rgb565,
            0x01 => Self::Rgb24,
            0x02 => Self::Rgbx32,
            0x03 => Self::Rgba16,
            0x04 => Self::Rgba24,
            0x05 => Self::Rgba32,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid color format: {value} (expected 0-5)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl ColorFormat {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        verify!(
            *self != ColorFormat::Invalid,
            "cannot serialize an Invalid color format"
        );

        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

/// A buffer of vertex colors.
///
/// This data cannot be used on its own. It is indexed into by the indices in the shape draw commands.
#[derive(Debug, Clone)]
pub struct ColorBuffer {
    header: SectionHeader,
    components: ColorComponents,
    format: ColorFormat,
    stride: u8,
    colors: Vec<glam::U8Vec4>,
}

impl ColorBuffer {
    #[inline]
    pub fn get_rgba(&self, index: usize) -> Option<glam::U8Vec4> {
        self.colors.get(index).copied()
    }
}

impl Visitable for ColorBuffer {
    fn accept(&self, node: VisitorContextNode<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_colors(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut<'_>,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_colors_mut(VisitorContextMut::new(node, self))
    }
}

impl DeserializeContents for ColorBuffer {
    const NAME: &str = "Colors";
    const KIND: IrNodeType = IrNodeType::ColorBuffer;

    #[tracing::instrument(skip_all)]
    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let header = SectionHeader::deserialize(reader)?;
        let components = ColorComponents::deserialize(reader)?;
        let format = ColorFormat::deserialize(reader)?;
        let stride = reader.read_u8()?;
        let _padding = reader.read_u8()?;
        let color_count = reader.read_u16::<BigEndian>()?;

        reader.set_position(header.get_data_start());

        let mut colors = Vec::with_capacity(color_count as usize);
        for _ in 0..color_count {
            let color = deserialize_color(reader, format)?;
            colors.push(color);
        }

        Ok(Self {
            header,
            components,
            format,
            stride,
            colors,
        })
    }
}
