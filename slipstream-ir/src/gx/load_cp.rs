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

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum VectorStorage {
    #[inspect(tooltip = "Do not store any data for this object.")]
    #[fallback]
    NotPresent = 0b00,
    #[inspect(tooltip = "Store the data directly in the draw command itself.\
        You should probably use indexing instead, for better performance.")]
    Inline = 0b01,
    #[inspect(tooltip = "Store an 8-bit index to an entry in a separate buffer.\
        The GPU will use this index to load the data from the buffer given by the corresponding `Array ID`\
        entry in this polygon.
        Keep in mind that this index only supports referencing entries in the range 0-255.\
        Use 16-bit indexing if you need more.")]
    Index8 = 0b10,
    #[inspect(tooltip = "Store a 16-bit index to an entry in a separate buffer.\
        The GPU wull use this index to load the data from the buffer given by the corresponding `Array ID`\
        entry in this polygon.")]
    Index16 = 0b11,
}

/// Vertex control descriptor part 1/2.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Control Descriptor 1")]
pub struct CpVcdLo {
    /// Whether the position/normal matrix index is stored in this vertex.
    ///
    /// This matrix index points to the matrix in the parent shape's bone table,
    /// that should be used to transform this vertex.
    ///
    /// Also called: `GX_VA_PNMTXIDX`
    #[inspect(rename = "Position/Normal Matrix Enabled")]
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
    #[bits(2)]
    pub pos_storage: VectorStorage,
    #[inspect(rename = "Normal Storage")]
    #[bits(2)]
    pub norm_storage: VectorStorage,
    #[inspect(rename = "Color Layer 1 Storage")]
    #[bits(2)]
    pub col0_storage: VectorStorage,
    #[inspect(rename = "Color Layer 2 Storage")]
    #[bits(2)]
    pub col1_storage: VectorStorage,
    #[bits(15)]
    _padding: u16,
}

/// Vertex control descriptor part 2/2.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Control Descriptor 2")]
pub struct CpVcdHi {
    #[inspect(rename = "Texture Coordinate 1 Storage")]
    #[bits(2)]
    pub uv0_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 2 Storage")]
    #[bits(2)]
    pub uv1_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 3 Storage")]
    #[bits(2)]
    pub uv2_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 4 Storage")]
    #[bits(2)]
    pub uv3_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 5 Storage")]
    #[bits(2)]
    pub uv4_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 6 Storage")]
    #[bits(2)]
    pub uv5_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 7 Storage")]
    #[bits(2)]
    pub uv6_storage: VectorStorage,
    #[inspect(rename = "Texture Coordinate 8 Storage")]
    #[bits(2)]
    pub uv7_storage: VectorStorage,
    #[bits(16)]
    _padding: u16,
}

/// Vertex attribute table, part 1/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 1")]
pub struct CpVatA {
    #[inspect(rename = "3D Position")]
    pub pos_extended: bool,
    #[inspect(rename = "Position Format")]
    #[bits(3)]
    pub pos_format: VertexFormat,
    #[inspect(rename = "Position Format Divisor")]
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
    pub uv0_extended: bool,
    #[inspect(rename = "Texture Coordinate 0 Format")]
    #[bits(3)]
    pub uv0_format: VertexFormat,
    #[inspect(rename = "Texture Coordinate 1 Divisor")]
    #[bits(5)]
    pub uv0_divisor: u8,
    pub dequant: bool,
    /// If this flag is set and [`norm_extended`] is also set, then normals are indexed using
    /// 3 separate indices. If not set and [`norm_extended`] is set, then the normals will be read
    /// as 9 consecutive floats.
    #[inspect(rename = "Additional Normals")]
    pub norm_i3: bool,
}

/// Vertex attribute table, part 2/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 2")]
pub struct CpVatB {
    pub uv1_extended: bool,
    #[bits(3)]
    pub uv1_format: VertexFormat,
    #[bits(5)]
    pub uv1_divisor: u8,
    pub uv2_extended: bool,
    #[bits(3)]
    pub uv2_format: VertexFormat,
    #[bits(5)]
    pub uv2_divisor: u8,
    pub uv3_extended: bool,
    #[bits(3)]
    pub uv3_format: VertexFormat,
    #[bits(5)]
    pub uv3_divisor: u8,
    pub uv4_extended: bool,
    #[bits(3)]
    pub uv4_format: VertexFormat,
    #[bits(1)]
    _padding: bool,
}

/// Vertex attribute table, part 3/3.
#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Vertex Attribute Table 3")]
pub struct CpVatC {
    #[bits(5)]
    pub uv4_divisor: u8,
    pub uv5_extended: bool,
    #[bits(3)]
    pub uv5_format: VertexFormat,
    #[bits(5)]
    pub uv5_divisor: u8,
    pub uv6_extended: bool,
    #[bits(3)]
    pub uv6_format: VertexFormat,
    #[bits(5)]
    pub uv6_divisor: u8,
    pub uv7_extended: bool,
    #[bits(3)]
    pub uv7_format: VertexFormat,
    #[bits(5)]
    pub uv7_divisor: u8,
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
