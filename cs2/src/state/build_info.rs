use cs2_schema_cutl::CStringUtil;
use raw_struct::{
    Copy,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::EngineBuildInfo,
    CS2Offset,
    StateCS2Memory,
    StateResolvedOffset,
};

#[derive(Debug)]
pub struct StateBuildInfo {
    pub revision: String,
    pub build_datetime: String,
}

impl State for StateBuildInfo {
    type Parameter = ();

    fn create(states: &StateRegistry, _params: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;

        let offset = match states.resolve::<StateResolvedOffset>(CS2Offset::BuildInfo) {
            Ok(offset) => offset,
            Err(err) => {
                log::warn!(
                    "Failed to resolve CS2 build info offset: {err}. Falling back to unknown build info."
                );
                return Ok(Self {
                    revision: "unknown".to_string(),
                    build_datetime: "unknown".to_string(),
                });
            }
        };

        let engine_build_info = match Copy::<dyn EngineBuildInfo>::read_object(memory.view(), offset.address) {
            Ok(engine_build_info) => engine_build_info,
            Err(err) => {
                log::warn!(
                    "Failed to read CS2 build info from address 0x{:X}: {err}. Falling back to unknown build info.",
                    offset.address
                );
                return Ok(Self {
                    revision: "unknown".to_string(),
                    build_datetime: "unknown".to_string(),
                });
            }
        };

        Ok(Self {
            revision: engine_build_info
                .revision()?
                .read_string(memory.view())?
                .unwrap_or_default(),
            build_datetime: format!(
                "{} {}",
                engine_build_info
                    .build_date()?
                    .read_string(memory.view())?
                    .unwrap_or_default(),
                engine_build_info
                    .build_time()?
                    .read_string(memory.view())?
                    .unwrap_or_default()
            ),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}
