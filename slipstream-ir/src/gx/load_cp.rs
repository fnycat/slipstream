use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield};
use slipstream_shared::{
    cursor::{MutCursor, RefCursor, SizeEstimate},
    error::{CorruptionError, SlipstreamResult},
};

use crate::{
    mdl0::{ColorFormat, NormalFormat},
    util::VertexFormat,
};

pub const FORMAT_DIVISOR_TOOLTIP: &str = "Determines how to scale lower quality formats back up to floats. \
A lower quality format is scaled up to a float by dividing it by `2^divisor`. \
This is used to convert integer formats into fractional floats. \
\
The setting does not have any effect when the format is set to `Float32`.";

const UV_EXTENDED_TOOLTIP: &str = "Whether the texture coordinate uses one or two components. \
If set to `true`, the coordinate will have both an S and a T component. While `false` will only use the S component.";

const UV_FORMAT_TOOLTIP: &str = "The format that texture coordinates are stored in. In comparison to `Float32`, \
lower qualify formats such as `Int16` can significantly reduce model size at the cost of precision.";

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum VectorStorage {
    #[inspect(tooltip = "Do not store any data for this object.")]
    #[fallback]
    NotPresent = 0b00,
    #[inspect(tooltip = "Store the data directly in the draw command itself. \
        You should probably use indexing instead, for better performance.")]
    Inline = 0b01,
    #[inspect(rename = "8-Bit Index")]
    #[inspect(tooltip = "Store an 8-bit index to an entry in a separate buffer. \
        The GPU will use this index to load the data from the buffer given by the corresponding `Array ID` \
        entry in this polygon.\
        Keep in mind that this index only supports referencing entries in the range 0-255. \
        Use 16-bit indexing if you need more.")]
    Index8 = 0b10,
    #[inspect(rename = "16-Bit Index")]
    #[inspect(tooltip = "Store a 16-bit index to an entry in a separate buffer. \
        The GPU will use this index to load the data from the buffer given by the corresponding `Array ID` \
        entry in this polygon.")]
    Index16 = 0b11,
}

/// Vertex control descriptor part 1/2.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Descriptor 1", summary = CpVcdLo::summary)]
pub struct CpVcdLo {
    /// Whether the position/normal matrix index is stored in this vertex.
    ///
    /// This matrix index points to the matrix in the parent shape's bone table,
    /// that should be used to transform this vertex.
    ///
    /// Also called: `GX_VA_PNMTXIDX`
    #[inspect(rename = "Position/Normal Matrix Enabled")]
    #[inspect(tooltip = "Known as `GA_VA_PNMTXID` in GX")]
    pub pn_index_enabled: bool,
    #[inspect(rename = "Texture Matrix 1 Enabled")]
    pub tm0: bool,
    #[inspect(rename = "Texture Matrix 2 Enabled")]
    pub tm1: bool,
    #[inspect(rename = "Texture Matrix 3 Enabled")]
    pub tm2: bool,
    #[inspect(rename = "Texture Matrix 4 Enabled")]
    pub tm3: bool,
    #[inspect(rename = "Texture Matrix 5 Enabled")]
    pub tm4: bool,
    #[inspect(rename = "Texture Matrix 6 Enabled")]
    pub tm5: bool,
    #[inspect(rename = "Texture Matrix 7 Enabled")]
    pub tm6: bool,
    #[inspect(rename = "Texture Matrix 8 Enabled")]
    pub tm7: bool,
    #[inspect(rename = "Position Storage")]
    #[inspect(tooltip = "How should positions be stored in this polygon?")]
    #[bits(2)]
    pub pos_storage: VectorStorage,
    #[inspect(rename = "Normal Storage")]
    #[inspect(tooltip = "How should normals for be stored in this polygon?")]
    #[bits(2)]
    pub norm_storage: VectorStorage,
    #[inspect(rename = "Color Layer 1 Storage")]
    #[inspect(
        tooltip = "How should the first color layer be stored in this polygon? \
        Many characters just use an entirely white color layer with an 8-bit index. \
        The main use of this layer is applying baked ambient occlusion"
    )]
    #[bits(2)]
    pub col0_storage: VectorStorage,
    #[inspect(rename = "Color Layer 2 Storage")]
    #[inspect(
        tooltip = "How should the second color layer be stored in this polygon? \
        This color layer is almost never used."
    )]
    #[bits(2)]
    pub col1_storage: VectorStorage,
    #[bits(15)]
    _padding: u16,
}

