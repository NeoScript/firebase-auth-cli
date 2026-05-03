use anyhow::{Context, Result};
use rs_firebase_admin_sdk::auth::FirebaseAuthService;
use rs_firebase_admin_sdk::auth::oob_code::{OobCodeAction, OobCodeActionType};

use crate::config::resolve_connection;
use crate::errors::IntoAnyhow;
use crate::firebase::{AuthBackend, init_firebase};
use crate::output::render_message;
use crate::prompt::resolve_email;
use crate::{Cli, LinksCommand};

pub async fn run(cli: &Cli, command: &LinksCommand) -> Result<()> {
    match command {
        LinksCommand::PasswordReset { email } => {
            generate_link(cli, email.clone(), OobCodeActionType::PasswordReset).await
        }
        LinksCommand::EmailVerify { email } => {
            generate_link(cli, email.clone(), OobCodeActionType::VerifyEmail).await
        }
        LinksCommand::SignIn { email } => {
            generate_link(cli, email.clone(), OobCodeActionType::EmailSignin).await
        }
    }
}

async fn generate_link(
    cli: &Cli,
    email: Option<String>,
    action_type: OobCodeActionType,
) -> Result<()> {
    let email = resolve_email(email)?;
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    tracing::debug!("Generating {action_type:?} link for {email}");
    let action = OobCodeAction::builder(action_type, email.clone()).build();
    let link = auth
        .generate_email_action_link(action)
        .await
        .into_anyhow()
        .context(format!("Failed to generate link for {email}"))?;

    render_message(&link);

    Ok(())
}
