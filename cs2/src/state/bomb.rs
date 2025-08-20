use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_BasePlayerPawn,
    C_CSPlayerPawn,
    C_EconEntity,
    C_PlantedC4,
    C_C4,
};
use nalgebra::Vector3;
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use super::StateGlobals;
use crate::{
    CEntityIdentityEx,
    ClassNameCache,
    StateCS2Memory,
    StateEntityList,
};

#[derive(Debug)]
pub struct BombDefuser {
    /// Totoal time remaining for a successfull bomb defuse
    pub time_remaining: f32,

    /// The defusers player name
    pub player_name: String,
}

#[derive(Debug)]
pub enum PlantedC4State {
    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        time_detonation: f32,
    },

    /// Bomb has detonated
    Detonated,

    /// Bomb has been defused
    Defused,

    /// Bomb has not been planted
    NotPlanted,
}

/// Information about the currently active planted C4
pub struct PlantedC4 {
    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    pub bomb_site: u8,

    /// Current state of the planted C4
    pub state: PlantedC4State,

    /// Position of the planted bomb.
    pub position: Vector3<f32>,

    /// Current bomb defuser
    pub defuser: Option<BombDefuser>,
}

/// Information about the current bomb carrier
#[derive(Debug, Clone)]
pub struct BombCarrierInfo {
    /// Entity ID of the player carrying the bomb
    pub carrier_entity_id: Option<u32>,

    /// Name of the player carrying the bomb
    pub carrier_name: Option<String>,

    /// Team ID of the bomb carrier (should be 2 for terrorists)
    pub carrier_team_id: Option<u8>,
}

impl State for PlantedC4 {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let globals = states.resolve::<StateGlobals>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        for entity_identity in entities.entities().iter() {
            let class_name = class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?;

            if !class_name
                .map(|name| name == "C_PlantedC4")
                .unwrap_or(false)
            {
                /* Entity isn't the planted bomb. */
                continue;
            }

            let bomb = entity_identity
                .entity_ptr::<dyn C_PlantedC4>()?
                .value_copy(memory.view())?
                .context("bomb entity nullptr")?;

            let game_scene_node = entity_identity
                .entity_ptr::<dyn C_BaseEntity>()?
                .value_reference(memory.view_arc())
                .context("C_BaseEntity pointer was null")?
                .m_pGameSceneNode()?
                .value_reference(memory.view_arc())
                .context("m_pGameSceneNode pointer was null")?
                .copy()?;

            let position = game_scene_node.m_vecAbsOrigin()?;

            if !bomb.m_bC4Activated()? {
                /* This bomb hasn't been activated (yet) */
                continue;
            }

            let bomb_site = bomb.m_nBombSite()? as u8;
            if bomb.m_bBombDefused()? {
                return Ok(Self {
                    bomb_site,
                    position: position.into(),
                    defuser: None,
                    state: PlantedC4State::Defused,
                });
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;

            if time_blow <= globals.time_2()? {
                return Ok(Self {
                    bomb_site,
                    position: position.into(),
                    defuser: None,
                    state: PlantedC4State::Detonated,
                });
            }

            let is_defusing = bomb.m_bBeingDefused()?;
            let defusing = if is_defusing {
                let time_defuse = bomb.m_flDefuseCountDown()?.m_Value()?;

                let handle_defuser = bomb.m_hBombDefuser()?;
                let defuser = entities
                    .entity_from_handle(&handle_defuser)
                    .context("missing bomb defuser pawn")?
                    .value_reference(memory.view_arc())
                    .context("defuser pawn nullptr")?;

                let defuser_controller = defuser.m_hController()?;
                let defuser_controller = entities
                    .entity_from_handle(&defuser_controller)
                    .with_context(|| obfstr!("missing bomb defuser controller").to_string())?
                    .value_reference(memory.view_arc())
                    .context("defuser constroller nullptr")?;

                let defuser_name =
                    CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                        .ok()
                        .map(CStr::to_string_lossy)
                        .unwrap_or("Name Error".into())
                        .to_string();

                Some(BombDefuser {
                    time_remaining: time_defuse - globals.time_2()?,
                    player_name: defuser_name,
                })
            } else {
                None
            };

            return Ok(Self {
                bomb_site,
                defuser: defusing,
                position: position.into(),
                state: PlantedC4State::Active {
                    time_detonation: time_blow - globals.time_2()?,
                },
            });
        }

        return Ok(Self {
            bomb_site: 0,
            defuser: None,
            position: Default::default(),
            state: PlantedC4State::NotPlanted,
        });
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl State for BombCarrierInfo {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        // Find the C4 entity and its owner
        for entity_identity in entities.entities().iter() {
            let class_info = entity_identity.entity_class_info()?;
            let class_name = class_name_cache.lookup(&class_info)?;

            if !class_name.map(|name| name == "C_C4").unwrap_or(false) {
                continue;
            }

            let c4_entity = entity_identity
                .entity_ptr::<dyn C_EconEntity>()?
                .value_reference(memory.view_arc())
                .context("C4 entity nullptr")?
                .cast::<dyn C_C4>();

            let owner_handle = c4_entity.m_hOwnerEntity()?;
            if !owner_handle.is_valid() {
                continue;
            }

            let owner_entity = entities.entity_from_handle(&owner_handle);
            if let Some(owner_identity) = owner_entity {
                let owner_pawn = owner_identity
                    .value_reference(memory.view_arc())
                    .context("owner pawn nullptr")?
                    .cast::<dyn C_CSPlayerPawn>();

                let controller_handle = owner_pawn.m_hController()?;
                let team_id = owner_pawn.m_iTeamNum()?;

                let carrier_name = if controller_handle.is_valid() {
                    entities
                        .entity_from_handle(&controller_handle)
                        .and_then(|controller| controller.value_reference(memory.view_arc()))
                        .and_then(|controller_ref| {
                            controller_ref
                                .m_iszPlayerName()
                                .ok()
                                .and_then(|name_bytes| {
                                    CStr::from_bytes_until_nul(&name_bytes)
                                        .ok()
                                        .map(|name| name.to_string_lossy().to_string())
                                })
                        })
                } else {
                    None
                };

                return Ok(Self {
                    carrier_entity_id: Some(owner_handle.get_entity_index()),
                    carrier_name,
                    carrier_team_id: Some(team_id),
                });
            }
        }

        // No bomb carrier found
        Ok(Self {
            carrier_entity_id: None,
            carrier_name: None,
            carrier_team_id: None,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
