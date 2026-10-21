use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, try_from};
use slipstream_shared::error::InvalidInputError;
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[try_from(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum AreaShape {
    Box = 0x00,
    Cylinder = 0x01
}

impl AreaShape {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let byte = reader.read_u8()?;
        Self::try_from(byte)
    }
}

pub trait AreaFromRaw: Sized {
    fn from_raw(raw: u16) -> SlipstreamResult<Self>;
    fn to_raw(&self) -> u16;
}

impl AreaFromRaw for bool {
    fn from_raw(raw: u16) -> SlipstreamResult<Self> {
        Ok(raw != 0)
    }

    fn to_raw(&self) -> u16 {
        *self as u16
    }
}

impl AreaFromRaw for u8 {
    fn from_raw(raw: u16) -> SlipstreamResult<Self> {
        let conv = raw.try_into().map_err(|_| {
            SlipstreamError::from(InvalidInputError {
                reason: format!("AREA setting {raw} out of range for byte."),
                ..Default::default()
            })
        })?;
        Ok(conv)
    }

    fn to_raw(&self) -> u16 {
        *self as u16
    }
}

impl AreaFromRaw for u16 {
    fn from_raw(raw: u16) -> SlipstreamResult<Self> {
        Ok(raw)
    }

    fn to_raw(&self) -> u16 {
        *self
    }
}

macro_rules! area_types {
    (
        $(
            $(#[$meta:meta])*
            $id:literal => $name:ident $({
                $(
                    $(#[$field_meta:meta])*
                    $slot:literal => $field:ident: $field_ty:ty
                ),*
            })?
        ),*
    ) => {
        #[derive(Debug, Clone, PartialEq, Inspect)]
        pub enum AreaType {
            $(
                $(#[$meta])*
                $name $({
                   $(
                       $(#[$field_meta])*
                       $field: $field_ty
                   ),*
                })?
            ),*
        }

        impl AreaType {
            pub fn from_raw(id: u8, settings: [u16; 2], camera_id: u8, route_id: u8, enemy_point_id: u8) -> SlipstreamResult<Self> {
                let values = [
                    settings[0],
                    settings[1],
                    camera_id as u16,
                    route_id as u16,
                    enemy_point_id as u16
                ];

                Ok(match id {
                    $(
                        $id => Self::$name $({
                            $(
                                $field: <$field_ty>::from_raw(values[$slot])?
                            ),*
                        })?
                    ),*,
                    _ => return Err(InvalidInputError {
                        reason: format!("Invalid AREA type: {id:#02x}"),
                        ..Default::default()
                    }.into())
                })
            }

            pub fn to_raw(&self) -> (u8, [u16; 5]) {
                todo!()
            }
        }
    }
}

area_types! {
    #[inspect(
        tooltip = "Found in all tracks and arenas. The related camera is enabled when the player enters this area. It is used when the player has finished or when waiting for the next online race."
    )]
    0x00 => Camera {
        2 => camera_id: u8
    },
    0x01 => EnvironmentEffect {
        0 => use_kareha_up: bool
    },
    #[inspect(rename = "BFG Entry Swapper")]
    0x02 => BfgEntrySwapper {
        0 => used_entry: u16
    },
    #[inspect(tooltip = "Causes moving terrain in a KCL file to move.")]
    0x03 => MovingRoad,
    #[inspect(
        tooltip = "Forces a CPU racer to recalculate its route. This is typically used where players can take multiple routes."
    )]
    0x04 => DestinationPoint,
    0x05 => MinimapControl,
    #[inspect(rename = "BBLM Swapper")]
    #[inspect(
        tooltip = "Determines which BBLM file to use. Setting this to some integer `n` will tell the game to use file `posteffect.bblmn`."
    )]
    0x06 => BblmSwapper {
        0 => used_entry: u16
    },
    #[inspect(tooltip = "Defines an area where flying boos (b_teresa) should appear.")]
    0x07 => FlyingBoos,
    #[inspect(tooltip = "Groups objects together for the GroupUnloading area.")]
    0x08 => ObjectGrouper {
        0 => group_id: u8
    },
    #[inspect(tooltip = "While being in this area, the group with the same ID is unloaded.")]
    0x09 => GroupUnloading {
        0 => group_id: u8
    },
    #[inspect(
        tooltip = "Adds new fall boundaries to the track without modifying the original KCL file. This is mainly used for tournaments."
    )]
    0x0a => FallBoundary
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Area {
    pub shape: AreaShape,
    pub ty: AreaType,
    pub priority: u8,
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3
}

impl Area {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let shape = AreaShape::deserialize(reader)?;
        let ty = reader.read_u8()?;
        let camera_id = reader.read_u8()?;
        let priority = reader.read_u8()?;
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let scale = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let settings = reader.read_u16_array::<2, BigEndian>()?;
        let route_id = reader.read_u8()?;
        let enemy_point_id = reader.read_u8()?;

        let ty = AreaType::from_raw(ty, settings, camera_id, route_id, enemy_point_id)?;

        Ok(Self {
            shape,
            ty,
            priority,
            position,
            rotation,
            scale
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Areas {
    pub areas: Vec<Area>
}

impl Areas {
    pub const NAME: &str = "AREA";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut areas =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            areas.push(Area::deserialize(reader)?);
        }

        Ok(Self { areas })
    }
}

impl Visitable for Areas {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_areas(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_areas_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
