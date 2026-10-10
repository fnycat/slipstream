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
    img::deserialize_color,
    mdl0::{SectionHeader, section::DeserializeContents},
};
use crate::{
    node::node::IrNodeType,
    visitor::{Visitable, Visitor},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum ColorComponents {
    #[inspect(rename = "RGB")]
    Rgb = 0x00,
    #[inspect(rename = "RGBA")]
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
    #[inspect(rename = "RGB565")]
    #[inspect(
        tooltip = "Stores a color in 16 bits. 5 bits are used for the red and blue channels, while the green channel uses 6 bits. The alpha channel is not supported."
    )]
    Rgb565 = 0x00,
    /// 24 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: N/A
    #[inspect(rename = "RGB8")]
    #[inspect(
        tooltip = "Stores a color in 24 bits. All color channels use 8 bits. The alpha channel is not supported."
    )]
    Rgb8 = 0x01,
    /// 32 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: 8 bits (discarded)
    #[inspect(rename = "RGBX8")]
    #[inspect(
        tooltip = "Stores a color in 32 bits. All color channels use 8 bits. The alpha channel is discarded."
    )]
    Rgbx32 = 0x02,
    /// 16 total bits.
    ///
    /// Red: 4 bits;
    /// Green: 4 bits;
    /// Blue: 4 bits;
    /// Alpha: 4 bits
    #[inspect(rename = "RGBA4")]
    #[inspect(tooltip = "Stores a color in 16 bits. Each channel uses 4 bits.")]
    Rgba4 = 0x03,
    /// 24 total bits.
    ///
    /// Red: 6 bits;
    /// Green: 6 bits;
    /// Blue: 6 bits;
    /// Alpha: 6 bits
    #[inspect(rename = "RGBA6")]
    #[inspect(tooltip = "Stores a color in 24 bits. Each channel uses 6 bits.")]
    Rgba6 = 0x04,
    /// 32 total bits.
    ///
    /// Red: 8 bits;
    /// Green: 8 bits;
    /// Blue: 8 bits;
    /// Alpha: 8 bits
    #[inspect(rename = "RGBA8")]
    #[inspect(tooltip = "Stores a color in 32-bits. Each channels has 8 bits.")]
    Rgba8 = 0x05,
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
            Self::Rgb8 => 3,
            Self::Rgbx32 => 4,
            Self::Rgba4 => 2,
            Self::Rgba6 => 3,
            Self::Rgba8 => 4,
            Self::Invalid => 0,
        }
    }
}

impl TryFrom<u32> for ColorFormat {
    type Error = SlipstreamError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Rgb565,
            0x01 => Self::Rgb8,
            0x02 => Self::Rgbx32,
            0x03 => Self::Rgba4,
            0x04 => Self::Rgba6,
            0x05 => Self::Rgba8,
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
#[derive(Debug, Clone, Inspect)]
pub struct ColorBuffer {
    #[inspect(opened = false)]
    header: SectionHeader,
    #[inspect(rename = "Enabled Components")]
    #[inspect(tooltip = "Whether to use only the color channels or also include alpha.")]
    components: ColorComponents,
    #[inspect(
        tooltip = "The color format used to store the color data. Smaller formats use less data but might result in lower quality."
    )]
    format: ColorFormat,
    #[inspect(opened = false)]
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
        // Stride can be determined from `format`.
        let _stride = reader.read_u8()?;
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
            colors,
        })
    }
}
