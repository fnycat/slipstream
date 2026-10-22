use std::ops::ControlFlow;

use bitfield_struct::bitenum;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield, try_from};
use slipstream_shared::cursor::{MutCursor, RefCursor};
use slipstream_shared::error::{CorruptionError, SlipstreamResult};
use slipstream_shared::verify;

use crate::brres::load_bp::{AlphaFunction, BlendMode, ConstantAlpha, DepthTest, LoadBpOpCode};
use crate::brres::{DeserializeContents, MdlSectionType, TextureMatrixMode};
use crate::encoding::ReadArrayExt;
use crate::node::node::IrNodeType;
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(rename = "Material Flags", summary = MaterialFlags::summary)]
pub struct MaterialFlags {
    #[inspect(rename = "Is Transparent")]
    pub xlu_material: bool,
    #[bits(23)]
    _padding: u32,
    #[inspect(rename = "Do Not Send Texture Matrix")]
    pub dont_send_tex_matrix: bool,
    #[inspect(rename = "Do Not Send UVs")]
    pub dont_send_uv: bool,
    #[inspect(rename = "Do Not Send Generator Modes")]
    pub dont_send_generator_mode: bool,
    #[inspect(rename = "Do Not Send Lighting Channels")]
    pub dont_send_lighting_channel: bool,
    #[inspect(rename = "Do Not Send Indirect Matrix")]
    pub dont_send_indirect_matrix: bool,
    #[inspect(rename = "Do Not Send UV Scale")]
    pub dont_send_uv_scale: bool,
    #[inspect(rename = "Do Not Send TEV Color")]
    pub dont_send_tev_color: bool,
    #[inspect(rename = "Do Not Send Pixel Display")]
    pub dont_send_pixel_display: bool
}

impl MaterialFlags {
    #[inline]
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.0)
    }

    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[inspect(rename = "Cull Mode")]
pub enum CullMode {
    None = 0x00,
    Front = 0x01,
    Back = 0x02,
    All = 0x03
}

impl CullMode {
    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

#[try_from(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum IndirectMethod {
    Wrap = 0x00,
    NormalMap = 0x01,
    NormalMapSpecular = 0x02,
    Fur = 0x03,
    Reserved1 = 0x04,
    Reserved2 = 0x05,
    User0 = 0x06,
    User1 = 0x07
}

impl IndirectMethod {
    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = UsedTextureMaps::summary)]
pub struct UsedTextureMaps {
    #[inspect(rename = "Texture 1 Used")]
    pub tex_used: bool,
    #[inspect(rename = "Texture 2 Used")]
    pub tex1_used: bool,
    #[inspect(rename = "Texture 3 Used")]
    pub tex2_used: bool,
    #[inspect(rename = "Texture 4 Used")]
    pub tex3_used: bool,
    #[inspect(rename = "Texture 5 Used")]
    pub tex4_used: bool,
    #[inspect(rename = "Texture 6 Used")]
    pub tex5_used: bool,
    #[inspect(rename = "Texture 7 Used")]
    pub tex6_used: bool,
    #[inspect(rename = "Texture 8 Used")]
    pub tex7_used: bool,
    #[bits(24)]
    _padding: u32
}

impl UsedTextureMaps {
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.into_bits())
    }

    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = UsedPalettes::summary)]
pub struct UsedPalettes {
    #[inspect(rename = "Palette 1 Used")]
    pub pal0_used: bool,
    #[inspect(rename = "Palette 2 Used")]
    pub pal1_used: bool,
    #[inspect(rename = "Palette 3 Used")]
    pub pal2_used: bool,
    #[inspect(rename = "Palette 4 Used")]
    pub pal3_used: bool,
    #[inspect(rename = "Palette 5 Used")]
    pub pal4_used: bool,
    #[inspect(rename = "Palette 6 Used")]
    pub pal5_used: bool,
    #[inspect(rename = "Palette 7 Used")]
    pub pal6_used: bool,
    #[inspect(rename = "Palette 8 Used")]
    pub pal7_used: bool,
    #[bits(24)]
    _padding: u32
}

