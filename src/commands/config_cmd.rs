use anyhow::{Result, bail};

use crate::config::{
    Profile, add_profile, config_dir, load_config, remove_profile, save_config,
    set_default, resolve_connection,
};
use crate::output::{render_message, render_single_record, render_table};
use crate::prompt::{resolve_select, resolve_string};
use crate::{Cli, ConfigCommand};

pub async fn run(cli: &Cli, command: &ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Init => init(cli).await,
        ConfigCommand::Add {
            name,
            project,
            credentials,
            emulator_host,
        } => {
            add(
                cli,
                name.clone(),
                project.clone(),
                credentials.clone(),
                emulator_host.clone(),
            )
            .await
        }
        ConfigCommand::Remove { name } => remove(cli, name.clone()).await,
        ConfigCommand::Default { name } => default(cli, name.clone()).await,
        ConfigCommand::List => list(cli).await,
        ConfigCommand::Show { name } => show(cli, name.clone()).await,
        ConfigCommand::Which => which(cli).await,
        ConfigCommand::Path => path(cli).await,
    }
}

async fn init(_cli: &Cli) -> Result<()> {
    let name = resolve_string(None, "Profile name")?;

    let auth_methods = vec![
        "ADC (Application Default Credentials)".to_string(),
        "Service Account JSON".to_string(),
        "Emulator".to_string(),
    ];
    let auth_choice = resolve_select(None, "Authentication method", &auth_methods)?;

    let profile = if auth_choice.starts_with("Emulator") {
        let host = resolve_string(
            Some("localhost:9099".to_string()),
            "Emulator host:port (default: localhost:9099)",
        )?;
        Profile {
            emulator_host: Some(host),
            ..Default::default()
        }
    } else if auth_choice.starts_with("Service Account") {
        let creds = resolve_string(None, "Path to service account JSON")?;
        let project = crate::prompt::resolve_optional_string(None, "Project ID")?;
        Profile {
            credentials: Some(creds),
            project,
            ..Default::default()
        }
    } else {
        let project = crate::prompt::resolve_optional_string(None, "Project ID")?;
        Profile {
            project,
            ..Default::default()
        }
    };

    let mut config = load_config()?;
    add_profile(&mut config, name.clone(), profile);

    let make_default = crate::prompt::confirm(
        &format!("Set '{name}' as the default profile?"),
        false,
    )?;
    if make_default {
        config.default_profile = Some(name.clone());
    }

    save_config(&config)?;
    render_message(&format!("Profile '{name}' created."));

    Ok(())
}

async fn add(
    _cli: &Cli,
    name: Option<String>,
    project: Option<String>,
    credentials: Option<String>,
    emulator_host: Option<String>,
) -> Result<()> {
    let name = resolve_string(name, "Profile name")?;

    let profile = Profile {
        project,
        credentials,
        emulator_host,
    };

    if profile.project.is_none() && profile.credentials.is_none() && profile.emulator_host.is_none()
    {
        bail!(
            "At least one of --project, --credentials, or --emulator-host is required.\n\
             Use 'fbadmin config init' for an interactive wizard."
        );
    }

    let mut config = load_config()?;
    let overwriting = config.profiles.contains_key(&name);
    add_profile(&mut config, name.clone(), profile);
    save_config(&config)?;

    if overwriting {
        render_message(&format!("Profile '{name}' updated."));
    } else {
        render_message(&format!("Profile '{name}' added."));
    }

    Ok(())
}

async fn remove(_cli: &Cli, name: Option<String>) -> Result<()> {
    let config = load_config()?;
    let names: Vec<String> = config.profiles.keys().cloned().collect();
    let name = resolve_select(name, "Profile to remove", &names)?;

    let mut config = load_config()?;
    remove_profile(&mut config, &name)?;
    save_config(&config)?;

    render_message(&format!("Profile '{name}' removed."));
    Ok(())
}

async fn default(_cli: &Cli, name: Option<String>) -> Result<()> {
    let config = load_config()?;
    let names: Vec<String> = config.profiles.keys().cloned().collect();
    let name = resolve_select(name, "Default profile", &names)?;

    let mut config = load_config()?;
    set_default(&mut config, &name)?;
    save_config(&config)?;

    render_message(&format!("Default profile set to '{name}'."));
    Ok(())
}

async fn list(cli: &Cli) -> Result<()> {
    let config = load_config()?;

    if config.profiles.is_empty() {
        render_message("No profiles configured. Run 'fbadmin config init' to create one.");
        return Ok(());
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    let default_name = config.default_profile.as_deref().unwrap_or("");

    for (name, profile) in &config.profiles {
        let is_default = if name == default_name { "✓" } else { "" };
        rows.push(vec![
            name.clone(),
            is_default.to_string(),
            profile.project.clone().unwrap_or_default(),
            profile.credentials.clone().unwrap_or_default(),
            profile.emulator_host.clone().unwrap_or_default(),
        ]);
    }

    rows.sort_by(|a, b| a[0].cmp(&b[0]));

    render_table(
        &cli.format,
        &["Name", "Default", "Project", "Credentials", "Emulator Host"],
        &rows,
    );

    Ok(())
}

async fn show(cli: &Cli, name: Option<String>) -> Result<()> {
    let config = load_config()?;
    let names: Vec<String> = config.profiles.keys().cloned().collect();
    let name = resolve_select(name, "Profile to show", &names)?;

    let profile = config
        .profiles
        .get(&name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;

    let is_default = config.default_profile.as_deref() == Some(name.as_str());

    render_single_record(
        &cli.format,
        &[
            ("name", name),
            ("default", is_default.to_string()),
            (
                "project",
                profile.project.clone().unwrap_or_else(|| "(not set)".to_string()),
            ),
            (
                "credentials",
                profile.credentials.clone().unwrap_or_else(|| "(not set)".to_string()),
            ),
            (
                "emulator_host",
                profile.emulator_host.clone().unwrap_or_else(|| "(not set)".to_string()),
            ),
        ],
    );

    Ok(())
}

async fn which(cli: &Cli) -> Result<()> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;

    render_single_record(
        &cli.format,
        &[
            (
                "profile",
                conn.profile_name
                    .unwrap_or_else(|| "(none)".to_string()),
            ),
            (
                "source",
                conn.profile_source
                    .unwrap_or_else(|| "(flags/env)".to_string()),
            ),
            (
                "project",
                conn.project
                    .unwrap_or_else(|| "(auto-detect)".to_string()),
            ),
            (
                "credentials",
                conn.credentials
                    .unwrap_or_else(|| "(ADC)".to_string()),
            ),
            (
                "emulator",
                conn.emulator_host
                    .unwrap_or_else(|| "no".to_string()),
            ),
        ],
    );

    Ok(())
}

async fn path(_cli: &Cli) -> Result<()> {
    let global_path = config_dir()?;
    render_message(&format!("Global: {}", global_path.display()));

    let local_path = std::path::PathBuf::from(".fbadmin.toml");
    if local_path.exists() {
        render_message(&format!("Local:  {}", local_path.display()));
    } else {
        render_message("Local:  (not found)");
    }

    Ok(())
}
