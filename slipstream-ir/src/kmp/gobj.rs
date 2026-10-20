use std::ops::ControlFlow;

use byteorder::{BigEndian, ReadBytesExt};
use slipstream_derive::{Inspect, inspect_bitfield, try_from};
use slipstream_shared::{RefCursor, SlipstreamResult};

use crate::encoding::ReadArrayExt;
use crate::kmp::{KmpSectionHeader, MAX_KMP_SECTION_ENTRY_COUNT};
use crate::visitor::{
    Visitable, Visitor, VisitorCtx, VisitorCtxMeta, VisitorCtxMetaMut, VisitorCtxMut
};

#[derive(Debug, Copy, Clone, PartialEq, Inspect)]
pub enum GeoObjectId {
    /// Invisible wall above barriers.
    ///
    /// ## Note:
    /// The Y and Z scale axes are swapped for this object!
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/airblock)
    AirBlock = 0x001,
    /// The sea on GCN Peach Beach.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/Psea)
    PeachBeachSea = 0x002,
    /// Generates a lens flare object when looking at the sun.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/lensFX)
    LensFx = 0x003,
    /// The rising and dropping sea on Delfino Pier. The `pocha` object is required if the player
    /// can interact with it.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/venice_nami)
    VeniceNami {
        /// Rising speed in odd stages.
        odd_rising_speed: u16,
        stage5_target_height: u16,
        stage1_target_height: u16,
        stage2_frame_count: u16,
        stage3_target_height: u16,
        stage4_frame_count: u16,
        /// Rising speed in even stages.
        even_rising_speed: u16
    },
    /// Generates a river sound effect.
    ///
    /// Only works on slots 3.2, 4.2, 5.2 and 7.3.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_river)
    SoundRiver,
    /// Generates a waterfall sound effect.
    ///
    /// Only works on slots 3.2, 4.2 and 7.3.
    SoundWaterFall,
    /// Generates a water splash effect when a player touches KCL flag `0x10` with variants 1
    /// (water) or 3 (icy water that also needs the `ice` object).
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/pocha)
    Pocha,
    /// Generates a lake sound effect.
    ///
    /// Only works on slots 2.2, 3.1, 5.1 and 8.2.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_lake)
    SoundLake,
    /// Generates a louder waterfall sound effect.
    ///
    /// Only works on slots 3.2, 4.2 and 7.3.
    ///
    /// See the [wiki](https://szs.wiimm.de/cgi/mkw/track-ref/object/sound_big_fall)
    SoundBigFall,
    SoundSea,
    SoundFountain,
    SoundVolcano,
    SoundAudience,
    SoundBigRiver,
    SoundSandFall,
    SoundLift,
    PochaYogan,
    Entry,
    PochaMori,
    ElineControl,
    SoundMii,
    BegomanManager,
    Ice,
    Startline2d,
    Itembox,
    DummyPole,
    Flag,
    FlagBlend,
    GndSphere,
    GndTrapezoid,
    GndWave1,
    GndWave2,
    GndWave3,
    GndWave4,
    Sun,
    Woodbox,
    KmoonZ,
    SunDs,
    Coin,
    Ironbox,
    ItemDirect,
    SItemBox,
    PileCoin,
    FItemBox,
    MashaBalloonGc,
    WlWallGc,
    CarA1,
    Basabasa,
    HeyHoShipGba,
    KoopaBall,
    KartTruck,
    CarBody,
    SkyShip,
    WWoodBox,
    WItemBox,
    WItemBoxLine,
    VolcanoBall1,
    PenguinS,
    PenguinM,
    PenguinL,
    CastleBalloon1,
    DossunC,
    DossunCSoko,
    Boble,
    KBombCar,
    KBombCarDummy,
    CarBodyDummy,
    KartTruckDummy,
    Hanachan,
    Seagull,
    Moray,
    Crab,
    BasabasaDummy,
    CarA2,
    CarA3,
    Hwanwan,
    HeyHoBallGba,
    Twanwan,
    CruiserR,
    Bird,
    SinItemBox,
    TwanwanUe,
    BossHanachan,
    KDossunC,
    BossHanachanHead,
    KBombCar1,
    Dummy,
    DokanSfc,
    CastleTree1,
    CastleTree1C,
    CastleTree2,
    CastleFlower1,
    MarioTreeGc,
    MarioTreeGcC,
    DonkyTree1Gc,
    DonkyTree2Gc,
    PeachTreeGc,
    NpcMiiA,
    NpcMiiB,
    NpcMiiC,
    ObakeBlockSfcC,
    WlArrowGc,
    WlScreenGc,
    WlDokanGc,
    MarioGo64C,
    PeachHunsuiGc,
    KinokoT1,
    KinokoT2,
    Pylon01,
    PalmTree,
    Parasol,
    Cruiser,
    KStickLift00,
    HeyHo2,
    HeyHoTreeGbaC,
    MFaceBill,
    TruckChimSmk,
    MiiObj01,
    MiiObj02,
    MiiObj03,
    GardenTreeDs,
    GardenTreeDsC,
    FlagA1,
    FlagA2,
    FlagB1,
    FlagB2,
    FlagA3,
    DkTreeA64,
    DkTreeA64C,
    DkTreeB64,
    DkTreeB64C,
    TownTreeDSc,
    Piston,
    OilSfc,
    DkMarutaGcC,
    DkRopeGcC,
    MiiBalloon,
    Windmill,
    Dossun,
    TownTreeDs,
    KStickEtc,
    MonteA,
    MiiStatueM1,
    ShMiiObj01,
    ShMiiObj02,
    ShMiiObj03,
    Hanabi,
    MiiPoster,
    DkMiiObj00,
    LightHouse,
    RParasol,
    ObakeBlock2SfcC,
    ObakeBlock3SfcC,
    KoopaFigure,
    Pukupuku,
    VStand1,
    VStand2,
    LeafEffect,
    Karehayama,
    EarthRing,
    SpaceSun,
    BlackHole,
    StarRing,
    MObjKanban,
    MiiStatueL1,
    MiiStatueD1,
    MiiSphinxY1,
    MiiSphinxY2,
    FlagA5,
    CarB,
    FlagA4,
    Steam,
    Alarm,
    GroupMonteA,
    MiiStatueL2,
    MiiStatueD2,
    MiiStatueP1,
    SentakuDs,
    FksScreenWii,
    KoopaFigure64,
    BTeresa,
    MiiStatueDk1,
    MiiKanban,
    BgTeresaSfc,
    Kuribo,
    Choropu,
    Cow,
    PakkunF,
    WlFirebarGc,
    Wanwan,
    Poihana,
    DkRockGc,
    Sanbo,
    Choropu2,
    TruckWagon,
    HeyHo,
    Press,
    PressSoko,
    Pile,
    ChoropuGround,
    WlFireRingGc,
    PakkunDokan,
    BegomanSpike,
    FireSnake,
    KoopaFireBar,
    EPropeller,
    DcPillarC,
    FireSnakeV,
    HoneBall,
    PunchiPakkun,
    SanboBig,
    SanboBig2,
    KinokoUd,
    KinokoBend,
    VolcanoRock1,
    BulldozerLeft,
    BulldozerRight,
    KinokoNm,
    Crane,
    VolcanoPiece,
    FlamePole,
    TwistedWay,
    TownBridgeDsc,
    DkShip64,
    KinokoKuki,
    DkTuribashiGcC,
    FlamePollEff,
    Aurora,
    VeniceSaku,
    CasinoRoulette,
    BossField01Obj1,
    DcPillar,
    DcSandcone,
    VeniceHasi,
    VeniceGondola,
    Quicksand,
    BBlock,
    Ami,
    MObjJump,
    StarGate,
    RmRing1,
    FlamePoleV,
    MObjSJump,
    InsekiA,
    InsekiB,
    FlamePoleVBig,
    MDush,
    HpPipe,
    DemoCol,
    MObjSJump2,
    MObjJump2,
    DonkyCannonGc,
    BeltEasy,
    BeltCrossing,
    BeltCurveA,
    BeltCurveB,
    Escalator,
    DonkyCannonWii,
    EscalatorGroup,
    TreeCannon,
    GroupEnemyB,
    GroupEnemyC,
    Taimatsu,
    TruckChimSmkW,
    MStand,
    DkMonitor,
    GroupEnemyA,
    FlagB3,
    Spot,
    GroupEnemyD,
    FlagB4,
    GroupEnemyE,
    GroupMonteL,
    GroupEnemyF,
    FallBsA,
    FallBsB,
    VolSmk,
    RidgeMii00,
    FlashL,
    FlashB,
    FlashW,
    FlashM,
    FlashS,
    MiiSignNoko,
    UtsuboDokan,
    Spot64,
    DemoEf,
    FallMh,
    FallY,
    DemoJugemu,
    GroupEnemyADemo,
    VolFall,
    MiiStatueM2,
    RhMiiKanban,
    MiiStatueL3,
    MiiSignWario,
    MiiStatueBl1,
    MiiStatueBd1,
    Kamifubuki,
    Crescent64,
    // TODO: This might be misspelled?
    MiiSighKino,
    MiiObjD01,
    MiiObjD02,
    MiiObjD03,
    MareA,
    MareB,
    EnvKareha,
    EnvFire,
    EnvSnow,
    MObjStart,
    EnvKarehaUp,
    MObjKanbanY,
    DkFalls
}

