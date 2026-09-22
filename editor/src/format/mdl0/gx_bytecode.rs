use std::ops::{Range, RangeInclusive};

use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult},
    format::{
        encoding::{Deserialize, ReadArrayExt},
        mdl0::util::VectorFormat,
    },
    shared::util::RefCursor,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GxOpCodeId {
    /// No-op
    Nop = 0x00,
    /// Load CP (command processor) register.
    LoadCp = 0x08,
    /// Load XF (transform unit) register.
    LoadXf = 0x10,
    /// Load indexed 3x3 position matrix.
    LoadPosMat = 0x20,
    /// Load indexed 3x3 normal matrix.
    LoadNorMat = 0x28,
    /// Load indexed texture matrix.
    LoadTexMat = 0x30,
    /// Load indexed light object.
    LoadLightObj = 0x38,
    /// Call display list
    Call = 0x40,
    /// Unknown opcode.
    Unknown = 0x44,
    /// Invalidate vertex cache.
    InvalidateVCache = 0x48,
    /// Load BP (blitting processor) register.
    LoadBp = 0x61,
    /// Draw quads.
    DrawQuads = 0x80,
    /// Draw triangles.
    DrawTris = 0x90,
    /// Draw triangle strip.
    DrawTriStrip = 0x98,
    /// Draw triangle fan.
    DrawTriFan = 0xa0,
    /// Draw lines.
    DrawLines = 0xa8,
    /// Draw line strip.
    DrawLineStrip = 0xb0,
    /// Draw points.
    DrawPoints = 0xb8,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VectorStorageMethod {
    #[fallback]
    NotPresent = 0b00,
    Direct = 0b01,
    Index8 = 0b10,
    Index16 = 0b11,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand1 {
    pub pm: bool,
    pub tm0: bool,
    pub tm1: bool,
    pub tm2: bool,
    pub tm3: bool,
    pub tm4: bool,
    pub tm5: bool,
    pub tm6: bool,
    pub tm7: bool,
    #[bits(2)]
    pub pos: VectorStorageMethod,
    #[bits(2)]
    pub norm: VectorStorageMethod,
    #[bits(2)]
    pub col0: VectorStorageMethod,
    #[bits(2)]
    pub col1: VectorStorageMethod,
    #[bits(15)]
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand2 {
    #[bits(2)]
    pub tex0: VectorStorageMethod,
    #[bits(2)]
    pub tex1: VectorStorageMethod,
    #[bits(2)]
    pub tex2: VectorStorageMethod,
    #[bits(2)]
    pub tex3: VectorStorageMethod,
    #[bits(2)]
    pub tex4: VectorStorageMethod,
    #[bits(2)]
    pub tex5: VectorStorageMethod,
    #[bits(2)]
    pub tex6: VectorStorageMethod,
    #[bits(2)]
    pub tex7: VectorStorageMethod,
    #[bits(16)]
    _padding: u16,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand3 {
    pub pos_e: bool,
    #[bits(3)]
    pub pos_format: VectorFormat,
    #[bits(5)]
    pub pos_divisor: u8,
    pub norm_e: bool,
    #[bits(3)]
    pub norm_format: VectorFormat,
    pub col0_e: bool,
    #[bits(3)]
    pub col0_format: VectorFormat,
    pub col1_e: bool,
    #[bits(3)]
    pub col1_format: VectorFormat,
    pub tex0_e: bool,
    #[bits(3)]
    pub tex0_format: VectorFormat,
    #[bits(5)]
    pub tex0_divisor: u8,
    pub dequant: bool,
    pub norm_l3: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand4 {
    pub tex1_e: bool,
    #[bits(3)]
    pub tex1_format: VectorFormat,
    #[bits(5)]
    pub tex1_divisor: u8,
    pub tex2_e: bool,
    #[bits(3)]
    pub tex2_format: VectorFormat,
    #[bits(5)]
    pub tex2_divisor: u8,
    pub tex3_e: bool,
    #[bits(3)]
    pub tex3_format: VectorFormat,
    #[bits(5)]
    pub tex3_divisor: u8,
    pub tex4_e: bool,
    #[bits(3)]
    pub tex4_format: VectorFormat,
    #[bits(1)]
    _padding: bool,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct CpSubCommand5 {
    #[bits(5)]
    pub tex4_divisor: u8,
    pub tex5_e: bool,
    #[bits(3)]
    pub tex5_format: VectorFormat,
    #[bits(5)]
    pub tex5_divisor: u8,
    pub tex6_e: bool,
    #[bits(3)]
    pub tex6_format: VectorFormat,
    #[bits(5)]
    pub tex6_divisor: u8,
    pub tex7_e: bool,
    #[bits(3)]
    pub tex7_format: VectorFormat,
    #[bits(5)]
    pub tex7_divisor: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadCpOpCode {
    C1(CpSubCommand1),
    C2(CpSubCommand2),
    C3(CpSubCommand3),
    C4(CpSubCommand4),
    C5(CpSubCommand5),
}

impl Deserialize for LoadCpOpCode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        let word = reader.read_u32::<BigEndian>()?;

        Ok(match byte {
            0x50 => Self::C1(CpSubCommand1::from_bits(word)),
            0x60 => Self::C2(CpSubCommand2::from_bits(word)),
            0x70 => Self::C3(CpSubCommand3::from_bits(word)),
            0x80 => Self::C4(CpSubCommand4::from_bits(word)),
            0x90 => Self::C5(CpSubCommand5::from_bits(word)),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid LoadCP subcommand: {byte:#04x}"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct XfSizePayload {
    #[bits(2)]
    pub color_count: u8,
    #[bits(2)]
    pub normal_count: u8,
    #[bits(4)]
    pub uv_count: u8,
    #[bits(24)]
    _padding: u32,
}

impl Deserialize for XfSizePayload {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum XfProjectionType {
    St,
    Stq,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum XfInputForm {
    /// Generally used with ST projection.
    Ab11,
    /// Generally used with STQ projection.
    Abc1,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum XfTexGenType {
    Regular,
    EmbossMapping,
    ColorMapping1,
    ColorMapping2,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum XfSourceRow {
    VertexGeometry,
    Normals,
    Colors,
    BinormalT,
    BinormalB,
    Uv1,
    Uv2,
    Uv3,
    Uv4,
    Uv5,
    Uv6,
    Uv7,
    #[fallback]
    Invalid,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct XfSetPayload {
    #[bits(1)]
    pub _unknown1: bool,
    #[bits(1)]
    pub projection: XfProjectionType,
    #[bits(1)]
    pub input_form: XfInputForm,
    #[bits(1, default = false)]
    pub _unknown2: bool,
    #[bits(3)]
    pub tex_gen_type: XfTexGenType,
    #[bits(5)]
    pub source_row: XfSourceRow,
    #[bits(3)]
    pub used_gen_tex_coord: u8,
    #[bits(3)]
    pub used_light_index: u8,
    #[bits(14)]
    _padding: u16,
}

impl Deserialize for XfSetPayload {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadXfPayload {
    Size(XfSizePayload),
    Set { tex_id: u8, payload: XfSetPayload },
    Unknown { tex_id: u8, payload: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadXfOpCode {
    pub loads: Vec<LoadXfPayload>,
}

impl Deserialize for LoadXfOpCode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let transfer_size = reader.read_u16::<BigEndian>()?;
        let address = reader.read_u16::<BigEndian>()?;

        let mut loads = Vec::with_capacity(transfer_size as usize + 1);
        for _ in 0..transfer_size + 1 {
            let load = match address {
                0x1008 => LoadXfPayload::Size(XfSizePayload::deserialize(reader)?),
                0x1040..=0x1047 => {
                    let tex_id = (address - 0x1040) as u8;
                    LoadXfPayload::Set {
                        tex_id,
                        payload: XfSetPayload::deserialize(reader)?,
                    }
                }
                0x1050..=0x1057 => {
                    let tex_id = (address - 0x1050) as u8;
                    LoadXfPayload::Unknown {
                        tex_id,
                        payload: reader.read_u32::<BigEndian>()?,
                    }
                },
                _ => return Err(CorruptionError {
                    reason: format!("invalid XF register: {address:#06x} (expected 0x1008, 0x1040..=0x1047 or 0x1050..=0x1057)"),
                    location: Some(reader.position())
                }.into())
            };

            loads.push(load);
        }

        Ok(Self { loads })
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct SetIndirectTexture {
    #[bits(3)]
    pub map0: u8,
    #[bits(3)]
    pub coord0: u8,
    #[bits(3)]
    pub map1: u8,
    #[bits(3)]
    pub coord1: u8,
    #[bits(3)]
    pub map2: u8,
    #[bits(3)]
    pub coord2: u8,
    #[bits(3)]
    pub map3: u8,
    #[bits(3)]
    pub coord3: u8,
    #[bits(8)]
    _padding: u8,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RasConstant {
    Col0 = 0b000,
    Col1 = 0b001,
    AlphaBump = 0b101,
    AlphaBumpCorrected = 0b110,
    Zero = 0b111,
    #[fallback]
    Invalid,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct TextureReadSettings {
    #[bits(3)]
    _unknown1: u8,
    #[bits(3)]
    _unknown2: u8,
    pub tex0_e: bool,
    #[bits(3)]
    pub ras0_value: RasConstant,
    #[bits(2)]
    _padding1: u8,
    #[bits(3)]
    _unknown3: u8,
    #[bits(3)]
    _unknown4: u8,
    pub tex1_e: bool,
    #[bits(3)]
    pub ras1_value: RasConstant,
    #[bits(10)]
    _padding2: u16,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum AlphaBlendDest {
    Zero = 0b000,
    One = 0b001,
    SourceColor = 0b010,
    InverseSourceColor = 0b011,
    SourceAlpha = 0b100,
    InverseSourceAlpha = 0b101,
    DestinationAlpha = 0b110,
    InverseDestinationAlpha = 0b111,
    #[fallback]
    Invalid,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct AlphaBlendSettings {
    pub enable_alpha: bool,
    #[bits(4)]
    _unknown1: u8,
    #[bits(3)]
    pub blend_dest: AlphaBlendDest,
    #[bits(3)]
    pub blend_src: AlphaBlendDest,
    #[bits(5)]
    _unknown2: u8,
    #[bits(16)]
    _padding: u16,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerBlendDest {
    FragmentOutput = 0b00,
    Temp0 = 0b01,
    Temp1 = 0b10,
    Temp2 = 0b11,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerBlendShift {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Five = 0b11,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorBlendArgument {
    FragmentOutput = 0b0000,
    FragmentOutputAlpha = 0b0001,
    Temp0 = 0b0010,
    Temp0Alpha = 0b0011,
    Temp1 = 0b0100,
    Temp1Alpha = 0b0101,
    Temp2 = 0b0110,
    Temp2Alpha = 0b0111,
    Texture = 0b1000,
    TextureAlpha = 0b1001,
    Ras = 0b1010,
    RasAlpha = 0b1011,
    One = 0b1100,
    Half = 0b1101,
    Const = 0b1110,
    Zero = 0b1111,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerBlendBias {
    Zero = 0b00,
    Half = 0b01,
    NegativeHalf = 0b10,
    Special = 0b11,
    #[fallback]
    Invalid,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerBlendOp {
    Add = 0b0,
    Subtract = 0b1,
    #[fallback]
    Invalid,
}

/// The command performed on each layer by the graphics obeys the format:
///
/// `dest = shift * (argument_d op lerp(argument_a, argument_b, argument_c) + bias)`.
#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct ColorLayerBlendSettings {
    #[bits(4)]
    pub argument_a: ColorBlendArgument,
    #[bits(4)]
    pub argument_b: ColorBlendArgument,
    #[bits(4)]
    pub argument_c: ColorBlendArgument,
    #[bits(4)]
    pub argument_d: ColorBlendArgument,
    #[bits(2)]
    pub bias: LayerBlendBias,
    #[bits(1)]
    pub op: LayerBlendOp,
    pub clamp: bool,
    #[bits(2)]
    pub shift: LayerBlendShift,
    #[bits(2)]
    pub dest: LayerBlendDest,
    #[bits(8)]
    _padding: u8,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum AlphaBlendArgument {
    FragmentOutput = 0b000,
    Temp0 = 0b001,
    Temp1 = 0b010,
    Temp2 = 0b011,
    Texture = 0b100,
    Ras = 0b101,
    Const = 0b110,
    Zero = 0b111,
    #[fallback]
    Invalid,
}

/// The command performed on each layer by the graphics card obeys the format:
///
/// `dest = shift * (argument_d op lerp(argument_a, argument_b, argument_c) + bias)`.
#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct AlphaLayerBlendSettings {
    #[bits(2)]
    _unknown1: u8,
    #[bits(2)]
    _unknown2: u8,
    #[bits(3)]
    pub argument_a: AlphaBlendArgument,
    #[bits(3)]
    pub argument_b: AlphaBlendArgument,
    #[bits(3)]
    pub argument_c: AlphaBlendArgument,
    #[bits(3)]
    pub argument_d: AlphaBlendArgument,
    #[bits(2)]
    pub bias: LayerBlendBias,
    #[bits(1)]
    pub op: LayerBlendOp,
    pub clamp: bool,
    #[bits(2)]
    pub shift: LayerBlendShift,
    #[bits(2)]
    pub dest: LayerBlendDest,
    #[bits(8)]
    _padding: u8,
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SwapComponent {
    Red = 0b00,
    Green = 0b01,
    Blue = 0b10,
    Alpha = 0b11,
    #[fallback]
    Invalid,
}

/// Controls how the swap mode table is accessed when rendering a texture on a polygon.
#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct SwapModeTableSettings {
    #[bits(2)]
    pub swap1: SwapComponent,
    #[bits(2)]
    pub swap2: SwapComponent,
    #[bits(5)]
    pub color0_index: u8,
    #[bits(5)]
    pub alpha0_index: u8,
    #[bits(5)]
    pub color1_index: u8,
    #[bits(5)]
    pub alpha1_index: u8,
    #[bits(8)]
    _padding: u8,
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct MaterialAddressRange {
    #[bits(11)]
    pub red_blue: u16,
    _padding1: bool,
    #[bits(11)]
    pub alpha_green: u16,
    pub constant: bool,
    #[bits(8)]
    _padding2: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadBpOpCode {
    IndirectTexture(SetIndirectTexture),
    TextureRead {
        tex_id: u8,
        payload: TextureReadSettings,
    },
    AlphaBlend(AlphaBlendSettings),
    ColorLayerBlend {
        layer: u8,
        payload: ColorLayerBlendSettings,
    },
    AlphaLayerBlend {
        layer: u8,
        payload: AlphaLayerBlendSettings,
    },
    SwapModeTable {
        table: u8,
        payload: SwapModeTableSettings,
    },
    MaterialAddress {
        components: ComponentsSet,
        payload: MaterialAddressRange,
    },
    WriteMask(u32),
}

const BP_INDIRECT_TEXTURE_OPCODE: u8 = 0x27;
const BP_TEXTURE_READ_OPCODE_RANGE: RangeInclusive<u8> = 0x28..=0x2f;
const BP_ALPHA_BLEND_OPCODE: u8 = 0x41;
const BP_COLOR_LAYER_BLEND_OPCODES: [u8; 8] = [0xc0, 0xc2, 0xc4, 0xc6, 0xc8, 0xca, 0xcc, 0xce];
const BP_ALPHA_LAYER_BLEND_OPCODES: [u8; 8] = [0xc1, 0xc3, 0xc5, 0xc7, 0xc9, 0xcb, 0xcd, 0xcf];
const BP_SWAP_MODE_OPCODE_RANGE: RangeInclusive<u8> = 0xf6..=0xfd;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ComponentsSet {
    RedAlpha,
    BlueGreen,
}

impl Deserialize for LoadBpOpCode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let address = reader.read_u16::<BigEndian>()?;
        let value = reader.read_u24::<BigEndian>()?;

        let payload = match address {
            0x27 => Self::IndirectTexture(SetIndirectTexture::from_bits(value)),
            0x28..=0x2f => {
                let tex_id = todo!("compute tex id");
                Self::TextureRead {
                    tex_id,
                    payload: TextureReadSettings::from_bits(value),
                }
            }
            0x41 => Self::AlphaBlend(AlphaBlendSettings::from_bits(value)),
            0xc0 | 0xc2 | 0xc4 | 0xc6 | 0xc8 | 0xca | 0xcc | 0xce => Self::ColorLayerBlend {
                layer: address as u8,
                payload: ColorLayerBlendSettings::from_bits(value),
            },
            0xc1 | 0xc3 | 0xc5 | 0xc7 | 0xc9 | 0xcb | 0xcd | 0xcf => Self::AlphaLayerBlend {
                layer: address as u8,
                payload: AlphaLayerBlendSettings::from_bits(value),
            },
            0xf6..=0xfd => {
                // table id is the 3 lowest bits of the address.
                let table = (address & 0x03) as u8;
                Self::SwapModeTable {
                    table: table,
                    payload: SwapModeTableSettings::from_bits(value),
                }
            }
            0xe0..=0xe8 => {
                let lsb_set = (address & 0x01) == 0x01;
                Self::MaterialAddress {
                    components: if lsb_set {
                        ComponentsSet::BlueGreen
                    } else {
                        ComponentsSet::RedAlpha
                    },
                    payload: MaterialAddressRange::from_bits(value),
                }
            }
            0xfe => Self::WriteMask(value),
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid blit processor opcode: {address:#04x}"),
                    location: Some(reader.position()),
                }
                .into());
            }
        };

        Ok(payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GxOpCode {
    Nop,
    LoadCp(LoadCpOpCode),
    LoadXf(LoadXfOpCode),
    LoadBp(LoadBpOpCode),
}

#[derive(Debug, Clone)]
pub struct GxBytecode {}

impl Deserialize for GxBytecode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        todo!()
    }
}
