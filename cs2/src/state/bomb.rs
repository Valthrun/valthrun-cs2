use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_BasePlayerPawn,
    C_PlantedC4,
    C_C4,
};
use nalgebra::Vector3;
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
    /// Total time remaining for a successful bomb defuse
    pub time_remaining: f32,

    /// The defuser's player name
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

/// Information about the C4 (not planted)
pub struct C4 {
    /// Entity ID of the player carrying the bomb
    pub owner_entity_id: Option<u32>,

    /// Position of the bomb.
    pub position: Vector3<f32>,
}

/// Information about the bomb
pub struct Bomb {
    pub planted_c4: Option<PlantedC4>,
    pub c4: Option<C4>,
}

impl State for Bomb {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let globals = states.resolve::<StateGlobals>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        let mut planted_c4_state = PlantedC4 {
            bomb_site: 0,
            defuser: None,
            position: Default::default(),
            state: PlantedC4State::NotPlanted,
        };

        let mut c4_state = C4 {
            owner_entity_id: None,
            position: Default::default(),
        };

        for entity_identity in entities.entities().iter() {
            let class_name = class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?;

            if !class_name
                .map(|name| name == "C_PlantedC4" || name == "C_C4")
                .unwrap_or(false)
            {
                continue;
            }

            let game_scene_node = entity_identity
                .entity_ptr::<dyn C_BaseEntity>()?
                .value_reference(memory.view_arc())
                .context("C_BaseEntity pointer was null")?
                .m_pGameSceneNode()?
                .value_reference(memory.view_arc())
                .context("m_pGameSceneNode pointer was null")?
                .copy()?;

            let position = game_scene_node.m_vecAbsOrigin()?;

            match class_name.as_ref().map(|s| s.as_str()) {
                Some("C_PlantedC4") => {
                    let planted_c4_entity = entity_identity
                        .entity_ptr::<dyn C_PlantedC4>()?
                        .value_copy(memory.view())?
                        .context("planted c4 entity nullptr")?;

                    let bomb_site = planted_c4_entity.m_nBombSite()? as u8;

                    if planted_c4_entity.m_bBombDefused()? {
                        planted_c4_state = PlantedC4 {
                            bomb_site,
                            position: position.into(),
                            defuser: None,
                            state: PlantedC4State::Defused,
                        };

                        break;
                    }

                    let time_blow = planted_c4_entity.m_flC4Blow()?.m_Value()?;
                    if time_blow <= globals.time_2()? {
                        planted_c4_state = PlantedC4 {
                            bomb_site,
                            position: position.into(),
                            defuser: None,
                            state: PlantedC4State::Detonated,
                        };

                        break;
                    }

                    let is_defusing = planted_c4_entity.m_bBeingDefused()?;
                    let defusing = if is_defusing {
                        let time_defuse = planted_c4_entity.m_flDefuseCountDown()?.m_Value()?;

                        let handle_defuser = planted_c4_entity.m_hBombDefuser()?;
                        let defuser = entities
                            .entity_from_handle(&handle_defuser)
                            .and_then(|e| e.value_reference(memory.view_arc()))
                            .context("missing bomb defuser pawn")?;

                        let defuser_controller = defuser.m_hController()?;
                        let defuser_controller = entities
                            .entity_from_handle(&defuser_controller)
                            .and_then(|e| e.value_reference(memory.view_arc()))
                            .context("defuser controller nullptr")?;

                        let defuser_name =
                            CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                                .ok()
                                .map(CStr::to_string_lossy)
                                .unwrap_or_else(|| "Name Error".into())
                                .to_string();

                        Some(BombDefuser {
                            time_remaining: time_defuse - globals.time_2()?,
                            player_name: defuser_name,
                        })
                    } else {
                        None
                    };

                    planted_c4_state = PlantedC4 {
                        bomb_site,
                        defuser: defusing,
                        position: position.into(),
                        state: PlantedC4State::Active {
                            time_detonation: time_blow - globals.time_2()?,
                        },
                    };

                    break;
                }
                Some("C_C4") => {
                    let c4_entity = entity_identity
                        .entity_ptr::<dyn C_C4>()?
                        .value_copy(memory.view())?
                        .context("c4 entity nullptr")?;

                    if c4_entity.m_bBombPlanted()? {
                        continue;
                    }

                    let owner_entity = c4_entity.m_hOwnerEntity()?;
                    let owner_entity_id = owner_entity
                        .is_valid()
                        .then_some(owner_entity.get_entity_index());

                    c4_state = C4 {
                        owner_entity_id,
                        position: position.into(),
                    };

                    break;
                }
                _ => continue,
            };
        }

        let is_not_planted = matches!(planted_c4_state.state, PlantedC4State::NotPlanted);

        Ok(Self {
            planted_c4: Some(planted_c4_state),
            c4: if is_not_planted { Some(c4_state) } else { None },
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
