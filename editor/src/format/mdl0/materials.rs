use std::mem::MaybeUninit;

use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{CorruptionError, EditorError, EditorResult},
    format::{
        brres::IndexGroup,
        encoding::{Deserialize, ReadArrayExt},
        mdl0::TextureMatrixMode,
    },
    node::{
        defer::Deferred,
        node::{VirtualNode, VirtualNodeBody, VirtualNodeKind},
        refs::{VirtualNodeId, VirtualNodeMap},
    },
    shared::util::RefCursor,
};

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct MaterialFlags {
    pub xlu_material: bool,
    #[bits(23)]
    _padding: u32,
    pub dont_send_tex_matrix: bool,
    pub dont_send_uv: bool,
    pub dont_send_generator_mode: bool,
    pub dont_send_lighting_channel: bool,
    pub dont_send_indirect_matrix: bool,
    pub dont_send_uv_scale: bool,
    pub dont_send_tev_color: bool,
    pub dont_send_pixel_display: bool,
}

impl Deserialize for MaterialFlags {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Culling {
    None = 0x00,
    Front = 0x01,
    Back = 0x02,
    All = 0x03,
}

impl TryFrom<u32> for Culling {
    type Error = EditorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::None,
            0x01 => Self::Front,
            0x02 => Self::Back,
            0x03 => Self::All,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid culling mode: {value:#04x} (expected 0-3)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for Culling {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Self::try_from(word)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IndirectMethod {
    Wrap = 0x00,
    NormalMap = 0x01,
    NormalMapSpecular = 0x02,
    Fur = 0x03,
    Reserved1 = 0x04,
    Reserved2 = 0x05,
    User0 = 0x06,
    User1 = 0x07,
}

impl TryFrom<u8> for IndirectMethod {
    type Error = EditorError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Wrap,
            0x01 => Self::NormalMap,
            0x02 => Self::NormalMapSpecular,
            0x03 => Self::Fur,
            0x04 => Self::Reserved1,
            0x05 => Self::Reserved2,
            0x06 => Self::User0,
            0x07 => Self::User1,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid indirect method: {value:#04x} (expected 0-7)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for IndirectMethod {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct UsedTextureMaps {
    pub tex0_used: bool,
    pub tex1_used: bool,
    pub tex2_used: bool,
    pub tex3_used: bool,
    pub tex4_used: bool,
    pub tex5_used: bool,
    pub tex6_used: bool,
    pub tex7_used: bool,
    #[bits(24)]
    _padding: u32,
}

impl Deserialize for UsedTextureMaps {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct UsedPalettes {
    pub pal0_used: bool,
    pub pal1_used: bool,
    pub pal2_used: bool,
    pub pal3_used: bool,
    pub pal4_used: bool,
    pub pal5_used: bool,
    pub pal6_used: bool,
    pub pal7_used: bool,
    #[bits(24)]
    _padding: u32,
}

impl Deserialize for UsedPalettes {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct LayerSettings {
    pub enable_layer: bool,
    pub scale_fixed: bool,
    pub rotation_fixed: bool,
    pub translation_fixed: bool,
    #[bits(28)]
    _padding: u32,
}

impl Deserialize for LayerSettings {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerCoordinates {
    pub scale_coordinate: [f32; 2],
    pub rotation_coordinate: f32,
    pub translation_coordinate: [f32; 2],
}

impl Deserialize for LayerCoordinates {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let scale_coordinate = reader.read_f32_array::<2, BigEndian>()?;
        let rotation_coordinate = reader.read_f32::<BigEndian>()?;
        let translation_coordinate = reader.read_f32_array::<2, BigEndian>()?;