impl UsedPalettes {
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.into_bits())
    }

    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct LayerSettings {
    pub enable_layer: bool,
    pub scale_fixed: bool,
    pub rotation_fixed: bool,
    pub translation_fixed: bool,
    #[bits(28)]
    _padding: u32
}

impl LayerSettings {
    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct LayerCoordinates {
    pub scale_coordinate: [f32; 2],
    pub rotation_coordinate: f32,
    pub translation_coordinate: [f32; 2]
}

impl LayerCoordinates {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let scale_coordinate = reader.read_f32_array::<2, BigEndian>()?;
        let rotation_coordinate = reader.read_f32::<BigEndian>()?;
        let translation_coordinate = reader.read_f32_array::<2, BigEndian>()?;

        Ok(Self {
            scale_coordinate,
            rotation_coordinate,
            translation_coordinate
        })
    }
}

#[try_from(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum TextureMapMode {
    #[inspect(rename = "Texture Coordinates")]
    Uv = 0x00,
    #[inspect(rename = "Environment Camera")]
    EnvCamera = 0x01,
    Projection = 0x02,
    #[inspect(rename = "Environment Light")]
    EnvLight = 0x03,
    #[inspect(rename = "Environment Specular")]
    EnvSpec = 0x04
}

impl TextureMapMode {
    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct TextureMatrixSettings {
    #[inspect(rename = "Scene Camera Reference")]
    pub scn0_camera_ref: i8,
    #[inspect(rename = "Scene Light Reference")]
    pub scn0_light_ref: i8,
    pub map_mode: TextureMapMode,
    pub enable_identity_matrix_effect: bool,
    pub texture_matrix: glam::Mat4
}

impl TextureMatrixSettings {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let scn0_camera_ref = reader.read_i8()?;
        let scn0_light_ref = reader.read_i8()?;
        let map_mode = TextureMapMode::deserialize(reader)?;
        let enable_identity_matrix_effect = reader.read_u8()? != 0;
        let tm = reader.read_f32_array::<12, BigEndian>()?;

        let texture_matrix = glam::mat4(
            glam::vec4(tm[0], tm[4], tm[8], 0.0),
            glam::vec4(tm[1], tm[5], tm[9], 0.0),
            glam::vec4(tm[2], tm[6], tm[10], 0.0),
            glam::vec4(tm[3], tm[7], tm[11], 1.0)
        );

        Ok(Self {
            scn0_camera_ref,
            scn0_light_ref,
            map_mode,
            enable_identity_matrix_effect,
            texture_matrix
        })
    }
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
#[inspect(summary = LightingChannelFlags::summary)]
pub struct LightingChannelFlags {
    pub material_color_enabled: bool,
    pub material_alpha_enabled: bool,
    pub ambient_color_enabled: bool,
    pub ambient_alpha_enabled: bool,
    pub raster_color_enabled: bool,
    pub raster_alpha_enabled: bool,
    #[bits(26)]
    _padding: u32
}

impl LightingChannelFlags {
    pub fn summary(&self) -> String {
        format!("{:#08x}", self.into_bits())
    }

    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(self.into_bits())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct LightChannelSettings {
    pub flags: LightingChannelFlags,
    pub material_color: glam::U8Vec4,
    pub ambient_color: glam::U8Vec4,
    pub color_light: LightChannelControl,
    pub alpha_light: LightChannelControl
}

impl LightChannelSettings {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let flags = LightingChannelFlags::deserialize(reader)?;
        let material_color = glam::U8Vec4::from_array(reader.read_u8_array::<4>()?);
        let ambient_color = glam::U8Vec4::from_array(reader.read_u8_array::<4>()?);
        let color_light = LightChannelControl::deserialize(reader)?;
        let alpha_light = LightChannelControl::deserialize(reader)?;