impl CpVcdLo {
    pub fn summary(&self) -> String {
        format!("GX_VTXDESC0 {:#08x}", self.into_bits())
    }
}

/// Vertex control descriptor part 2/2.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Descriptor 2", summary = CpVcdHi::summary)]
pub struct CpVcdHi {
    #[inspect(rename = "Texture Coordinate 1 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 1 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv0_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 2 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 2 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv1_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 3 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 3 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv2_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 4 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 4 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv3_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 5 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 5 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv4_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 6 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 6 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv5_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 7 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 7 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv6_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 8 Storage")]
    #[inspect(
        tooltip = "How should the texture coordinates for texture 8 be stored in this polygon?"
    )]
    #[bits(2)]
    pub uv7_storage: VectorStorage,
    #[bits(16)]
    _padding: u16,
}

impl CpVcdHi {
    #[inline]
    pub fn summary(&self) -> String {
        format!("GX_VTXDESC1 {:#08x}", self.into_bits())
    }
}

/// Vertex attribute table, part 1/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 1", summary = CpVatA::summary)]
pub struct CpVatA {
    #[inspect(rename = "3D Position")]
    pub pos_extended: bool,
    #[inspect(rename = "Position Format")]
    #[inspect(
        tooltip = "The format that positions are stored in. In comparison to `Float32`, \
        lower qualify formats such as `Int16` can significantly reduce model size at the cost of precision."
    )]
    #[bits(3)]
    pub pos_format: VertexFormat,
    #[inspect(rename = "Position Format Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub pos_divisor: u8,
    pub norm_extended: bool,
    #[inspect(rename = "Normal Format")]
    #[bits(3)]
    pub norm_format: NormalFormat,
    pub col0_extended: bool,
    #[inspect(rename = "Color Layer 1 Format")]
    #[bits(3)]
    pub col0_format: ColorFormat,
    pub col1_extended: bool,
    #[inspect(rename = "Color Layer 2 Format")]
    #[bits(3)]
    pub col1_format: ColorFormat,
    #[inspect(rename = "Extend UV 1")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv0_extended: bool,
    #[inspect(rename = "UV 0 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv0_format: VertexFormat,
    #[inspect(rename = "UV 1 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv0_divisor: u8,
    pub dequant: bool,
    /// If this flag is set and [`norm_extended`] is also set, then normals are indexed using
    /// 3 separate indices. If not set and [`norm_extended`] is set, then the normals will be read
    /// as 9 consecutive floats.
    #[inspect(rename = "Use Additional Normals")]
    pub norm_i3: bool,
}

impl CpVatA {
    #[inline]
    pub fn summary(&self) -> String {
        format!("GX_VTXFMT0 {:#08x}", self.into_bits())
    }
}

/// Vertex attribute table, part 2/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 2", summary = CpVatB::summary)]
pub struct CpVatB {
    #[inspect(rename = "Extend UV 2")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv1_extended: bool,
    #[inspect(rename = "UV 2 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv1_format: VertexFormat,
    #[inspect(rename = "UV 2 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv1_divisor: u8,
    #[inspect(rename = "Extend UV 3")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv2_extended: bool,
    #[inspect(rename = "UV 3 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv2_format: VertexFormat,
    #[inspect(rename = "UV 3 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv2_divisor: u8,
    #[inspect(rename = "Extend UV 4")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv3_extended: bool,
    #[inspect(rename = "UV 4 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv3_format: VertexFormat,
    #[inspect(rename = "UV 4 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv3_divisor: u8,
    #[inspect(rename = "Extend UV 5")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv4_extended: bool,
    #[inspect(rename = "UV 5 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv4_format: VertexFormat,
    #[bits(1)]
    _padding: bool,
}

