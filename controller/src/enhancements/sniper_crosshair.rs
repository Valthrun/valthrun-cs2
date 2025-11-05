use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    LocalCameraControllerTarget,
    StateCS2Memory,
    StateEntityList,
    WeaponId,
};
use cs2_schema_generated::cs2::client::{
    C_CSPlayerPawn,
    C_EconEntity,
};
use overlay::UnicodeTextRenderer;
use utils_state::StateRegistry;

use super::Enhancement;
use crate::settings::AppSettings;

pub struct SniperCrosshair;

impl SniperCrosshair {
    pub fn new() -> Self {
        Self
    }

    fn is_sniper_weapon(&self, weapon_id: u16) -> bool {
        matches!(
            WeaponId::from_id(weapon_id).unwrap_or(WeaponId::Unknown),
            WeaponId::AWP | WeaponId::Ssg08 | WeaponId::Scar20 | WeaponId::G3SG1
        )
    }
}

impl Enhancement for SniperCrosshair {
    fn update(&mut self, _ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(
        &self,
        states: &StateRegistry,
        ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        let settings = states.resolve::<AppSettings>(())?;
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let view = states.resolve::<crate::view::ViewController>(())?;
        let view_target = states.resolve::<LocalCameraControllerTarget>(())?;

        if !settings.sniper_crosshair {
            return Ok(());
        }

        let Some(target_entity_id) = view_target.target_entity_id else {
            return Ok(());
        };

        let player_pawn = entities
            .identity_from_index(target_entity_id)
            .context("missing entity identity")?
            .entity_ptr::<dyn C_CSPlayerPawn>()?
            .value_reference(memory.view_arc())
            .context("player pawn nullptr")?;

        let Some(weapon) = player_pawn
            .m_pClippingWeapon()?
            .value_reference(memory.view_arc())
        else {
            return Ok(());
        };

        let weapon_id = weapon
            .cast::<dyn C_EconEntity>()
            .m_AttributeManager()?
            .m_Item()?
            .m_iItemDefinitionIndex()?;

        if !self.is_sniper_weapon(weapon_id) {
            return Ok(());
        }

        let draw = ui.get_window_draw_list();
        let screen_center = [view.screen_bounds.x / 2.0, view.screen_bounds.y / 2.0];

        // Draw shadow (black outline)
        draw.add_circle(screen_center, 3.5, [0.0, 0.0, 0.0, 0.8])
            .filled(true)
            .build();

        // Draw white dot
        draw.add_circle(screen_center, 2.0, [1.0, 1.0, 1.0, 0.8])
            .filled(true)
            .build();

        Ok(())
    }

    fn render_debug_window(
        &mut self,
        _states: &StateRegistry,
        _ui: &imgui::Ui,
        _unicode_text: &UnicodeTextRenderer,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