        Ok(Self {
            flags,
            material_color,
            ambient_color,
            color_light,
            alpha_light
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct MaterialMode {
    #[inspect(rename = "Alpha Function")]
    pub alpha_fn: AlphaFunction,
    pub depth_test: DepthTest,
    pub write_mask: u32,
    pub blend_mode: BlendMode,
    pub constant_alpha: ConstantAlpha
}

impl MaterialMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        verify!(reader.read_u8()? == 0x61, "expected LoadBP opcode");
        let LoadBpOpCode::AlphaFunction(alpha_fn) = LoadBpOpCode::deserialize(reader)? else {
            return Err(CorruptionError {
                reason: String::from("excepted LoadBP opcode of type AlphaFunction (0xf3)"),
                location: Some(reader.position())
            }
            .into());
        };

        verify!(reader.read_u8()? == 0x61, "expected LoadBP opcode");
        let LoadBpOpCode::DepthTest(depth_test) = LoadBpOpCode::deserialize(reader)? else {
            return Err(CorruptionError {
                reason: String::from("excepted LoadBP opcode of type DepthTest (0x40)"),
                location: Some(reader.position())
            }
            .into());
        };

        verify!(reader.read_u8()? == 0x61, "expected LoadBP opcode");
        let LoadBpOpCode::WriteMask(write_mask) = LoadBpOpCode::deserialize(reader)? else {
            return Err(CorruptionError {
                reason: String::from("excepted LoadBP opcode of type WriteMask (0xFE)"),
                location: Some(reader.position())
            }
            .into());
        };

        verify!(reader.read_u8()? == 0x61, "expected LoadBP opcode");
        let LoadBpOpCode::BlendMode(blend_mode) = LoadBpOpCode::deserialize(reader)? else {
            return Err(CorruptionError {
                reason: String::from("excepted LoadBP opcode of type BlendMode (0x41)"),
                location: Some(reader.position())
            }
            .into());
        };

        verify!(reader.read_u8()? == 0x61, "expected LoadBP opcode");
        let LoadBpOpCode::ConstantAlpha(constant_alpha) = LoadBpOpCode::deserialize(reader)? else {
            return Err(CorruptionError {
                reason: String::from("excepted LoadBP opcode of type ConstantAlpha (0x42)"),
                location: Some(reader.position())
            }
            .into());
        };

        // Then 7 bytes of padding
        reader.set_position(reader.position() + 7);

        Ok(Self {
            alpha_fn,
            depth_test,
            write_mask,
            blend_mode,
            constant_alpha
        })
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum MinificationFilter {
    Nearest = 0x00,
    Linear = 0x01,
    NearestMipmapNearest = 0x02,
    LinearMipmapNearest = 0x03,
    NearestMipmapLinear = 0x04,
    LinearMipmapLinear = 0x05
}

impl MinificationFilter {
    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum MagnificationFilter {
    Nearest = 0x00,
    Linear = 0x01
}

impl MagnificationFilter {
    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum AnisotropyFiltering {
    #[inspect(rename = "1X")]
    One = 0x00,
    #[inspect(rename = "2X")]
    Two = 0x01,
    #[inspect(rename = "4X")]
    Four = 0x02
}

impl AnisotropyFiltering {
    #[inline]
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    #[inline]
    fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

#[try_from(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum WrapMode {
    Clamp = 0x00,
    Repeat = 0x01,
    Mirror = 0x02
}

impl WrapMode {
    #[inline]
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }

    #[inline]
    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u32::<BigEndian>(*self as u32)?;
        Ok(())
    }
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum ColorSource {
    Register,
    Vertex,
    #[inspect(ignore)]
    #[fallback]
    Invalid
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum DiffuseFunction {
    Disabled,
    Enabled,
    Clamped,
    #[inspect(ignore)]
    #[fallback]
    Invalid
}

#[bitenum]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[repr(u8)]
pub enum AttenuationFunction {
    Specular,
    Spotlight,
    None,
    #[inspect(ignore)]
    #[fallback]
    Invalid
}

#[inspect_bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct LightChannelControl {
    #[bits(1)]
    pub material_source: ColorSource,
    pub light_enabled: bool,
    pub light_0: bool,
    pub light_1: bool,
    pub light_2: bool,
    pub light_3: bool,
    #[bits(1)]
    pub ambient_source: ColorSource,
    #[bits(2)]
    pub diffuse_func: DiffuseFunction,
    pub enable_attenuation: bool,
    #[bits(1)]
    pub attenuation_function: AttenuationFunction,
    pub light_4: bool,
    pub light_5: bool,
    pub light_6: bool,
    pub light_7: bool,
    #[bits(17)]
    _unused: u32
}

impl LightChannelControl {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct TextureReference {
    pub texture_id: i32,
    pub palette_id: i32,
    pub texture_data_id: u32,
    pub palette_data_id: u32,
    pub wrapping: [WrapMode; 2],
    #[inspect(rename = "Minification Filter")]
    #[inspect(tooltip = "How should the texture be sampled if the GPU must downscale it.")]
    pub min_filter: MinificationFilter,
    #[inspect(rename = "Magnification Filter")]
    #[inspect(tooltip = "How should the texture be sampled if the GPU must upscale it.")]
    pub mag_filter: MagnificationFilter,
    #[inspect(rename = "LOD Bias")]
    pub lod_bias: f32,
    pub max_anisotropy_filtering: AnisotropyFiltering,
    pub clamp_bias: u8,
    pub texel_interpolate: u8
}

impl TextureReference {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let texture_id = reader.read_i32::<BigEndian>()?;
        let palette_id = reader.read_i32::<BigEndian>()?;
        let _texture_pointer = reader.read_i32::<BigEndian>()?;
        let _palette_pointer = reader.read_i32::<BigEndian>()?;
        let texture_data_id = reader.read_u32::<BigEndian>()?;
        let palette_data_id = reader.read_u32::<BigEndian>()?;
        let uwrap = WrapMode::deserialize(reader)?;
        let vwrap = WrapMode::deserialize(reader)?;
        let wrapping = [uwrap, vwrap];
        let min_filter = MinificationFilter::deserialize(reader)?;
        let mag_filter = MagnificationFilter::deserialize(reader)?;
        let lod_bias = reader.read_f32::<BigEndian>()?;
        let max_anisotropy_filtering = AnisotropyFiltering::deserialize(reader)?;
        let clamp_bias = reader.read_u8()?;
        let texel_interpolate = reader.read_u8()?;

        // Skip over padding
        reader.set_position(reader.position() + 2);

        Ok(Self {
            texture_id,
            palette_id,
            texture_data_id,
            palette_data_id,
            wrapping,
            min_filter,
            mag_filter,
            lod_bias,
            max_anisotropy_filtering,
            clamp_bias,
            texel_interpolate
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Material {
    pub index: u32,
    pub flags: MaterialFlags,
    pub texture_gen_count: u8,
    pub light_channel_count: u8,
    pub shader_stages: u8,
    pub indirect_shader_stages: u8,
    pub cull_mode: CullMode,
    pub depth_test: u8,
    pub light_set_index: i8,
    pub fog_index: i8,
    #[inspect(max_horizontal_elements = 2)]
    pub indirect_methods: [IndirectMethod; 4],
    pub light_normal_map_refs: [i8; 4],
    pub shader_offset: i32,
    pub texture_count: u32,
    pub material_ref_offset: i32,
    pub fur_data_offset: i32,
    pub user_data_offset: i32,
    pub material_bytecode: MaterialMode,
    pub used_texture_maps: UsedTextureMaps,
    // pub precompiled_texture_code: [u8; 160],
    pub used_palettes: UsedPalettes,
    // pub precompiled_palette_code: [u8; 160],
    pub layer_settings: LayerSettings,
    pub texture_matrix_mode: TextureMatrixMode,
    #[inspect(max_horizontal_elements = 1)]
    pub layer_coordinates: [LayerCoordinates; 8],
    #[inspect(max_horizontal_elements = 1)]
    pub texture_matrix_settings: [TextureMatrixSettings; 8],
    #[inspect(max_horizontal_elements = 1)]
    pub light_channel_settings: [LightChannelSettings; 2],
    #[inspect(max_horizontal_elements = 1)]
    pub texture_references: Vec<TextureReference>
}

impl Visitable for Material {
    fn accept(&self, node: VisitorCtxMeta<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_material(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut<'_>,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_mdl()
            .map(|vis| vis.visit_material_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}

impl DeserializeContents for Material {
    const NAME: &str = "Materials";
    const KIND: IrNodeType = IrNodeType::MdlSection {
        ty: MdlSectionType::Material
    };

    #[tracing::instrument(skip_all)]
    fn deserialize_contents(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let material_start = reader.position();
        let _length = reader.read_u32::<BigEndian>()?;
        let _mdl_offset = reader.read_i32::<BigEndian>()?;
        let _name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let flags = MaterialFlags::deserialize(reader)?;
        let texture_gen_count = reader.read_u8()?;
        let light_channel_count = reader.read_u8()?;
        let shader_stages = reader.read_u8()?;
        let indirect_shader_stages = reader.read_u8()?;
        let cull_mode = CullMode::deserialize(reader)?;
        let depth_test = reader.read_u8()?;
        let light_set_index = reader.read_i8()?;
        let fog_index = reader.read_i8()?;
        let _padding = reader.read_u8()?;
        let indirect_methods = [
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?
        ];
        let light_normal_map_refs = reader.read_i8_array::<4>()?;
        let shader_offset = reader.read_i32::<BigEndian>()?;
        let texture_count = reader.read_u32::<BigEndian>()?;
        let material_ref_offset = reader.read_i32::<BigEndian>()?;
        let fur_data_offset = reader.read_i32::<BigEndian>()?;
        let user_data_offset = reader.read_i32::<BigEndian>()?;
        let mode_offset = reader.read_i32::<BigEndian>()?; // does not exist in v9 MDL or lower.

        let used_texture_maps = UsedTextureMaps::deserialize(reader)?;
        // let _precompiled_texture_code = reader.read_u8_array::<1>()?;

        // Precompiled texture code
        reader.set_position(reader.position() + 8 * 32);

        let used_palettes = UsedPalettes::deserialize(reader)?;
        // let _precompiled_palette_code = reader.read_u8_array::<160>()?;

        // Precompiled palette code
        reader.set_position(reader.position() + 8 * 12);

        let layer_settings = LayerSettings::deserialize(reader)?;
        let texture_matrix_mode = TextureMatrixMode::deserialize(reader)?;

        // Incorrect from here onwards

        let layer_coordinates = [
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?
        ];

        let texture_matrix_settings = [
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?
        ];

        let light_channel_settings = [
            LightChannelSettings::deserialize(reader)?,
            LightChannelSettings::deserialize(reader)?
        ];

        let material_bytecode = {
            let mode_start = material_start as i64 + mode_offset as i64;
            reader.set_position(mode_start as u64);

            MaterialMode::deserialize(reader)?
        };

        let texture_references = {
            let ref_start = material_start as i64 + material_ref_offset as i64;
            reader.set_position(ref_start as u64);

            let mut refs = Vec::with_capacity(texture_count as usize);
            for _ in 0..texture_count {
                refs.push(TextureReference::deserialize(reader)?);
            }

            refs
        };

        tracing::error!("TODO SHADER DATA");

        Ok(Self {
            index,
            flags,
            texture_gen_count,
            light_channel_count,
            shader_stages,
            indirect_shader_stages,
            cull_mode,
            depth_test,
            light_set_index,
            fog_index,
            indirect_methods,
            light_normal_map_refs,
            shader_offset,
            texture_count,
            material_ref_offset,
            fur_data_offset,
            user_data_offset,
            material_bytecode,
            used_texture_maps,
            used_palettes,
            layer_settings,
            texture_matrix_mode,
            layer_coordinates,
            texture_matrix_settings,
            light_channel_settings,

            texture_references
        })
    }
}
