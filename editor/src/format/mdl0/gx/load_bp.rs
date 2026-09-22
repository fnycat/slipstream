use bitfield_struct::{bitenum, bitfield};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorResult},
    format::encoding::Deserialize,
    shared::util::RefCursor,
};

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
