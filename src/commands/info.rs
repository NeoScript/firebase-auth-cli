use anyhow::Result;
use rs_firebase_admin_sdk::auth::FirebaseAuthService;

use crate::config::resolve_connection;
use crate::errors::IntoAnyhow;
use crate::firebase::{AuthBackend, init_firebase};
use crate::output::{render_message, render_single_record};
use crate::Cli;

pub async fn run(cli: &Cli) -> Result<()> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;

    let fields = [
        (
            "profile",
            conn.profile_name
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
        ),
        (
            "source",
            conn.profile_source
                .clone()
                .unwrap_or_else(|| "(flags/env)".to_string()),
        ),
        (
            "project",
            conn.project
                .clone()
                .unwrap_or_else(|| "(auto-detect)".to_string()),
        ),
        (
            "credentials",
            conn.credentials
                .clone()
                .unwrap_or_else(|| "(ADC)".to_string()),
        ),
        (
            "emulator",
            conn.emulator_host
                .clone()
                .unwrap_or_else(|| "no".to_string()),
        ),
    ];

    render_single_record(&cli.format, &fields);

    match init_firebase(AuthBackend::from_resolved(&conn)).await {
        Ok(auth) => match auth.list_users(1, None).await.into_anyhow() {
            Ok(_) => render_message("status: connected ✓"),
            Err(err) => render_message(&format!("status: error - {err}")),
        },
        Err(err) => render_message(&format!("status: error - {err}")),
    }

    Ok(())
}
