use cs2::state::{
    Bomb,
    BombState,
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

/// Bomb label constants
const BOMB_LABEL_CLOSE_DISTANCE: f32 = 200.0;
const BOMB_LABEL_FAR_DISTANCE: f32 = 1000.0;
const BOMB_LABEL_MIN_ALPHA: u8 = 80;
const BOMB_LABEL_HOVER_ALPHA: u8 = 50;
const BOMB_LABEL_MOUSE_PADDING: f32 = 5.0;
const BOMB_LABEL_Y_OFFSET: f32 = 30.0;

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
        let bomb = states.resolve::<Bomb>(())?;

        if !settings.bomb_timer {
            return Ok(());
        }

        if !matches!(bomb.state, BombState::Planted) {
            return Ok(());
        }

        let Some(planted_c4) = bomb.planted_c4.as_ref() else {
            return Ok(());
        };

        let group = ui.begin_group();

        let line_count = match &planted_c4.state {
            PlantedC4State::Active { .. } => 3,
            PlantedC4State::Defused | PlantedC4State::Detonated => 2,
        };
        let text_height = ui.text_line_height_with_spacing() * line_count as f32;

        /* Align to be on the right side after the players */
        let offset_x = ui.io().display_size[0] * 1730.0 / 2560.0;
        let offset_y = ui.io().display_size[1] * PLAYER_AVATAR_TOP_OFFSET;
        let offset_y = offset_y
            + 0_f32.max((ui.io().display_size[1] * PLAYER_AVATAR_SIZE - text_height) / 2.0);

        // Bomb site text
        ui.set_cursor_pos([offset_x, offset_y]);
        ui.text_with_shadow(&format!(
            "Bomb planted {}",
            if planted_c4.bomb_site == 0 { "A" } else { "B" }
        ));

        let mut offset_y = offset_y + ui.text_line_height_with_spacing();

        match &planted_c4.state {
            PlantedC4State::Active { time_detonation } => {
                // Time text
                ui.set_cursor_pos([offset_x, offset_y]);
                ui.text_with_shadow(&format!("Time: {:.3}", time_detonation));

                offset_y += ui.text_line_height_with_spacing();

                if let Some(defuser) = &planted_c4.defuser {
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

            let mut alpha = if distance < BOMB_LABEL_CLOSE_DISTANCE {
                255
            } else if distance < BOMB_LABEL_FAR_DISTANCE {
                let t = (distance - BOMB_LABEL_CLOSE_DISTANCE)
                    / (BOMB_LABEL_FAR_DISTANCE - BOMB_LABEL_CLOSE_DISTANCE);
                (255.0 - t * 175.0) as u8
            } else {
                BOMB_LABEL_MIN_ALPHA
            };

            let text = "Bomb";
            let text_size = ui.calc_text_size(text);

            // Position text above the bomb
            let text_x = screen_pos.x - text_size[0] / 2.0;
            let text_y = screen_pos.y - BOMB_LABEL_Y_OFFSET;

            // Check whether mouse is near the text
            let mouse_pos = ui.io().mouse_pos;
            let text_bounds_padding = BOMB_LABEL_MOUSE_PADDING;

            let is_mouse_near = mouse_pos[0] >= text_x - text_bounds_padding
                && mouse_pos[0] <= text_x + text_size[0] + text_bounds_padding
                && mouse_pos[1] >= text_y - text_bounds_padding
                && mouse_pos[1] <= text_y + ui.text_line_height() + text_bounds_padding;

            // When mouse is near (hover) and text is in fade range, reduce opacity to minimum
            if is_mouse_near
                && distance >= BOMB_LABEL_CLOSE_DISTANCE
                && distance < BOMB_LABEL_FAR_DISTANCE
            {
                alpha = BOMB_LABEL_HOVER_ALPHA;
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
        let bomb = states.resolve::<Bomb>(())?;
        let view = states.resolve::<ViewController>(())?;

        if !settings.bomb_label {
            return Ok(());
        }

        if matches!(bomb.state, BombState::Unknown) {
            return Ok(());
        }

        // Show bomb label for planted bomb
        if matches!(bomb.state, BombState::Planted) {
            if let Some(planted_c4) = &bomb.planted_c4 {
                self.render_bomb_text(
                    ui,
                    unicode_text,
                    &view,
                    &planted_c4.position,
                    ImColor32::from_rgba(255, 0, 0, 255), // Red color for planted bomb
                )?;
            }
        }

        // Show bomb label for dropped bomb
        if matches!(bomb.state, BombState::Dropped) {
            if let Some(c4) = &bomb.c4 {
                self.render_bomb_text(
                    ui,
                    unicode_text,
                    &view,
                    &c4.position,
                    ImColor32::from_rgba(255, 165, 0, 255), // Orange color for dropped bomb
                )?;
            }
        }

        Ok(())
    }
}