        Ok(Self {
            scale_coordinate,
            rotation_coordinate,
            translation_coordinate,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextureMapMode {
    Uv = 0x00,
    EnvCamera = 0x01,
    Projection = 0x02,
    EnvLight = 0x03,
    EnvSpec = 0x04,
}

impl TryFrom<u8> for TextureMapMode {
    type Error = EditorError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Uv,
            0x01 => Self::EnvCamera,
            0x02 => Self::Projection,
            0x03 => Self::EnvLight,
            0x04 => Self::EnvSpec,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid texture map mode: {value} (expected 0-4)"),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

impl Deserialize for TextureMapMode {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureMatrixSettings {
    pub scn0_camera_ref: i8,
    pub scn0_light_ref: i8,
    pub map_mode: TextureMapMode,
    pub enable_identity_matrix_effect: bool,
    pub texture_matrix: glam::Mat4,
}

impl Deserialize for TextureMatrixSettings {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let scn0_camera_ref = reader.read_i8()?;
        let scn0_light_ref = reader.read_i8()?;
        let map_mode = TextureMapMode::deserialize(reader)?;
        let enable_identity_matrix_effect = reader.read_u8()? != 0;
        let tm = reader.read_f32_array::<12, BigEndian>()?;

        todo!("some of these entries are NaN");
        let texture_matrix = glam::mat4(
            glam::vec4(tm[0], tm[4], tm[8], 0.0),
            glam::vec4(tm[1], tm[5], tm[9], 0.0),
            glam::vec4(tm[2], tm[6], tm[10], 0.0),
            glam::vec4(tm[3], tm[7], tm[11], 1.0),
        );

        Ok(Self {
            scn0_camera_ref,
            scn0_light_ref,
            map_mode,
            enable_identity_matrix_effect,
            texture_matrix,
        })
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct LightingChannelFlags {
    pub material_color_enabled: bool,
    pub material_alpha_enabled: bool,
    pub ambient_color_enabled: bool,
    pub ambient_alpha_enabled: bool,
    pub raster_color_enabled: bool,
    pub raster_alpha_enabled: bool,
    #[bits(26)]
    _padding: u32,
}

impl Deserialize for LightingChannelFlags {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        Ok(Self::from_bits(word))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightChannelSettings {
    pub flags: LightingChannelFlags,
    pub material_color: glam::U8Vec4,
    pub ambient_color: glam::U8Vec4,
    pub color_light_channel_control: u32,
    pub alpha_light_channel_control: u32,
}

impl Deserialize for LightChannelSettings {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let flags = LightingChannelFlags::deserialize(reader)?;
        let material_color = glam::U8Vec4::from_array(reader.read_u8_array::<4>()?);
        let ambient_color = glam::U8Vec4::from_array(reader.read_u8_array::<4>()?);
        let color_light_channel_control = reader.read_u32::<BigEndian>()?;
        let alpha_light_channel_control = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            flags,
            material_color,
            ambient_color,
            color_light_channel_control,
            alpha_light_channel_control,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialBuf {
    pub index: u32,
    pub flags: MaterialFlags,
    pub tex_gens: u8,
    pub light_channels: u8,
    pub shader_stages: u8,
    pub indirect_textures: u8,
    pub culling: Culling,
    pub depth_test: u8,
    pub lightset_index: u8,
    pub fog_index: u8,
    pub indirect_methods: [IndirectMethod; 4],
    pub light_normal_map_refs: [u8; 4],
    pub shader_offset: i32,
    pub texture_count: u32,
    pub layer_offset: i32,
    pub fur_data_offset: i32,
    pub user_data: i32,
    pub display_list_offset: i32,
    pub used_texture_maps: UsedTextureMaps,
    pub precompiled_texture_code: [u8; 160],
    pub used_palettes: UsedPalettes,
    pub precompiled_palette_code: [u8; 160],
    pub layer_settings: LayerSettings,
    pub texture_matrix_mode: TextureMatrixMode,
    pub layer_coordinates: [LayerCoordinates; 8],
    pub texture_matrix_settings: [TextureMatrixSettings; 8],
    pub light_channel_settings: [LightChannelSettings; 2],
}

impl Deserialize for MaterialBuf {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let length = reader.read_u32::<BigEndian>()?;
        let mdl0_offset = reader.read_i32::<BigEndian>()?;
        let name_offset = reader.read_i32::<BigEndian>()?;
        let index = reader.read_u32::<BigEndian>()?;
        let flags = MaterialFlags::deserialize(reader)?;
        let tex_gens = reader.read_u8()?;
        let light_channels = reader.read_u8()?;
        let shader_stages = reader.read_u8()?;
        let indirect_textures = reader.read_u8()?;
        let culling = Culling::deserialize(reader)?;
        todo!("depth test is probably a bitfield that is not documented");
        let depth_test = reader.read_u8()?;
        let lightset_index = reader.read_u8()?;
        let fog_index = reader.read_u8()?;
        let _padding = reader.read_u8()?;

        let indirect_methods = [
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?,
            IndirectMethod::deserialize(reader)?,
        ];

        let light_normal_map_refs = reader.read_u8_array::<4>()?;
        let shader_offset = reader.read_i32::<BigEndian>()?;
        let texture_count = reader.read_u32::<BigEndian>()?;
        let layer_offset = reader.read_i32::<BigEndian>()?;
        let fur_data_offset = reader.read_i32::<BigEndian>()?;
        let user_data = reader.read_i32::<BigEndian>()?;
        let display_list_offset = reader.read_i32::<BigEndian>()?;
        let used_texture_maps = UsedTextureMaps::deserialize(reader)?;
        let precompiled_texture_code = reader.read_u8_array::<160>()?;
        let used_palettes = UsedPalettes::deserialize(reader)?;
        let precompiled_palette_code = reader.read_u8_array::<160>()?;
        let layer_settings = LayerSettings::deserialize(reader)?;
        let texture_matrix_mode = TextureMatrixMode::deserialize(reader)?;

        let layer_coordinates = [
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
            LayerCoordinates::deserialize(reader)?,
        ];

        let texture_matrix_settings = [
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
            TextureMatrixSettings::deserialize(reader)?,
        ];

        let light_channel_settings = [
            LightChannelSettings::deserialize(reader)?,
            LightChannelSettings::deserialize(reader)?,
        ];

        todo!("further deserialization");

        Ok(Self {
            index,
            flags,
            tex_gens,
            light_channels,
            shader_stages,
            indirect_textures,
            culling,
            depth_test,
            lightset_index,
            fog_index,
            indirect_methods,
            light_normal_map_refs,
            shader_offset,
            texture_count,
            layer_offset,
            fur_data_offset,
            user_data,
            display_list_offset,
            used_texture_maps,
            precompiled_texture_code,
            used_palettes,
            precompiled_palette_code,
            layer_settings,
            texture_matrix_mode,
            layer_coordinates,
            texture_matrix_settings,
            light_channel_settings,
        })
    }
}

#[tracing::instrument(skip_all, fields(parent_id))]
pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    header_start: u32,
    parent_id: VirtualNodeId,
    node_map: &VirtualNodeMap,
) -> EditorResult<VirtualNodeBody> {
    let section_index = IndexGroup::deserialize(reader)?;

    let mut materials = Vec::with_capacity(section_index.entries.len() - 1);
    for entry in &section_index.entries[1..] {
        let name = section_index.get_entry_name(reader, entry)?;
        let data_start = section_index.get_entry_data_start(entry);

        reader.set_position(data_start as u64);

        let material = MaterialBuf::deserialize(reader)?;
        let id = node_map.next_id();
        let node = VirtualNode {
            label: name,
            id,
            parent: Some(parent_id),
            kind: VirtualNodeKind::Materials,
            body: Deferred::evaluated(VirtualNodeBody {
                children: Vec::new(),
                inspectable: Some(Box::new(material)),
            }),
        };

        node_map.insert(id, node);
        materials.push(id);
    }

    Ok(VirtualNodeBody {
        children: materials,
        inspectable: None,
    })
}

// #[tracing::instrument(skip_all, fields(parent_id))]
// pub fn deserialize_virtual(
//     reader: &mut RefCursor<[u8]>,
//     header_start: u32,
//     parent_id: VirtualNodeId,
//     node_map: &VirtualNodeMap,
// ) -> EditorResult<VirtualNodeBody> {
//     let section_index = IndexGroup::deserialize(reader)?;

//     let mut models = Vec::with_capacity(section_index.entries.len() - 1);
//     for entry in &section_index.entries[1..] {
//         let name = section_index.get_entry_name(reader, entry)?;
//         let data_start = section_index.get_entry_data_start(entry);

//         reader.set_position(data_start as u64);

//         let normals = NormalBuf::deserialize(reader, header_start)?;

//         let id = node_map.next_id();
//         let node = VirtualNode {
//             label: name,
//             id,
//             kind: VirtualNodeKind::Normals,
//             parent: Some(parent_id),
//             body: Deferred::evaluated(VirtualNodeBody {
//                 children: Vec::new(),
//                 inspectable: Some(Box::new(normals)),
//             }),
//         };
//         node_map.insert(id, node);
//         models.push(id);
//     }

//     Ok(VirtualNodeBody {
//         children: models,
//         inspectable: None,
//     })
// }
