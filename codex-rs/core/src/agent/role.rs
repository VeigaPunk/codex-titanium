//! Applies agent-role configuration layers on top of an existing session config.
//!
//! Roles are selected at spawn time and are loaded with the same config machinery as
//! `config.toml`. This module resolves built-in and user-defined role files, inserts the role as a
//! high-precedence layer, and preserves the caller's current model, reasoning effort, provider,
//! and service tier unless the role layer sets them. It does not decide when to spawn a sub-agent
//! or which role to use; the multi-agent tool handler owns that orchestration.

use crate::config::AgentRoleConfig;
use crate::config::Config;
use crate::config::ConfigOverrides;
use crate::config::agent_roles::parse_agent_role_file_contents;
use crate::config::deserialize_config_toml_with_base;
use anyhow::anyhow;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::ConfigLayerStackOrdering;
use codex_config::config_toml::ConfigToml;
use codex_config::loader::resolve_relative_paths_in_config_toml;
use codex_exec_server::LOCAL_FS;
use std::collections::BTreeMap;
use toml::Value as TomlValue;

/// The role name used when a caller omits `agent_type`.
pub const DEFAULT_ROLE_NAME: &str = "default";
const AGENT_TYPE_UNAVAILABLE_ERROR: &str = "agent type is currently not available";

/// Applies a named role layer to `config` while preserving caller-owned provider settings.
///
/// The role layer is inserted at session-flag precedence so it can override persisted config, but
/// the caller's current `model_provider` and `service_tier` remain sticky runtime choices unless
/// the role explicitly sets the corresponding top-level config key. Rebuilding the config without
/// those overrides would make a spawned agent silently fall back to default settings.
pub(crate) async fn apply_role_to_config(
    config: &mut Config,
    role_name: Option<&str>,
) -> Result<(), String> {
    let Some(role_name) = role_name else {
        return Ok(());
    };

    let role = resolve_role_config(config, role_name)
        .cloned()
        .ok_or_else(|| format!("unknown agent_type '{role_name}'"))?;

    apply_role_to_config_inner(config, role_name, &role)
        .await
        .map_err(|err| {
            tracing::warn!("failed to apply role to config: {err}");
            AGENT_TYPE_UNAVAILABLE_ERROR.to_string()
        })
}

async fn apply_role_to_config_inner(
    config: &mut Config,
    role_name: &str,
    role: &AgentRoleConfig,
) -> anyhow::Result<()> {
    let Some(config_file) = role.config_file.as_ref() else {
        return Ok(());
    };
    let role_layer_toml = load_role_layer_toml(config_file, role_name).await?;
    if role_layer_toml
        .as_table()
        .is_some_and(toml::map::Map::is_empty)
    {
        return Ok(());
    }
    let preserve_current_provider = role_layer_toml.get("model_provider").is_none();
    let preserve_current_service_tier = role_layer_toml.get("service_tier").is_none();

    *config = reload::build_next_config(
        config,
        role_layer_toml,
        preserve_current_provider,
        preserve_current_service_tier,
    )
    .await?;
    Ok(())
}

async fn load_role_layer_toml(
    config_file: &std::path::Path,
    role_name: &str,
) -> anyhow::Result<TomlValue> {
    let role_config_contents = tokio::fs::read_to_string(config_file).await?;
    let role_config_base = config_file
        .parent()
        .ok_or(anyhow!("No corresponding config content"))?;
    let role_config_toml = parse_agent_role_file_contents(
        &role_config_contents,
        config_file,
        role_config_base,
        Some(role_name),
    )?
    .config;

    deserialize_config_toml_with_base(role_config_toml.clone(), role_config_base)?;
    Ok(resolve_relative_paths_in_config_toml(
        role_config_toml,
        role_config_base,
    )?)
}

pub(crate) fn resolve_role_config<'a>(
    config: &'a Config,
    role_name: &str,
) -> Option<&'a AgentRoleConfig> {
    config.agent_roles.get(role_name)
}

mod reload {
    use super::*;

