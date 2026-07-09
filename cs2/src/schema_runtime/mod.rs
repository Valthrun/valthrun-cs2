use std::{
    fs,
    path::PathBuf,
};

use anyhow::Context;
use sha1::Digest;
use utils_state::StateRegistry;
use vtd_libum::InterfaceError;

use crate::{
    schema_runtime::{
        dump::RuntimeSchemaState,
        provider::RuntimeSchemaProvider,
    },
    CS2Offset,
    StateBuildInfo,
    StatePredefinedOffset,
};

pub mod dump;
mod generator;
mod provider;

fn setup_state(registry: &StateRegistry, state: &RuntimeSchemaState) -> anyhow::Result<()> {
    cs2_schema_provider::setup_provider(Box::new(RuntimeSchemaProvider::new(&state.offsets)));

    for offset in CS2Offset::available_offsets() {
        if let Some(value) = state.resolved_offsets.get(offset.cache_name()).cloned() {
            let predefined_offset = StatePredefinedOffset::new(registry, *offset, value)
                .with_context(|| format!("resolving predefined offset {}", offset.cache_name()))?;

            log::debug!(
                "Registering predefined offset {} (offset: {:X}, current address: {:X})",
                offset.cache_name(),
                value,
                predefined_offset.resolved
            );
            let _ = registry.set(predefined_offset, *offset);
        }
    }

    return Ok(());
}

pub struct SetupOptions {
    pub file: Option<PathBuf>,
    pub fscache: Option<PathBuf>,
}

pub fn setup(registry: &StateRegistry, options: &SetupOptions) -> anyhow::Result<()> {
    if let Some(file) = &options.file {
        log::info!(
            "Loading CS2 schema (offsets) offsets from {}",
            file.display()
        );

        self::setup_state(registry, &RuntimeSchemaState::load_from_file(&file)?)?;
        return Ok(());
    }

    let cache_file = if let Some(fscache) = &options.fscache {
        let cs2_build_info = registry
            .resolve::<StateBuildInfo>(())
            .context("build info")?;

        let version_hash = {
            let mut hasher = sha1::Sha1::new();
            hasher.update(&cs2_build_info.build_datetime);
            hasher.update(&cs2_build_info.revision);
            hex::encode(&hasher.finalize())
        };
        let cache_file = fscache.join(format!("cache_{version_hash}.json",));
        log::debug!("Looking for schema cache file {}", cache_file.display());
        if cache_file.exists() {
            log::debug!(" -> found, try loading");

            match RuntimeSchemaState::load_from_file(&cache_file)
                .and_then(|state| self::setup_state(registry, &state))
            {
                Ok(_) => {
                    log::debug!(" -> success");
                    log::info!(
                        "Loaded CS2 schema cache from cache {}",
                        cache_file.display()
                    );
                    return Ok(());
                }
                Err(error) => {
                    log::debug!(" -> failed: {error}");
                }
            }
        } else {
            log::debug!(" -> not found");
        }

        Some(cache_file)
    } else {
        None
    };

    log::info!("Loading CS2 schema (offsets) from ingame CS2 schema system",);
    let runtime_state = match RuntimeSchemaState::from_game(registry) {
        Ok(schema) => schema,
        Err(error) => {
            if let Some(inner) = error.downcast_ref::<InterfaceError>() {
                if matches!(inner, InterfaceError::MemoryAccessPagedOut) {
                    log::error!(
                        "Could not load schema from game memory as memory has been paged out."
                    );
                    log::error!("Please restart the game and try again. Once succeeded the schema will be cached.");
                    anyhow::bail!("schema memory paged out");
                }
            }

            anyhow::bail!(error);
        }
    };

    if let Some(cache_file) = cache_file {
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match runtime_state.write_to_file(&cache_file) {
            Ok(_) => log::info!("Wrote schema cache to {}", cache_file.display()),
            Err(error) => log::warn!(
                "Failed to write schema cache to {}: {error}",
                cache_file.display()
            ),
        }
    }

    self::setup_state(registry, &runtime_state)?;
    log::info!("CS2 schema (offsets) loaded.");
    Ok(())
}
