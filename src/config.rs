use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FbadminConfig {
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Profile {
    pub project: Option<String>,
    pub credentials: Option<String>,
    pub emulator_host: Option<String>,
}

pub struct ResolvedConnection {
    pub profile_name: Option<String>,
    pub profile_source: Option<String>,
    pub project: Option<String>,
    pub credentials: Option<String>,
    pub emulator_host: Option<String>,
}

pub fn config_dir() -> Result<PathBuf> {
    let path = confy::get_configuration_file_path("fbadmin", "config")
        .context("Failed to determine config path")?;
    Ok(path)
}

pub fn load_config() -> Result<FbadminConfig> {
    let global: FbadminConfig =
        confy::load("fbadmin", "config").context("Failed to load global config")?;

    let local_path = PathBuf::from(".fbadmin.toml");
    if local_path.exists() {
        let local: FbadminConfig =
            confy::load_path(&local_path).context("Failed to load local config (.fbadmin.toml)")?;
        Ok(merge_configs(global, local))
    } else {
        Ok(global)
    }
}

fn merge_configs(global: FbadminConfig, local: FbadminConfig) -> FbadminConfig {
    let mut merged = global;

    for (name, local_profile) in local.profiles {
        let entry = merged.profiles.entry(name).or_default();
        if local_profile.project.is_some() {
            entry.project = local_profile.project;
        }
        if local_profile.credentials.is_some() {
            entry.credentials = local_profile.credentials;
        }
        if local_profile.emulator_host.is_some() {
            entry.emulator_host = local_profile.emulator_host;
        }
    }

    if local.default_profile.is_some() {
        merged.default_profile = local.default_profile;
    }

    merged
}

pub fn resolve_profile_name(
    cli_profile: &Option<String>,
    config: &FbadminConfig,
) -> Result<Option<String>> {
    if let Some(name) = cli_profile {
        if !config.profiles.contains_key(name) {
            let available: Vec<&str> = config.profiles.keys().map(|s| s.as_str()).collect();
            bail!(
                "Profile '{}' not found. Available: {}",
                name,
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available.join(", ")
                }
            );
        }
        return Ok(Some(name.clone()));
    }

    if let Ok(env_profile) = std::env::var("FBADMIN_PROFILE") {
        if !config.profiles.contains_key(&env_profile) {
            let available: Vec<&str> = config.profiles.keys().map(|s| s.as_str()).collect();
            bail!(
                "Profile '{}' (from FBADMIN_PROFILE) not found. Available: {}",
                env_profile,
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available.join(", ")
                }
            );
        }
        return Ok(Some(env_profile));
    }

    if let Some(ref default) = config.default_profile {
        if config.profiles.contains_key(default) {
            return Ok(Some(default.clone()));
        }
    }

    Ok(None)
}

pub fn resolve_connection(
    cli_profile: &Option<String>,
    cli_project: &Option<String>,
    cli_credentials: &Option<String>,
    cli_emulator_host: &Option<String>,
) -> Result<ResolvedConnection> {
    let config = load_config()?;
    let profile_name = resolve_profile_name(cli_profile, &config)?;

    let (profile, profile_source) = if let Some(ref name) = profile_name {
        let p = config.profiles.get(name).cloned().unwrap_or_default();
        let source = if cli_profile.is_some() {
            "cli flag".to_string()
        } else if std::env::var("FBADMIN_PROFILE").is_ok() {
            "env:FBADMIN_PROFILE".to_string()
        } else {
            "default_profile".to_string()
        };
        (p, Some(source))
    } else {
        (Profile::default(), None)
    };

    let emulator_host = cli_emulator_host
        .clone()
        .or(profile.emulator_host);

    let project = cli_project
        .clone()
        .or(profile.project);

    let credentials = cli_credentials
        .clone()
        .or_else(|| {
            profile
                .credentials
                .map(|c| shellexpand::tilde(&c).to_string())
        });

    Ok(ResolvedConnection {
        profile_name,
        profile_source,
        project,
        credentials,
        emulator_host,
    })
}

pub fn save_config(config: &FbadminConfig) -> Result<()> {
    confy::store("fbadmin", "config", config).context("Failed to save config")?;
    Ok(())
}

pub fn add_profile(config: &mut FbadminConfig, name: String, profile: Profile) {
    config.profiles.insert(name, profile);
}

pub fn remove_profile(config: &mut FbadminConfig, name: &str) -> Result<()> {
    if config.profiles.remove(name).is_none() {
        bail!("Profile '{}' not found", name);
    }
    if config.default_profile.as_deref() == Some(name) {
        config.default_profile = None;
    }
    Ok(())
}

pub fn set_default(config: &mut FbadminConfig, name: &str) -> Result<()> {
    if !config.profiles.contains_key(name) {
        bail!("Profile '{}' not found", name);
    }
    config.default_profile = Some(name.to_string());
    Ok(())
}