    pub(super) async fn build_next_config(
        config: &Config,
        role_layer_toml: TomlValue,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> anyhow::Result<Config> {
        let preserve_current_model = role_layer_toml.get("model").is_none();
        let preserve_current_reasoning_effort =
            role_layer_toml.get("model_reasoning_effort").is_none();
        let config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        let merged_config = deserialize_effective_config(config, &config_layer_stack)?;

        let mut next_config = Config::load_config_with_layer_stack(
            LOCAL_FS.as_ref(),
            merged_config,
            reload_overrides(
                config,
                preserve_current_model,
                preserve_current_provider,
                preserve_current_service_tier,
            ),
            config.codex_home.clone(),
            config_layer_stack,
        )
        .await?;
        if preserve_current_reasoning_effort {
            next_config
                .model_reasoning_effort
                .clone_from(&config.model_reasoning_effort);
        }
        Ok(next_config)
    }

    fn build_config_layer_stack(
        config: &Config,
        role_layer_toml: &TomlValue,
    ) -> anyhow::Result<ConfigLayerStack> {
        let mut layers = existing_layers(config);
        insert_layer(&mut layers, role_layer(role_layer_toml.clone()));
        Ok(ConfigLayerStack::new(
            layers,
            config.config_layer_stack.requirements().clone(),
            config.config_layer_stack.requirements_toml().clone(),
        )?)
    }

    fn deserialize_effective_config(
        config: &Config,
        config_layer_stack: &ConfigLayerStack,
    ) -> anyhow::Result<ConfigToml> {
        Ok(deserialize_config_toml_with_base(
            config_layer_stack.effective_config(),
            &config.codex_home,
        )?)
    }

    fn existing_layers(config: &Config) -> Vec<ConfigLayerEntry> {
        config
            .config_layer_stack
            .get_layers(
                ConfigLayerStackOrdering::LowestPrecedenceFirst,
                /*include_disabled*/ true,
            )
            .into_iter()
            .cloned()
            .collect()
    }

    fn insert_layer(layers: &mut Vec<ConfigLayerEntry>, layer: ConfigLayerEntry) {
        let insertion_index =
            layers.partition_point(|existing_layer| existing_layer.name <= layer.name);
        layers.insert(insertion_index, layer);
    }

    fn role_layer(role_layer_toml: TomlValue) -> ConfigLayerEntry {
        ConfigLayerEntry::new(ConfigLayerSource::SessionFlags, role_layer_toml)
    }

    fn reload_overrides(
        config: &Config,
        preserve_current_model: bool,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> ConfigOverrides {
        ConfigOverrides {
            cwd: Some(config.cwd.to_path_buf()),
            model: preserve_current_model
                .then(|| config.model.clone())
                .flatten(),
            model_provider: preserve_current_provider.then(|| config.model_provider_id.clone()),
            service_tier: preserve_current_service_tier.then(|| config.service_tier.clone()),
            codex_linux_sandbox_exe: config.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: config.main_execve_wrapper_exe.clone(),
            ..Default::default()
        }
    }
}

pub(crate) mod spawn_tool_spec {
    use super::*;

    /// Builds the spawn-agent tool description text from configured roles.
    pub(crate) fn build(user_defined_agent_roles: &BTreeMap<String, AgentRoleConfig>) -> String {
        build_from_configs(user_defined_agent_roles)
    }

    // This function is not inlined for testing purpose.
    fn build_from_configs(user_defined_roles: &BTreeMap<String, AgentRoleConfig>) -> String {
        let formatted_roles = user_defined_roles
            .iter()
            .map(|(name, declaration)| format_role(name, declaration))
            .collect::<Vec<_>>();

        format!("Available roles:\n{}", formatted_roles.join("\n"))
    }

    fn format_role(name: &str, declaration: &AgentRoleConfig) -> String {
        if let Some(description) = &declaration.description {
            let locked_settings_note = declaration
                .config_file
                .as_ref()
                .and_then(|config_file| std::fs::read_to_string(config_file).ok())
                .and_then(|contents| toml::from_str::<TomlValue>(&contents).ok())
                .map(|role_toml| {
                    let model = role_toml
                        .get("model")
                        .and_then(TomlValue::as_str);
                    let reasoning_effort = role_toml
                        .get("model_reasoning_effort")
                        .and_then(TomlValue::as_str);
                    let service_tier = role_toml
                        .get("service_tier")
                        .and_then(TomlValue::as_str);

                    let model_and_reasoning_note = match (model, reasoning_effort) {
                        (Some(model), Some(reasoning_effort)) => format!(
                            "\n- This role's model is set to `{model}` and its reasoning effort is set to `{reasoning_effort}`. These settings cannot be changed."
                        ),
                        (Some(model), None) => {
                            format!(
                                "\n- This role's model is set to `{model}` and cannot be changed."
                            )
                        }
                        (None, Some(reasoning_effort)) => {
                            format!(
                                "\n- This role's reasoning effort is set to `{reasoning_effort}` and cannot be changed."
                            )
                        }
                        (None, None) => String::new(),
                    };
                    let service_tier_note = service_tier
                        .map(|service_tier| {
                            format!(
                                "\n- This role's service tier is set to `{service_tier}`. If it is supported by the resolved model, it takes precedence over a valid spawn request service tier."
                            )
                        })
                        .unwrap_or_default();
                    format!("{model_and_reasoning_note}{service_tier_note}")
                })
                .unwrap_or_default();
            format!("{name}: {{\n{description}{locked_settings_note}\n}}")
        } else {
            format!("{name}: no description")
        }
    }
}

#[cfg(test)]
#[path = "role_tests.rs"]
mod tests;
