use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield, try_from};
use slipstream_shared::error::InvalidInputError;
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

pub trait GeoSetting: Sized {
    fn into_raw(&self) -> u16;
    fn from_raw(setting: u16) -> SlipstreamResult<Self>;
}

impl GeoSetting for u16 {
    fn into_raw(&self) -> u16 {
        *self
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Ok(setting)
    }
}

impl GeoSetting for bool {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Ok(setting != 0)
    }
}

macro_rules! geo_objects {
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
       pub enum GeoObjectId {
           $(
               $name $({
                   $($field: $field_ty),*
               })?
           ),*
       }

       impl GeoObjectId {
           pub fn from_raw(id: u16, settings: [u16; 8]) -> SlipstreamResult<Self> {
               Ok(match id {
                   $(
                       $id => Self::$name $({
                           $($field: <$field_ty>::from_raw(settings[$slot])?),*
                       })?,
                   )*
                   _ => return Err(SlipstreamError::from(InvalidInputError {
                       reason: format!("Invalid Geo Object id: {id}"),
                       ..Default::default()
                   }))
               })
           }

           pub fn to_raw(&self) -> (u16, [u16; 8]) {
               match self {
                   $(
                       Self::$name $({ $($field),* })? => {
                           #[allow(unused_mut)]
                           let mut settings = [0u16; 8];
                           $($( settings[$slot] = $field.into_raw(); )*)?
                           ($id, settings)
                       }
                   ),*
               }
           }
       }
   }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[try_from(u16)]
pub enum ThwompRouteBehavior {
    StompOnEveryPoint = 0x01,
    LeftAndRightStomp = 0x02,
    LeftAndRightGrounded = 0x03
}

