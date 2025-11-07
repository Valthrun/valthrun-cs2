use anyhow::Context;
use cs2::{
    schema::ConVar,
    CEntityIdentityEx,
    LocalCameraControllerTarget,
    MouseState,
    StateCS2Memory,
    StateEntityList,
};
use cs2_schema_generated::cs2::client::C_CSPlayerPawn;
use obfstr::obfstr;
use overlay::UnicodeTextRenderer;
use raw_struct::Reference;

use super::Enhancement;
use crate::{
    settings::AppSettings,
    view::KeyToggle,
};

pub struct AntiAimPunsh {
    mouse_sensitivity: Reference<dyn ConVar>,
    toggle: KeyToggle,

    mouse_adjustment_x: i32,
    mouse_adjustment_y: i32,
}

impl AntiAimPunsh {
    pub fn new(mouse_sensitivity: Reference<dyn ConVar>) -> Self {
        Self {
            mouse_sensitivity,
            toggle: KeyToggle::new(),

            mouse_adjustment_x: 0,
            mouse_adjustment_y: 0,
        }
    }
}

impl Enhancement for AntiAimPunsh {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StateCS2Memory>(())?;
        let entities = ctx.states.resolve::<StateEntityList>(())?;
        let settings = ctx.states.resolve::<AppSettings>(())?;

        let toggle_state_changed = self.toggle.update(
            &settings.aim_assist_recoil_mode,
            ctx.input,
            &settings.key_aim_assist_recoil,
        );
        let toggle_enabled = toggle_state_changed && self.toggle.enabled;

        if toggle_state_changed {
            ctx.cs2.add_metrics_record(
                obfstr!("feature-recoil-control-toggle"),
                &format!(
                    "enabled: {}, mode: {:?}",
                    self.toggle.enabled, settings.aim_assist_recoil_mode
                ),
            );
        }

        // Pause when settings menu is open
        if ctx.settings_visible {
            return Ok(());
        }

        if !self.toggle.enabled {
            return Ok(());
        }

        let mouse_sensitivity = {
            let sens = self.mouse_sensitivity.fl_value()?;
            if sens != 0.0 {
                sens
            } else {
                self.mouse_sensitivity.fl_value_default()?
            }
        };

        if mouse_sensitivity == 0.0 {
            log::error!(
                "Unable to read mouse sensitivity, aim assist recoil control doesn't work."
            );
            return Ok(());
        }

        let view_target = ctx.states.resolve::<LocalCameraControllerTarget>(())?;
        let Some(target_entity_id) = view_target.target_entity_id else {
            return Ok(());
        };

        let player_pawn = entities
            .identity_from_index(target_entity_id)
            .context("missing entity identity")?
            .entity_ptr::<dyn C_CSPlayerPawn>()?
            .value_reference(memory.view_arc())
            .context("player pawn nullptr")?;

        let shots_fired = player_pawn.m_iShotsFired()?;
        let min_bullets = settings
            .aim_assist_recoil_min_bullets
            .try_into()
            .unwrap_or(1);

        if shots_fired < min_bullets {
            return Ok(());
        }

        let punch_angle = nalgebra::Vector4::from_row_slice(&player_pawn.m_aimPunchAngle()?) * 2.0;

        let mouse_x = (punch_angle.y
            / settings.aim_assist_recoil_pitch
            / (mouse_sensitivity * 0.022))
            .round() as i32;

        let mouse_y = (punch_angle.x / settings.aim_assist_recoil_yaw / (mouse_sensitivity * 0.022))
            .round() as i32;

        // Reset tracking when toggle became enabled, when shooting stops or min bullets count is reached
        // smoothly reset tracking to current punch angle state to prevent sudden jumps when punch angle hasn't fully decayed
        if toggle_enabled || shots_fired <= 1 || shots_fired == min_bullets {
            self.mouse_adjustment_x = mouse_x;
            self.mouse_adjustment_y = mouse_y;
            return Ok(());
        }

        let delta_x = mouse_x - self.mouse_adjustment_x;
        let delta_y = mouse_y - self.mouse_adjustment_y;

        if delta_x != 0 || delta_y != 0 {
            ctx.cs2.send_mouse_state(&[MouseState {
                last_y: -delta_y,
                last_x: delta_x,
                ..Default::default()
            }])?;

            self.mouse_adjustment_x = mouse_x;
            self.mouse_adjustment_y = mouse_y;
        }

        Ok(())
    }

    fn render(
        &self,
        _states: &utils_state::StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
