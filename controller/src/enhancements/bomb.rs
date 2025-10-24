use anyhow::Context;
use cs2::{
    state::PlantedC4,
    BombCarrierInfo,
    CEntityIdentityEx,
    ClassNameCache,
    PlantedC4State,
    StateCS2Memory,
    StateEntityList,
};
use cs2_schema_generated::cs2::client::{
    C_BaseEntity,
    C_C4,
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
    view::ViewController,
};

pub struct BombInfoIndicator {}
impl BombInfoIndicator {
    pub fn new() -> Self {
        Self {}
    }
}

/// % of the screens height
const PLAYER_AVATAR_TOP_OFFSET: f32 = 0.004;

/// % of the screens height
const PLAYER_AVATAR_SIZE: f32 = 0.05;

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

        group.end();
        Ok(())
    }
}

pub struct BombLabelIndicator {}
impl BombLabelIndicator {
    pub fn new() -> Self {
        Self {}
    }

    /// Render bomb label text above the bomb
    fn render_bomb_text(
        &self,
        ui: &imgui::Ui,
        unicode_text: &UnicodeTextRenderer,
        view: &ViewController,
        position: &nalgebra::Vector3<f32>,
        color: ImColor32,
    ) -> anyhow::Result<()> {
        if let Some(screen_pos) = view.world_to_screen(position, false) {
            let text = "Bomb";
            let text_size = ui.calc_text_size(text);

            // Position text above the bomb
            let text_x = screen_pos.x - text_size[0] / 2.0;
            let text_y = screen_pos.y - 30.0;

            ui.set_cursor_pos([text_x, text_y]);
            ui.unicode_text_colored_with_shadow(unicode_text, color, text);
        }
        Ok(())
    }
}

impl Enhancement for BombLabelIndicator {
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
        let bomb_carrier = states.resolve::<BombCarrierInfo>(())?;
        let view = states.resolve::<ViewController>(())?;

        if !settings.bomb_label {
            return Ok(());
        }

        // Show bomb label for planted bombs
        if !matches!(bomb_state.state, PlantedC4State::NotPlanted) {
            self.render_bomb_text(
                ui,
                unicode_text,
                &view,
                &bomb_state.position,
                ImColor32::from_rgba(255, 0, 0, 255), // Red color for planted bomb
            )?;
        }

        // Show bomb label for dropped C4 entities (when not being carried)
        if bomb_carrier.carrier_entity_id.is_none() {
            let memory = states.resolve::<StateCS2Memory>(())?;
            let entities = states.resolve::<StateEntityList>(())?;
            let class_name_cache = states.resolve::<ClassNameCache>(())?;

            for entity_identity in entities.entities().iter() {
                let class_name = class_name_cache
                    .lookup(&entity_identity.entity_class_info()?)
                    .context("class name")?;

                if !class_name.map(|name| name == "C_C4").unwrap_or(false) {
                    continue;
                }

                let c4_entity = entity_identity
                    .entity_ptr::<dyn C_C4>()?
                    .value_copy(memory.view())?
                    .context("C4 entity nullptr")?;

                // Skip if bomb is planted
                if c4_entity.m_bBombPlanted()? {
                    continue;
                }

                // Get the position of the dropped C4
                let game_scene_node = entity_identity
                    .entity_ptr::<dyn C_BaseEntity>()?
                    .value_reference(memory.view_arc())
                    .context("C_BaseEntity pointer was null")?
                    .m_pGameSceneNode()?
                    .value_reference(memory.view_arc())
                    .context("m_pGameSceneNode pointer was null")?
                    .copy()?;

                let position = game_scene_node.m_vecAbsOrigin()?;

                self.render_bomb_text(
                    ui,
                    unicode_text,
                    &view,
                    &position.into(),
                    ImColor32::from_rgba(255, 165, 0, 255), // Orange color for dropped bomb
                )?;
            }
        }

        Ok(())
    }
}