impl CpVatB {
    #[inline]
    pub fn summary(&self) -> String {
        format!("GX_VTXFMT1 {:#08x}", self.into_bits())
    }
}

/// Vertex attribute table, part 3/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 3", summary = CpVatC::summary)]
pub struct CpVatC {
    #[inspect(rename = "UV 5 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv4_divisor: u8,
    #[inspect(rename = "Extend UV 6")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv5_extended: bool,
    #[inspect(rename = "UV 6 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv5_format: VertexFormat,
    #[inspect(rename = "UV 6 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv5_divisor: u8,
    #[inspect(rename = "Extend UV 7")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv6_extended: bool,
    #[inspect(rename = "UV 7 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv6_format: VertexFormat,
    #[inspect(rename = "UV 7 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv6_divisor: u8,
    #[inspect(rename = "Extend UV 8")]
    #[inspect(
        tooltip = UV_EXTENDED_TOOLTIP
    )]
    pub uv7_extended: bool,
    #[inspect(rename = "UV 8 Format")]
    #[inspect(
        tooltip = UV_FORMAT_TOOLTIP
    )]
    #[bits(3)]
    pub uv7_format: VertexFormat,
    #[inspect(rename = "UV 8 Divisor")]
    #[inspect(tooltip = FORMAT_DIVISOR_TOOLTIP)]
    #[bits(5)]
    pub uv7_divisor: u8,
}

impl CpVatC {
    #[inline]
    pub fn summary(&self) -> String {
        format!("GX_VTXFMT2 {:#08x}", self.into_bits())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadCpOpCode {
    VcdLo(CpVcdLo),
    VcdHi(CpVcdHi),
    VatA(CpVatA),
    VatB(CpVatB),
    VatC(CpVatC),
}

const VCD_LO_OPCODE: u8 = 0x50;
const VCD_HI_OPCODE: u8 = 0x60;
const VAT_A_OPCODE: u8 = 0x70;
const VAT_B_OPCODE: u8 = 0x80;
const VAT_C_OPCODE: u8 = 0x90;

impl LoadCpOpCode {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        let word = reader.read_u32::<BigEndian>()?;

        Ok(match byte {
            VCD_LO_OPCODE => Self::VcdLo(CpVcdLo::from_bits(word)),
            VCD_HI_OPCODE => Self::VcdHi(CpVcdHi::from_bits(word)),
            VAT_A_OPCODE => Self::VatA(CpVatA::from_bits(word)),
            VAT_B_OPCODE => Self::VatB(CpVatB::from_bits(word)),
            VAT_C_OPCODE => Self::VatC(CpVatC::from_bits(word)),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid LoadCP subcommand: {byte:#04x}"),
                    ..Default::default()
                }
                .into());
            }
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        match self {
            Self::VcdLo(x) => {
                writer.write_u8(VCD_LO_OPCODE)?;
                writer.write_u32::<BigEndian>(x.into_bits())?;
            }
            Self::VcdHi(x) => {
                writer.write_u8(VCD_HI_OPCODE)?;
                writer.write_u32::<BigEndian>(x.into_bits())?;
            }
            Self::VatA(x) => {
                writer.write_u8(VAT_A_OPCODE)?;
                writer.write_u32::<BigEndian>(x.into_bits())?;
            }
            Self::VatB(x) => {
                writer.write_u8(VAT_B_OPCODE)?;
                writer.write_u32::<BigEndian>(x.into_bits())?;
            }
            Self::VatC(x) => {
                writer.write_u8(VAT_C_OPCODE)?;
                writer.write_u32::<BigEndian>(x.into_bits())?;
            }
        }
        Ok(())
    }
}

impl SizeEstimate for LoadCpOpCode {
    #[inline]
    fn estimate_size(&self) -> usize {
        1 + 4
    }
}