impl GeoObjectId {
    pub fn try_from(id: u16, settings: [u16; 8]) -> SlipstreamResult<Self> {
        use GeoObjectId::*;

        Ok(match id {
            0x00 => AirBlock,
            0x01 => PeachBeachSea,
            0x02 => LensFx,
            0x03 => VeniceNami {
                odd_rising_speed: settings[0],
                stage5_target_height: settings[1],
                stage1_target_height: settings[2],
                stage2_frame_count: settings[3],
                stage3_target_height: settings[4],
                stage4_frame_count: settings[5],
                even_rising_speed: settings[6]
            },
            0x04
        })
    }
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct GeoObject {
    pub object_id: GeoObjectId,
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
    /// Route used by the object
    pub route: u16,
    pub settings: [u16; 8],
    pub presence_flags: u16
}

impl GeoObject {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let object_id = GeoObjectId::deserialize(reader)?;
        let _padding = reader.read_u16::<BigEndian>()?;
        let position = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let rotation = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let scale = glam::Vec3::from_array(reader.read_f32_array::<3, BigEndian>()?);
        let route = reader.read_u16::<BigEndian>()?;
        let settings = reader.read_u16_array::<8, BigEndian>()?;
        let presence_flags = reader.read_u16::<BigEndian>()?;

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
