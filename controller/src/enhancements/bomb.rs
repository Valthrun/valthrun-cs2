use anyhow::Context;
use cs2::{
    state::{
        PlantedC4,
        StatePawnInfo,
    },
    CEntityIdentityEx,
    ClassNameCache,
    LocalCameraControllerTarget,
    PlantedC4State,
    StateCS2Memory,
    StateEntityList,
};
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    CBasePlayerController,
    C_CSObserverPawn,
    C_CSPlayerPawn,
    C_CSPlayerPawnBase,
};
use imgui::ImColor32;
use overlay::UnicodeTextRenderer;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    utils::{
        TextWithShadowUi,
        UnicodeTextWithShadowUi,
    },
};

pub struct BombInfoIndicator {}
impl BombInfoIndicator {
    pub fn new() -> Self {
        Self {}
    }

    fn get_view_pawn_info(
        &self,
        states: &utils_state::StateRegistry,
        memory: &StateCS2Memory,
        entities: &StateEntityList,
        class_name_cache: &ClassNameCache,
        target_entity_id: u32,
    ) -> anyhow::Result<Option<StatePawnInfo>> {
        let entity_identity = entities
            .identity_from_index(target_entity_id)
            .context("missing entity identity")?;

        let entity_class = class_name_cache
            .lookup(&entity_identity.entity_class_info()?)?
            .context("failed to resolve entity class")?;

        let pawn_identity = match entity_class.as_str() {
            "C_CSPlayerPawn" => Some(entity_identity),
            "C_CSObserverPawn" => {
                let observer_pawn = entity_identity
                    .entity_ptr::<dyn C_CSObserverPawn>()?
                    .value_reference(memory.view_arc())
                    .context("observer pawn nullptr")?;

                let controller_handle = observer_pawn.m_hOriginalController()?;

                let player_controller = entities
                    .entity_from_handle(&controller_handle)
                    .context("missing observer controller")?
                    .value_reference(memory.view_arc())
                    .context("nullptr")?
                    .cast::<dyn CBasePlayerController>();

                let pawn_handle = player_controller.m_hPawn()?;

                entities.identity_from_index(pawn_handle.value & 0x7FFF)
            }
            _ => None,
        };

        if let Some(pawn_identity) = pawn_identity {
            // Get the handle and then the entity index from it
            let handle = pawn_identity.handle::<()>()?;
            let entity_index = handle.get_entity_index();

            // Create a C_CSPlayerPawn EntityHandle from the entity index
            let pawn_handle = EntityHandle::<dyn C_CSPlayerPawn>::from_index(entity_index);

            states
                .resolve::<StatePawnInfo>(pawn_handle)
                .map(|pawn_info| Some(pawn_info.clone()))
        } else {
            Ok(None)
        }
    }
}

/// % of the screens height
const PLAYER_AVATAR_TOP_OFFSET: f32 = 0.004;

/// % of the screens height
const PLAYER_AVATAR_SIZE: f32 = 0.05;

/// Maximum distance to be damaged by bomb
pub const MAX_DAMAGE_RANGE: f32 = 1768.0;

impl Enhancement for BombInfoIndicator {
    fn update(&mut self, _ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(
        &self,
        states: &utils_state::StateRegistry,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        let bomb_state = states.resolve::<PlantedC4>(())?;
        let memory = states.resolve::<StateCS2Memory>(())?;
        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        // Get the current target entity ID (whether local player or being spectated)
        let target_entity_id = match view_target.target_entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        if !settings.bomb_timer {
            return Ok(());
        }

        if matches!(bomb_state.state, PlantedC4State::NotPlanted) {
            return Ok(());
        }

        let group = ui.begin_group();

        let line_count = match &bomb_state.state {
            PlantedC4State::Active { .. } => 3,
            PlantedC4State::Defused | PlantedC4State::Detonated => 2,
            PlantedC4State::NotPlanted => unreachable!(),
        };
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        /* align to be on the right side after the players */
        let offset_x = ui.io().display_size[0] * 1730.0 / 2560.0;
        let offset_y = ui.io().display_size[1] * PLAYER_AVATAR_TOP_OFFSET;
        let offset_y = offset_y
            + 0_f32.max((ui.io().display_size[1] * PLAYER_AVATAR_SIZE - text_height) / 2.0);

        // Bomb site text
        ui.set_cursor_pos([offset_x, offset_y]);
        ui.text_with_shadow(&format!(
            "Bomb planted {}",
            if bomb_state.bomb_site == 0 { "A" } else { "B" }
        ));

        let mut offset_y = offset_y + ui.text_line_height_with_spacing();

        match &bomb_state.state {
            PlantedC4State::Active { time_detonation } => {
                // Time text
                ui.set_cursor_pos([offset_x, offset_y]);
                ui.text_with_shadow(&format!("Time: {:.3}", time_detonation));

                offset_y += ui.text_line_height_with_spacing();

                if let Some(defuser) = &bomb_state.defuser {
                    let color = if defuser.time_remaining > *time_detonation {
                        ImColor32::from_rgba(201, 28, 28, 255) // Red
                    } else {
                        ImColor32::from_rgba(28, 201, 66, 255) // Green
                    };

                    let defuse_text = format!(
                        "Defused in {:.3} by {}",
                        defuser.time_remaining, defuser.player_name
                    );

                    ui.set_cursor_pos([offset_x, offset_y]);
                    ui.unicode_text_colored_with_shadow(unicode_text, color, &defuse_text);
                } else {
                    ui.set_cursor_pos([offset_x, offset_y]);
                    ui.text_with_shadow("Not defusing");
                }
            }
            PlantedC4State::Defused => {
                ui.set_cursor_pos([offset_x, offset_y]);
                ui.text_with_shadow("Bomb has been defused");
            }
            PlantedC4State::Detonated => {
                ui.set_cursor_pos([offset_x, offset_y]);
                ui.text_with_shadow("Bomb has been detonated");
            }
            PlantedC4State::NotPlanted => unreachable!(),
        }

        if let PlantedC4State::Active { .. } = &bomb_state.state {
            if let Some(pawn_info) = self.get_view_pawn_info(
                states,
                &memory,
                &entities,
                &class_name_cache,
                target_entity_id,
            )? {
                let distance = (pawn_info.position - bomb_state.position).magnitude();

                // Damage formula from CS:GO, likely similar in CS2.
                let damage = if distance > MAX_DAMAGE_RANGE {
                    0.0
                } else {
                    // Damage = 500 * e^(-distance_in_units / 500)
                    500.0 * (-distance / 500.0).exp()
                };

                let is_safe = (damage as i32) < pawn_info.player_health;

                let (safety_text, safety_color) = if is_safe {
                    ("You're safe!", ImColor32::from_rgba(28, 201, 66, 255))
                } else {
                    ("You're not safe!", ImColor32::from_rgba(201, 28, 28, 255))
                };

                offset_y += ui.text_line_height_with_spacing();

                ui.set_cursor_pos([offset_x, offset_y]);
                ui.unicode_text_colored_with_shadow(unicode_text, safety_color, safety_text);
            }
        }

        group.end();
        Ok(())
    }
}
