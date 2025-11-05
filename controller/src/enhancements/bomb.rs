use cs2::state::{
    Bomb,
    PlantedC4State,
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
        let bomb_state = states.resolve::<Bomb>(())?;

        if !settings.bomb_timer {
            return Ok(());
        }

        let Some(bomb_state) = &bomb_state.planted_c4 else {
            return Ok(());
        };

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
        base_color: ImColor32,
    ) -> anyhow::Result<()> {
        if let Some(screen_pos) = view.world_to_screen(position, false) {
            // Calculate distance from camera to bomb
            let camera_pos = match view.get_camera_world_position() {
                Some(pos) => pos,
                None => return Ok(()),
            };
            let distance = (position - &camera_pos).norm();

            // Calculate opacity based on distance with smooth fade
            // Close range (0-200 units): full opacity (255)
            // Smooth fade range (200-1000 units): gradually fade from 255 to 80
            // Far range (1000+ units): minimum opacity (80)
            let mut alpha = if distance < 200.0 {
                255
            } else if distance < 1000.0 {
                // Smooth linear interpolation from 255 to 80
                let t = (distance - 200.0) / (1000.0 - 200.0); // 0.0 at 200, 1.0 at 1000
                (255.0 - t * 175.0) as u8
            } else {
                80
            };

            let text = "Bomb";
            let text_size = ui.calc_text_size(text);

            // Position text above the bomb
            let text_x = screen_pos.x - text_size[0] / 2.0;
            let text_y = screen_pos.y - 30.0;

            // Check whether mouse is near the text (within 5 pixels)
            let mouse_pos = ui.io().mouse_pos;
            let text_bounds_padding = 5.0;

            let is_mouse_near = mouse_pos[0] >= text_x - text_bounds_padding
                && mouse_pos[0] <= text_x + text_size[0] + text_bounds_padding
                && mouse_pos[1] >= text_y - text_bounds_padding
                && mouse_pos[1] <= text_y + ui.text_line_height() + text_bounds_padding;

            // When mouse is near (hover) and text is in fade range, reduce opacity to minimum
            if is_mouse_near && distance >= 200.0 && distance < 1000.0 {
                alpha = 50;
            }

            // Apply calculated alpha to the base color
            let color = ImColor32::from_rgba(base_color.r, base_color.g, base_color.b, alpha);

            ui.set_cursor_pos([text_x, text_y]);
            ui.unicode_text_colored_with_shadow_alpha(unicode_text, color, text);
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
        let bomb_state = states.resolve::<Bomb>(())?;
        let view = states.resolve::<ViewController>(())?;

        if !settings.bomb_label {
            return Ok(());
        }

        // Show bomb label for planted bomb
        if let Some(planted_c4_state) = &bomb_state.planted_c4 {
            if matches!(planted_c4_state.state, PlantedC4State::Active { .. })
                && planted_c4_state.position != nalgebra::Vector3::default()
            {
                self.render_bomb_text(
                    ui,
                    unicode_text,
                    &view,
                    &planted_c4_state.position,
                    ImColor32::from_rgba(255, 0, 0, 255), // Red color for planted bomb
                )?;
            }
        }

        // Show bomb label for dropped bomb
        if let Some(c4_state) = &bomb_state.c4 {
            if c4_state.owner_entity_id.is_none()
                && c4_state.position != nalgebra::Vector3::default()
            {
                self.render_bomb_text(
                    ui,
                    unicode_text,
                    &view,
                    &c4_state.position,
                    ImColor32::from_rgba(255, 165, 0, 255), // Orange color for dropped bomb
                )?;
            }
        }

        Ok(())
    }
}