impl GeoSetting for ThwompRouteBehavior {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[try_from(u16)]
pub enum FallBoundary {
    AirFall = 0x00,
    Water = 0x01,
    Lava = 0x02,
    IcyWater = 0x03,
    LavaNoGfx = 0x04,
    BurningAirFall = 0x05,
    Quicksand = 0x06,
    ShortFall = 0x07
}

impl GeoSetting for FallBoundary {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[try_from(u16)]
pub enum WallFlag {
    Normal = 0x00,
    Rock = 0x01,
    Metal = 0x02,
    Wood = 0x03,
    Ice = 0x04,
    Bush = 0x05,
    Rope = 0x06,
    Rubber = 0x07
}

impl GeoSetting for WallFlag {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum StartDelay {
    Delayed(u16),
    Immediate
}

impl GeoSetting for StartDelay {
    fn into_raw(&self) -> u16 {
        match self {
            Self::Delayed(x) => *x,
            Self::Immediate => 0xf0
        }
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Ok(match setting {
            0xf0 => Self::Immediate,
            _ => Self::Delayed(setting)
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[try_from(u16)]
pub enum EnemyLineEntryFlag {
    DoNotEnter = 0x0,
    Enter = 0x1
}

impl GeoSetting for EnemyLineEntryFlag {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
#[try_from(u16)]
pub enum TopmanBehavior {
    Unknown = 0x00,
    #[inspect(tooltip = "Ignores the player")]
    Coward = 0x01,
    #[inspect(tooltip = "Attacks the player")]
    Aggressive = 0x02
}

impl GeoSetting for TopmanBehavior {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Inspect)]
pub enum Item {
    Default = 0x000,
    Banana = 0x001,
    Mushroom = 0x002,
    TripleMushrooms = 0x003,
    Star = 0x004,
    TripleGreenShells = 0x005,
    BananOrMushroom = 0x006,
    GreenShell = 0x007,
    Bobomb = 0x008,
    RedShell = 0x009,
    MegaMushroom = 0x00a,
    /// Mushroom if another thunder cloud is in play.
    ThunderCloud = 0x00b,
    /// Star (95%) or mushroom (5%)
    StarOrMushroom = 0x00c,
    /// Grants a mushroom (roulette shows green shell)
    Empty0 = 0x00d,
    /// Grants a mushroom (roulette shows green shell)
    Empty1 = 0x00e,
    /// Grants a mushroom (roulette shows green shell)
    Empty2 = 0x00f,
    /// Grants a mushroom (roulette shows green shell)
    Empty3 = 0x010,
    /// Same as [`Default`]
    Empty4 = 0x255
}

impl GeoSetting for Item {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimingLottery {
    /// Roulette spin is normal, respawn time is 150.
    Normal150 = 0,
    /// Roulette spin is short, respawn time is 150.
    Short150 = 2,
    /// Roulette spin is short, respawn time is 150.
    Short150_2 = 3,
    /// Roulette spin is short, respawn time is 600.
    Short600 = 4,
    /// Roulette spin is normal speed, respawn time is 600.
    Normal600 = 5,
    /// Roulette spin is normal speed, respawn time is 900.
    Normal900 = 6
}

impl GeoSetting for TimingLottery {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum ChainChompStartBehavior {
    Wait = 0x00,
    Release = 0x01
}

impl GeoSetting for ChainChompStartBehavior {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum CoinMode {
    Battle = 0x00,
    TournamentOrMission = 0x01
}

impl GeoSetting for CoinMode {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

/// Not exactly sure what this does yet.
///
/// [Wiiki link](https://mkwiiki.org/wiki/Object/coin#Setting_2:_Start_or_Respawn_Place).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum CoinSpawnPlace {
    Start(u16),
    Respawn
}

impl GeoSetting for CoinSpawnPlace {
    fn into_raw(&self) -> u16 {
        match self {
            Self::Start(x) => *x,
            Self::Respawn => u16::MAX
        }
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Ok(match setting {
            u16::MAX => Self::Respawn,
            _ => Self::Start(setting)
        })
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum CoinSpawnBehavior {
    /// Coins are inserted at a place and snap to the ground.
    InPlaceSnapGround = 0x00,
    /// Coins are inserted at a place and float in the air.
    InPlaceFloat = 0x01,
    /// Coins fall down from the sky and keep their X and Z value.
    FallFixed = 0x02,
    /// Coins fall down from the sky. When they fall on a slope, they ricochet.
    FallRicochet = 0x03
}

impl GeoSetting for CoinSpawnBehavior {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum WaterfallType {
    A = 0x00,
    B = 0x01,
    C = 0x02
}

impl GeoSetting for WaterfallType {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum PhysicalItem {
    GreenShell = 0x00,
    RedShell = 0x01,
    Banana = 0x02,
    Mushroom = 0x03,
    Star = 0x04
}

impl GeoSetting for PhysicalItem {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum TruckTexture {
    MooMoo = 0x00,
    Fruit = 0x01,
    Factory = 0x02
}

impl GeoSetting for TruckTexture {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum CarBodyColor {
    Blue = 0x00,
    Red = 0x01,
    Yellow = 0x02
}

impl GeoSetting for CarBodyColor {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum PylonColor {
    Red = 0x00,
    Blue = 0x01,
    Yellow = 0x02
}

impl GeoSetting for PylonColor {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum CrabViewDirection {
    Left = 0x00,
    Right = 0x01
}

impl GeoSetting for CrabViewDirection {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum BlockFallTime {
    Inactive,
    Immediate,
    Delayed(u16)
}

impl GeoSetting for BlockFallTime {
    fn into_raw(&self) -> u16 {
        match self {
            Self::Inactive => 0,
            Self::Immediate => 4,
            Self::Delayed(x) => *x
        }
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Ok(match setting {
            0 => Self::Inactive,
            4 => Self::Immediate,
            _ => Self::Delayed(setting)
        })
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum ChairliftModel {
    Chairlift00 = 0x00,
    Chairlift01 = 0x01
}

impl GeoSetting for ChairliftModel {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum WaluigiFlashType {
    A = 0x00,
    B = 0x01,
    C = 0x02
}

impl GeoSetting for WaluigiFlashType {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum FunkyFlashType {
    A = 0x00,
    B = 0x01
}

impl GeoSetting for FunkyFlashType {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum SmokeVariant {
    DefaultPermanent = 0x00,
    Fluffy = 0x01,
    Default = 0x02,
    Thick = 0x03,
    Default2 = 0x04,
    Explode = 0x05
}

impl GeoSetting for SmokeVariant {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

#[try_from(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Inspect)]
pub enum KoopaFireFunction {
    Decoration = 0x0,
    Obstacle = 0x1
}

impl GeoSetting for KoopaFireFunction {
    fn into_raw(&self) -> u16 {
        *self as u16
    }

    fn from_raw(setting: u16) -> SlipstreamResult<Self> {
        Self::try_from(setting)
    }
}

geo_objects! {
    /// Invisible wall above barriers.
    ///
    /// ## Note:
    /// The Y and Z scale axes are swapped for this object!
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/airblock)
    0x001 => AirBlock,
    /// The sea on GCN Peach Beach.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/Psea)
    0x002 => PSea,
    /// Generates a lens flare object when looking at the sun.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/lensFX)
    0x003 => LensFx,
    /// The rising and dropping sea on Delfino Pier. The `pocha` object is required if the player
    /// can interact with it.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/venice_nami)
    0x004 => VeniceNami
    {
        // Rising speed in odd stages.
        0 => odd_rising_speed: u16,
        1 => stage5_target_height: u16,
        2 => stage1_target_height: u16,
        3 => stage2_frame_count: u16,
        4 => stage3_target_height: u16,
        5 => stage4_frame_count: u16,
        // Rising speed in even stages.
        6 => even_rising_speed: u16
    },
    /// Generates a river sound effect.
    ///
    /// Only works on slots 3.2, 4.2, 5.2 and 7.3.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_river)
    0x005 => SoundRiver,
    /// Generates a waterfall sound effect.
    ///
    /// Only works on slots 3.2, 4.2 and 7.3.
    0x006 => SoundWaterFall,
    /// Generates a water splash effect when a player touches KCL flag `0x10` with variants 1
    /// (water) or 3 (icy water that also needs the `ice` object).
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/pocha)
    0x007 => Pocha {
        0 => group_y_kcl_index: FallBoundary
    },
    /// Generates a lake sound effect.
    ///
    /// Only works on slots 2.2, 3.1, 5.1 and 8.2.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_lake)
    0x008 => SoundLake,
    /// Generates a louder waterfall sound effect.
    ///
    /// Only works on slots 3.2, 4.2 and 7.3.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_big_fall)
    0x009 => SoundBigFall,
    0x00a => SoundSea,
    0x00b => SoundFountain,
    0x00c => SoundVolcano,
    0x00d => SoundAudience,
    0x00e => SoundBigRiver,
    0x00f => SoundSandFall,
    0x010 => SoundLift,
    0x011 => PochaYogan {
        0 => group_y_kcl_index: FallBoundary
    },
    0x012 => Entry,
    0x013 => PochaMori {
        0 => group_y_kcl_index: FallBoundary
    },
    0x014 => ElineControl {
        0 => entry_flag: EnemyLineEntryFlag,
        1 => time_to_next_controller: u16,
        2 => controller_id: u16,
        3 => next_controller_id: u16,
        7 => enph_section: u16
    },
    0x015 => SoundMii,
    0x016 => BegomanManager {
        0 => manager_id: u16,
        1 => topmen_number: u16,
        2 => first_topman_id: u16,
        3 => topman_behavior: TopmanBehavior,
        /// Number of topmen to control + number of topment controlled by the previous manager.
        4 => topmen_number_additive: u16
    },
    0x017 => Ice,
    0x018 => Startline2d,
    0x065 => Itembox {
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        4 => timing_lottery: TimingLottery,
        5 => height_offset: u16
    },
    0x066 => DummyPole {
        0 => wall_kcl_variant: WallFlag
    },
    0x067 => Flag,
    0x068 => FlagBlend,
    0x069 => GndSphere,
    0x06a => GndTrapezoid,
    0x06b => GndWave1,
    0x06c => GndWave2,
    0x06c => GndWave3,
    0x06d => GndWave4,
    0x06e => Sun,
    0x070 => Woodbox {
        1 => item: Item,
        /// A chance between 0 and 100 to spawn an item when this crate is hit.
        2 => item_chance: u16,
        3 => disable_respawn: bool
    },
    0x071 => KmoonZ,
    0x072 => SunDs {
        0 => speed: u16,
        1 => delay: StartDelay
    },
    0x073 => Coin {
        0 => mode: CoinMode,
        1 => start_or_respawn_place: CoinSpawnPlace,
        2 => spawn_behavior: CoinSpawnBehavior
    },
    0x074 => Ironbox,
    0x075 => ItemDirect {
        0 => ty: PhysicalItem
    },
    0x076 => SItemBox {
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        5 => timing_lottery: TimingLottery,
        6 => height_offset: u16
    },
    0x077 => PileCoin,
    0x0c9 => FItemBox {
        0 => start_speed: u16,
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        4 => route_start_point: u16,
        5 => timing_lottery: TimingLottery
    },
    0x0ca => MashaBalloonGc {
        0 => speed: u16,
        1 => max_y_difference: u16
    },
    /// The piranha plant wall from GCN Waluigi Stadium. It is combined with `WlDokanGc`.
    0x0cb => WlWallGc {
        /// How long the object should stay still at its initial position.
        0 => time_stationary_at_init: u16,
        1 => time_stationary_at_dest: u16,
        2 => speed: u16,
        3 => move_distance: u16,
        4 => delay: u16
    },
    0x0cc => CarA1 {
        0 => speed: u16,
        1 => acceleration: u16,
        2 => time_of_pause: u16
    },
    0x0cd => Basabasa {
        0 => speed: u16,
        1 => next_group_delay: u16,
        2 => group_size: u16,
        3 => route_spread: u16,
        4 => max_height: u16,
        5 => entity_distance: u16,
        6 => start_delay: u16
    },
    0x0ce => HeyHoShipGba {
        0 => speed: u16,
        1 => vertical_movement: u16
    },
    0x0cf => KoopaBall,
    0x0d0 => KartTruck {
        0 => route_start: u16,
        1 => speed0: u16,
        2 => speed1: u16,
        3 => texturing: TruckTexture
    },
    0x0d1 => CarBody {
        0 => route_start: u16,
        1 => speed0: u16,
        2 => speed1: u16,
        3 => pat_anim_frame: u16
    },
    0x0d2 => SkyShip {
        0 => speed: u16,
        1 => vertical_movement: u16
    },
    /// A wooden box that ollows a route.
    0x0d3 => WWoodBox {
        0 => speed: u16,
        1 => item: Item,
        2 => item_chance: u16,
        4 => start_delay: u16,
        5 => interval: u16,
        6 => max_box_count: u16
    },
    /// Multiple item boxes that follow a route from a starting point to an ending point.
    0x0d4 => WItemBox {
        0 => speed: u16,
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        4 => start_delay: u16,
        5 => interval: u16,
        6 => box_count: u16
    },
    0x0d5 => WItemBoxLine {
        0 => speed: u16,
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        4 => start_delay: u16,
        5 => interval: u16,
        6 => box_count: u16
    },
    0x0d6 => VolcanoBall1 {
        0 => speed: u16,
        1 => start_delay: u16,
        2 => interval: u16,
        3 => extinguish_time: u16
    },
    0x0d7 => PenguinS {
        0 => speed: u16
    },
    0x0d8 => PenguinM {
        0 => speed: u16
    },
    0x0d9 => PenguinL {
        0 => speed: u16
    },
    0x0da => CastleBalloon1 {
        0 => speed: u16,
        1 => vertical_movement: u16
    },
    0x0db => DossunC {
        0 => speed: u16,
        1 => route_behavior: ThwompRouteBehavior,
        2 => start_delay: StartDelay,
        3 => sleep_time: u16
    },
    0x0dc => DossunCSoko,
    0x0dd => Boble {
        0 => speed: u16
    },
    0x0de => KBombCar {
        0 => route_start: u16,
        1 => speed0: u16,
        2 => speed1: u16
    },
    0x0df => KBombCarDummy,
    0x0e0 => CarBodyDummy,
    0x0e1 => KartTruckDummy,
    0x0e2 => Hanachan {
        0 => speed: u16
    },
    0x0e3 => Seagull {
        0 => speed: u16,
        1 => count: u16
    },
    0x0e4 => Moray {
        0 => speed: u16
    },
    0x0e5 => Crab {
        0 => speed: u16,
        1 => view_direction: CrabViewDirection
    },
    0x0e6 => BasabasaDummy,
    0x0e7 => CarA2 {
        0 => speed: u16,
        1 => acceleration: u16,
        2 => pause_time: u16
    },
    0x0e8 => CarA3 {
        0 => speed: u16,
        1 => acceleration: u16,
        2 => pause_time: u16
    },
    0x0e9 => Hwanwan {
        0 => speed: u16,
        1 => item_box_count: u16
    },
    0x0ea => HeyHoBallGba {
        0 => ship_route_id: u16,
        1 => fly_time: u16
    },
    0x0eb => Twanwan {
        0 => speed: u16,
        1 => start_behavior: ChainChompStartBehavior,
        2 => sine_amplitude: u16,
        3 => sine_period: u16,
        4 => start_delay: u16,
        5 => target_start_delay: u16
    },
    0x0ec => CruiserR {
        0 => speed: u16
    },
    0x0ed => Bird {
        0 => speed: u16,
        1 => count: u16
    },
    0x0ee => SinItemBox {
        0 => cycle_time: u16,
        1 => item_player: Item,
        2 => item_enemy: Item,
        3 => disable_shadow: bool,
        4 => start_delay: u16,
        5 => timing_lottery: TimingLottery
    },
    0x0ef => TwanwanUe,
    0x0f0 => BossHanachan {
        0 => primary_speed: u16,
        1 => angry_wait_delay: u16,
        2 => secondary_speed: u16,
        /// Route point that must be passed to set Wiggler speed to `secondary_speed`.
        3 => secondary_trigger: u16,
        4 => angry_speed: u16,
        /// Route point that must be passed to set Wiggler speed to `angry_speed`.
        5 => angry_trigger: u16,
        6 => angry_cooldown: u16
    },
    0x0f1 => KDossunC,
    0x0f2 => BossHanachanHead,
    0x0f3 => KBombCar1,
    0x012d => Dummy,
    0x012e => DokanSfc,
    0x12f => CastleTree1,
    0x130 => CastleTree1C,
    0x131 => CastleTree2,
    0x132 => CastleFlower1,
    0x133 => MarioTreeGc,
    0x134 => MarioTreeGcC,
    0x135 => DonkyTree1Gc,
    0x136 => DonkyTree2Gc,
    0x137 => PeachTreeGc,
    0x138 => PeachTreeGcC,
    0x138 => NpcMiiA,
    0x13a => NpcMiiB,
    0x13b => NpcMiiC,
    0x13c => ObakeBlockSfcC {
        1 => fall_time: BlockFallTime,
        2 => additional_delay: u16
    },
    0x13d => WlArrowGc,
    0x13e => WlScreenGc,
    0x13f => WlDokanGc,
    0x140 => MarioGo64C,
    0x141 => PeachHunsuiGc,
    0x142 => KinokoT1,
    0x143 => KinokoT2,
    0x144 => Pylon01 {
        0 => color: PylonColor
    },
    0x145 => PalmTree,
    0x146 => Parasol,
    0x147 => Cruiser,
    0x148 => KStickLift00,
    /// Manages the chairlift in DK Summit.
    0x149 => HeyHo2 {
        0 => model: ChairliftModel
    },
    0x14a => HeyHoTreeGbaC,
    0x14b => MFaceBill,
    0x14c => TruckChimSmk {
        0 => start_delay: u16,
        1 => variant: SmokeVariant
    },
    0x14d => MiiObj01,
    0x14e => MiiObj02,
    0x14f => MiiObj03,
    0x150 => GardenTreeDs,
    0x151 => GardenTreeDsC,
    0x152 => FlagA1,
    0x153 => FlagA2,
    0x154 => FlagB1,
    0x155 => FlagB2,
    0x156 => FlagA3,
    0x157 => DkTreeA64,
    0x158 => DkTreeA64C,
    0x159 => DkTreeB64,
    0x15a => DkTreeB64C,
    0x15b => TownTreeDSc,
    0x15c => Piston,
    0x15d => OilSfc,
    0x15e => DkMarutaGcC,
    0x15f => DkRopeGcC,
    0x160 => MiiBalloon,
    0x161 => Windmill,
    0x162 => Dossun,
    0x163 => TownTreeDs,
    0x164 => KStickEtc,
    0x165 => MonteA {
        1 => rotation_and_clapping: bool,
        2 => attention_distance: u16,
        3 => trigger_distance: u16
    },
    0x166 => MiiStatueM1,
    0x167 => ShMiiObj01,
    0x168 => ShMiiObj02,
    0x169 => ShMiiObj03,
    0x16a => Hanabi,
    0x16b => MiiPoster,
    0x16c => DkMiiObj00,
    0x16d => LightHouse,
    0x16e => RParasol {
        0 => is_closed: bool
    },
    0x16f => ObakeBlock2SfcC {
        1 => fall_time: BlockFallTime,
        2 => additional_delay: u16
    },
    0x170 => ObakeBlock3SfcC {
        1 => fall_time: BlockFallTime,
        2 => additional_delay: u16
    },
    0x171 => KoopaFigure,
    0x172 => Pukupuku,
    0x173 => VStand1,
    0x174 => VStand2,
    0x175 => LeafEffect,
    /// Leaf pile from Maple Treeway.
    0x176 => Karehayama {
        1 => item: Item,
        2 => item_chance: u16
    },
    0x177 => EarthRing,
    0x178 => SpaceSun,
    0x179 => BlackHole,
    0x17a => StarRing,
    0x17b => MObjKanban,
    0x17c => MiiStatueL1,
    0x17d => MiiStatueD1,
    0x17e => MiiSphinxY1,
    0x17f => MiiSphinxY2,
    0x180 => FlagA5,
    0x181 => CarB,
    0x182 => FlagA4,
    0x183 => Steam {
        1 => start_delay: u16,
        2 => duration: u16,
        3 => interval: u16
    },
    0x184 => Alarm,
    0x185 => GroupMonteA,
    0x186 => MiiStatueL2,
    0x187 => MiiStatueD2,
    0x188 => MiiStatueP1,
    0x189 => SentakuDs,
    0x18a => FksScreenWii,
    0x18b => KoopaFigure64 {
        0 => trigger_distance: u16,
        1 => fire_function: KoopaFireFunction,
        2 => start_delay: StartDelay
    },
    0x18c => BTeresa,
    0x18d => MiiStatueDk1,
    0x18e => MiiKanban,
    0x18f => BgTeresaSfc,
    0x191 => Kuribo {
        0 => respawn_time: u16,
        1 => speed: u16,
        2 => animation_speed: u16
    },
    0x192 => Choropu {
        0 => speed: u16,
        1 => start_delay: u16
    },
    0x193 => Cow {
        0 => followers: u16,
        2 => start_delay: u16
    },
    0x194 => PakkunF {
        0 => idle_time: u16
    },
    0x195 => WlFirebarGc {
        0 => fireballs_per_arm: u16,
        1 => rotations_per_6mins: u16,
        2 => fireball_distance: u16,
        3 => arm_count: u16
    },
    0x196 => Wanwan {
        0 => chain_length: u16,
        3 => snap_x: u16,
        4 => snap_z: u16,
        5 => wander_time: u16,
        6 => wedge_size: u16
    },
    0x197 => Poihana {
        0 => pat_anim_index: u16,
        1 => trigger_distance: u16
    },
    0x198 => DkRockGc {
        0 => interval: u16,
        1 => min_cycle_time: u16,
        2 => speed: u16,
        3 => bounce_factor: u16
    },
    0x199 => Sanbo {
        0 => speed: u16
    },
    0x19a => Choropu2 {
        0 => hide_time: u16,
        1 => start_delay: u16
    },
    0x19b => TruckWagon {
        0 => speed: u16,
        1 => interval: u16,
        2 => start_delay: u16,
        3 => disable_collision: bool
    },
    0x19c => HeyHo {
        0 => speed: u16,
        1 => pat_animation_frame: u16
    },
    0x19d => Press {
        1 => start_delay: u16,
        2 => sleep_time: u16
    },
    0x19e => PressSoko,
    0x19f => Pile,
    0x1a0 => ChoropuGround,
    0x1a1 => WlFireRingGc {
        0 => fireballs_per_arm: u16,
        1 => rotations_per_6mins: u16,
        2 => pulsation_factor: u16,
        3 => radius: u16
    },
    0x1a2 => PakkunDokan,
    0x1a3 => BegomanSpike {
        0 => id: u16,
        1 => roam1_radius: u16,
        2 => roam2_start_delay: u16,
        3 => roam2_radius: u16,
        4 => roam3_start_delay: u16,
        5 => roam3_radius: u16,
        6 => roam4_start_delay: u16,
        7 => roam4_radius: u16
    },
    0x1a4 => FireSnake {
        0 => route_id: u16,
        1 => alive_time: u16
    },
    0x1a5 => KoopaFireBar {
        0 => fireballs_per_arm: u16,
        1 => rotations_per_6mins: u16,
        2 => fireball_distance: u16,
        3 => arm_count: u16
    },
    0x1a6 => EPropeller {
        0 => rotations_per_12sec: u16,
        1 => is_counter_clockwise: bool
    },
    0x1a7 => DcPillarC,
    0x1a8 => FireSnakeV {
        0 => start_delay: u16,
        1 => interval: u16,
        2 => landing_distance: u16
    },
    0x1a9 => HoneBall,
    0x1aa => PunchiPakkun,
    0x1ab => SanboBig,
    0x1ac => SanboBig2,
    0x1f5 => KinokoUd {
        0 => texturing: u16,
        1 => vertical_movement: u16,
        2 => cycle_time: u16,
        3 => cycle_shift: u16,
        4 => sleep_time: u16
    },
    0x1f6 => KinokoBend {
        0 => texturing: u16,
        1 => max_angle: u16,
        2 => cycle_time: u16,
        3 => cycle_shift: u16
    },
    0x1f7 => VolcanoRock1 {
        0 => use_second_variant: bool,
        1 => z_period: u16,
        2 => z_amplitude: u16,
        3 => z_offset: u16,
        4 => y_period: u16,
        5 => y_amplitude: u16
    },
    0x1f8 => BulldozerLeft {
        1 => move_distance: u16,
        2 => move_time: u16,
        3 => start_delay: u16,
        4 => sleep_time: u16
    },
    0x1f9 => BulldozerRight {
        1 => move_distance: u16,
        2 => move_time: u16,
        3 => start_delay: u16,
        4 => sleep_time: u16
    },
    0x1fa => KinokoNm {
        0 => texturing: u16
    },
    0x1fb => Crane {
        1 => cycle_period: u16,
        2 => amplitude: u16,
        3 => start_delay: u16
    },
    0x1fc => VolcanoPiece {
        0 => model_kcl_ref: u16,
        1 => start_delay_150cc: u16,
        2 => shake_time_150cc: u16,
        3 => start_delay_100cc: u16,
        4 => shake_time_100cc: u16,
        5 => start_delay_50cc: u16,
        6 => shake_time_50cc: u16,
        7 => fall_delay: u16
    },
    0x1fd => FlamePole {
        0 => cycle_time: u16,
        1 => start_delay: u16,
        2 => size_factor: u16
    },
    0x1fe => TwistedWay,
    0x1ff => TownBridgeDsc {
        0 => angle: u16,
        1 => move_time: u16,
        2 => up_time: u16,
        3 => down_time: u16
    },
    0x200 => DkShip64 {
        0 => speed: u16
    },
    0x201 => KinokoKuki,
    0x202 => DkTuribashiGcC,
    0x203 => FlamePollEff,
    0x204 => Aurora,
    0x205 => VeniceSaku {
        0 => start_delay: u16,
        1 => vertical_movement: u16,
        2 => speed: u16
    },
    0x206 => CasinoRoulette,
    0x207 => BossField01Obj1,
    0x208 => DcPillar {
        0 => start_delay_150cc: u16,
        1 => final_angle: u16,
        2 => start_delay_100cc: u16,
        3 => start_delay_50cc: u16
    },
    0x209 => DcSandcone {
        0 => grow_speed: u16,
        1 => final_size: u16,
        2 => start_delay: u16
    },
    0x20a => VeniceHasi {
        0 => start_delay: u16,
        1 => open_speed: u16
    },
    0x20b => VeniceGondola {
        0 => speed: u16
    },
    0x20c => Quicksand {
        0 => dont_use_thwomp: bool,
        1 => sand_shake_wave_length: u16,
        2 => time_until_end: u16,
        3 => push_pull_strength: u16
    },
    0x20d => BBlock {
        0 => model_kcl_ref: u16,
        1 => start_delay_150cc: u16,
        2 => shake_time_150cc: u16,
        3 => start_delay_100cc: u16,
        4 => shake_time_100cc: u16,
        5 => start_delay_50cc: u16,
        6 => shake_time_50cc: u16,
        7 => fall_delay: u16
    },
    0x20e => Ami,
    0x20f => MObjJump,
    0x210 => StarGate,
    0x211 => RmRing1 {
        0 => ring_id: u16,
        1 => shake_delay: u16,
        2 => shake_time: u16,
        3 => fall_delay: u16
    },
    0x212 => FlamePoleV {
        0 => cycle_length: u16,
        1 => start_delay: u16,
        2 => dropping_height: u16,
        3 => sleep_time: u16
    },
    0x213 => MObjSJump,
    0x214 => InsekiA {
        1 => is_counter_clockwise: bool,
        2 => rotation_speed: u16,
        3 => rotation_offset: u16
    },
    0x215 => InsekiB {
        1 => is_counter_clockwise: bool,
        2 => rotation_speed: u16,
        3 => rotation_offset: u16
    },
    0x216 => FlamePoleVBig {
        0 => time_outside_lava: u16,
        1 => start_delay: u16,
        2 => enable_time_outside_lava: bool
    },
    0x217 => MDush,
    0x218 => HpPipe,
    0x219 => DemoCol,
    0x21a => MObjSJump2,
    0x21b => MObjJump2,
    0x259 => DonkyCannonGc,
    0x25a => BeltEasy,
    0x25b => BeltCrossing,
    0x25c => BeltCurveA {
        1 => start_left_fast: bool,
        2 => switch_interval: u16,
        3 => switch_back_interval: u16
    },
    0x25d => BeltCurveB,
    0x25e => Escalator {
        1 => speed1: u16,
        2 => speed2_delay: u16,
        3 => speed2: u16,
        4 => speed3_delay: u16,
        5 => speed3: u16
    },
    0x25f => DonkyCannonWii,
    0x260 => EscalatorGroup {
        1 => speed1: u16,
        2 => speed2_delay: u16,
        3 => speed2: u16,
        4 => speed3_delay: u16,
        5 => speed3: u16
    },
    0x261 => TreeCannon,
    0x2bd => GroupEnemyB,
    0x2be => GroupEnemyC,
    0x2bf => Taimatsu,
    0x2c0 => TruckChimSmkW,
    0x2c1 => MStand,
    0x2c2 => DkMonitor,
    0x2c3 => GroupEnemyA,
    0x2c4 => FlagB3,
    0x2c5 => Spot,
    0x2c6 => GroupEnemyD,
    0x2c7 => FlagB4,
    0x2c8 => GroupEnemyE,
    0x2c9 => GroupMonteL,
    0x2ca => GroupEnemyF,
    0x2cb => FallBsA,
    0x2cc => FallBsB,
    0x2cd => FallBsC,
    0x2ce => VolSmk {
        1 => delay_150cc: u16,
        2 => active_time_150cc: u16,
        3 => delay_100cc: u16,
        4 => active_time_100cc: u16,
        5 => delay_50cc: u16,
        6 => active_time_50cc: u16
    },
    0x2cf => RidgeMii00,
    0x2d0 => FlashL,
    0x2d1 => FlashB,
    0x2d2 => FlashW {
        0 => ty: WaluigiFlashType
    },
    0x2d3 => FlashM,
    0x2d4 => FlashS {
        0 => ty: FunkyFlashType
    },
    0x2d5 => MiiSignNoko,
    0x2d6 => UtsuboDokan,
    0x2d7 => Spot64,
    0x2d8 => DemoEf,
    0x2d9 => FallMh,
    0x2da => FallY {
        0 => ty: WaterfallType
    },
    0x2db => DemoJugemu,
    0x2dc => GroupEnemyADemo,
    0x2dd => GroupMonteADemo,
    0x2de => VolFall,
    0x2df => MiiStatueM2,
    0x2e0 => RhMiiKanban,
    0x2e1 => MiiStatueL3,
    0x2e2 => MiiSignWario,
    0x2e3 => MiiStatueBl1,
    0x2e4 => MiiStatueBd1,
    0x2e5 => Kamifubuki,
    0x2e6 => Crescent64,
    // TODO: This might be misspelled?
    0x2e7 => MiiSighKino,
    0x2e8 => MiiObjD01,
    0x2e9 => MiiObjD02,
    0x2ea => MiiObjD03,
    0x2eb => MareA {
        1 => rotation_and_clapping: bool,
        2 => attention_distance: u16,
        3 => trigger_distance: u16
    },
    0x2ec => MareB {
        1 => rotation_and_clapping: bool,
        2 => attention_distance: u16,
        3 => trigger_distance: u16
    },
    0x2ed => EnvKareha,
    0x2ee => EnvFire,
    0x2ef => EnvSnow,
    0x2f0 => MObjStart,
    0x2f1 => EnvKarehaUp,
    0x2f2 => MObjKanbanY,
    0x2f3 => DkFalls
}

/// An object that can be placed on the track through a KMP file.
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct GeoObject {
    /// The type of object and its settings.
    pub object_id: GeoObjectId,
    /// Position of the object.
    pub position: glam::Vec3,
    /// Rotation of the object.
    ///
    /// Some objects might not obey this setting properly.
    pub rotation: glam::Vec3,
    /// Scale of the object.
    ///
    /// Some objects might not obey this setting properly.
    pub scale: glam::Vec3,
    /// Route used by the object. This is used for moving objects.
    pub route: u16,
    pub presence_flags: u16
}

impl GeoObject {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let raw_object_id = reader.read_u16::<BigEndian>()?;
        // let object_id = GeoObjectId::deserialize(reader)?;
        let _padding = reader.read_u16::<BigEndian>()?;
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let scale = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let route = reader.read_u16::<BigEndian>()?;
        let settings = reader.read_u16_array::<8, BigEndian>()?;
        let presence_flags = reader.read_u16::<BigEndian>()?;

        let object_id = GeoObjectId::from_raw(raw_object_id, settings)?;

        Ok(Self {
            object_id,
            position,
            rotation,
            scale,
            route,
            settings,
            presence_flags
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct GeoObjects {
    pub objects: Vec<GeoObject>
}

impl GeoObjects {
    pub const NAME: &str = "GOBJ";

    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header: KmpSectionHeader
    ) -> SlipstreamResult<Self> {
        // We cap the capacity at 255, as files generally shouldn't have more entries
        // and 255 is still a reasonable upper bound.
        let mut objects =
            Vec::with_capacity(header.entry_count.min(MAX_KMP_SECTION_ENTRY_COUNT) as usize);

        for _ in 0..header.entry_count {
            objects.push(GeoObject::deserialize(reader)?);
        }

        Ok(Self { objects })
    }
}

impl Visitable for GeoObjects {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_geo_objects(VisitorCtx::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }

    fn accept_mut(
        &mut self,
        node: VisitorCtxMetaMut,
        visitor: &mut dyn Visitor
    ) -> ControlFlow<()> {
        visitor
            .as_kmp()
            .map(|v| v.visit_geo_objects_mut(VisitorCtxMut::new(node, self)))
            .unwrap_or(ControlFlow::Break(()))
    }
}
