use anyhow::Result;
use rs_firebase_admin_sdk::auth::FirebaseEmulatorAuthService;

use crate::config::resolve_connection;
use crate::errors::IntoAnyhow;
use crate::firebase::{AuthBackend, init_firebase, require_emulator};
use crate::output::{render_json_value, render_message};
use crate::{Cli, EmulatorCommand};

pub async fn run(cli: &Cli, command: &EmulatorCommand) -> Result<()> {
    match command {
        EmulatorCommand::ClearUsers => clear_users(cli).await,
        EmulatorCommand::Config => config(cli).await,
    }
}

async fn clear_users(cli: &Cli) -> Result<()> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    require_emulator(&conn)?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    auth.clear_all_users().await.into_anyhow()?;

    render_message("All users cleared.");

    Ok(())
}

async fn config(cli: &Cli) -> Result<()> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    require_emulator(&conn)?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let config = auth.get_emulator_configuration().await.into_anyhow()?;
    let value = serde_json::to_value(&config)?;

    render_json_value(&cli.format, &value);

    Ok(())
}
