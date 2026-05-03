use std::time::SystemTime;

use anyhow::{Context, Result, anyhow, bail};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rs_firebase_admin_sdk::auth::{
    AttributeOp, FirebaseAuthService, NewUser, User, UserIdentifiers, UserUpdate,
};
use time::OffsetDateTime;

use crate::config::resolve_connection;
use crate::errors::IntoAnyhow;
use crate::firebase::{AuthBackend, init_firebase};
use crate::output::{render_message, render_single_record, render_success, render_table};
use crate::prompt::{
    EmailOrUid, confirm, resolve_csv_path, resolve_email, resolve_email_or_uid,
    resolve_optional_string, resolve_password,
};
use crate::{Cli, UsersCommand};

pub async fn run(cli: &Cli, command: &UsersCommand) -> Result<()> {
    match command {
        UsersCommand::Get { email, uid } => get(cli, email.clone(), uid.clone()).await,
        UsersCommand::Create {
            email,
            password,
            display_name,
        } => create(cli, email.clone(), password.clone(), display_name.clone()).await,
        UsersCommand::Disable { email } => disable(cli, email.clone()).await,
        UsersCommand::Enable { email } => enable(cli, email.clone()).await,
        UsersCommand::Remove { email, csv } => remove(cli, email.clone(), csv.clone()).await,
        UsersCommand::List { limit } => list(cli, *limit).await,
        UsersCommand::ListInactive { days } => list_inactive(cli, *days).await,
        UsersCommand::Count => count(cli).await,
    }
}

async fn build_auth(
    cli: &Cli,
) -> Result<
    rs_firebase_admin_sdk::auth::FirebaseAuth<rs_firebase_admin_sdk::client::ReqwestApiClient>,
> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let backend = AuthBackend::from_resolved(&conn);
    init_firebase(backend).await
}

fn format_epoch_ms(dt: &OffsetDateTime) -> String {
    let format =
        time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")
            .expect("valid format");
    dt.format(&format).unwrap_or_else(|_| "N/A".to_string())
}

fn user_epoch_ms(user: &User, field: &str) -> Option<OffsetDateTime> {
    match field {
        "last_login_at" => user.last_login_at.clone().map(OffsetDateTime::from),
        "created_at" => user.created_at.clone().map(OffsetDateTime::from),
        _ => None,
    }
}

fn format_providers(user: &User) -> String {
    user.provider_user_info
        .as_ref()
        .map(|providers| {
            providers
                .iter()
                .map(|p| p.provider_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

fn format_claims(user: &User) -> String {
    user.custom_claims
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default()
}

async fn lookup_user_by_email(
    auth: &rs_firebase_admin_sdk::auth::FirebaseAuth<
        rs_firebase_admin_sdk::client::ReqwestApiClient,
    >,
    email: &str,
) -> Result<User> {
    tracing::debug!("Looking up user by email: {email}");
    let ids = UserIdentifiers::builder()
        .with_email(email.to_string())
        .build();
    auth.get_user(ids)
        .await
        .into_anyhow()
        .context(format!("Failed to fetch user {email}"))?
        .ok_or_else(|| anyhow!("User not found: {email}"))
}

async fn get(cli: &Cli, email: Option<String>, uid: Option<String>) -> Result<()> {
    let auth = build_auth(cli).await?;
    let lookup = resolve_email_or_uid(email, uid)?;

    let ids = match &lookup {
        EmailOrUid::Email(e) => UserIdentifiers::builder().with_email(e.clone()).build(),
        EmailOrUid::Uid(u) => UserIdentifiers::builder().with_uid(u.clone()).build(),
    };

    tracing::debug!("Fetching user info");
    let user = auth
        .get_user(ids)
        .await
        .into_anyhow()
        .context("Failed to fetch user")?
        .ok_or_else(|| anyhow!("User not found"))?;

    let last_login = user_epoch_ms(&user, "last_login_at")
        .map(|dt| format_epoch_ms(&dt))
        .unwrap_or_else(|| "Never".to_string());
    let created = user_epoch_ms(&user, "created_at")
        .map(|dt| format_epoch_ms(&dt))
        .unwrap_or_else(|| "N/A".to_string());
    let providers = format_providers(&user);
    let claims = format_claims(&user);
    let disabled_val = user.disabled.unwrap_or(false);
    let disabled = if disabled_val {
        console::style("true").red().to_string()
    } else {
        console::style("false").green().to_string()
    };
    let email_str = user.email.unwrap_or_else(|| "N/A".to_string());
    let display_name = user.display_name.unwrap_or_else(|| "N/A".to_string());

    render_single_record(
        &cli.format,
        &[
            ("UID", user.uid),
            ("Email", email_str),
            ("Display Name", display_name),
            ("Disabled", disabled),
            ("Created", created),
            ("Last Login", last_login),
            ("Providers", providers),
            ("Claims", claims),
        ],
    );

    Ok(())
}

async fn create(
    cli: &Cli,
    email: Option<String>,
    password: Option<String>,
    display_name: Option<String>,
) -> Result<()> {
    let auth = build_auth(cli).await?;
    let email = resolve_email(email)?;
    let password_input = resolve_password(password)?;
    let display_name = resolve_optional_string(display_name, "Display name")?;

    let auto_generated = password_input.is_none();
    let password = password_input.unwrap_or_else(|| {
        rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    });

    tracing::debug!("Creating user {email}");
    let user = auth
        .create_user(NewUser::email_and_password(email.clone(), password.clone()))
        .await
        .into_anyhow()
        .context(format!("Failed to create user {email}"))?;

    let user = if let Some(ref name) = display_name {
        let update = UserUpdate::builder(user.uid.clone())
            .display_name(AttributeOp::Change(name.clone()))
            .build();
        auth.update_user(update)
            .await
            .into_anyhow()
            .context(format!("Failed to set display name for {email}"))?
    } else {
        user
    };

    let mut fields = vec![
        ("UID", user.uid),
        ("Email", user.email.unwrap_or(email)),
        (
            "Display Name",
            user.display_name.unwrap_or_else(|| "N/A".to_string()),
        ),
    ];
    if auto_generated {
        fields.push(("Password", password));
    }

    render_single_record(&cli.format, &fields);
    Ok(())
}

async fn disable(cli: &Cli, email: Option<String>) -> Result<()> {
    let auth = build_auth(cli).await?;
    let email = resolve_email(email)?;
    let user = lookup_user_by_email(&auth, &email).await?;

    if !confirm(&format!("Disable user {email}?"), cli.yes)? {
        render_message("Cancelled.");
        return Ok(());
    }

    tracing::debug!("Disabling user {email}");
    let update = UserUpdate::builder(user.uid).disabled(true).build();
    auth.update_user(update)
        .await
        .into_anyhow()
        .context(format!("Failed to disable user {email}"))?;

    render_success(&format!("User {email} has been disabled."));
    Ok(())
}

async fn enable(cli: &Cli, email: Option<String>) -> Result<()> {
    let auth = build_auth(cli).await?;
    let email = resolve_email(email)?;
    let user = lookup_user_by_email(&auth, &email).await?;

    tracing::debug!("Enabling user {email}");
    let update = UserUpdate::builder(user.uid).disabled(false).build();
    auth.update_user(update)
        .await
        .into_anyhow()
        .context(format!("Failed to enable user {email}"))?;

    render_success(&format!("User {email} has been enabled."));
    Ok(())
}

async fn remove(cli: &Cli, email: Option<String>, csv_path: Option<String>) -> Result<()> {
    if let Some(email) = email {
        return remove_single(cli, email).await;
    }
    remove_bulk(cli, csv_path).await
}

async fn remove_single(cli: &Cli, email: String) -> Result<()> {
    let auth = build_auth(cli).await?;
    let user = lookup_user_by_email(&auth, &email).await?;

    if cli.dry_run {
        render_message(&format!(
            "Dry run: would delete user {email} ({})",
            user.uid
        ));
        return Ok(());
    }

    if !confirm(
        &format!("Delete user {email} ({})? This cannot be undone.", user.uid),
        cli.yes,
    )? {
        render_message("Cancelled.");
        return Ok(());
    }

    tracing::debug!("Deleting user {} ({})", email, user.uid);
    auth.delete_users(vec![user.uid], true)
        .await
        .into_anyhow()
        .context(format!("Failed to delete user {email}"))?;

    render_success(&format!("Deleted user {email}."));
    Ok(())
}

async fn remove_bulk(cli: &Cli, csv_path: Option<String>) -> Result<()> {
    let auth = build_auth(cli).await?;
    let path = resolve_csv_path(csv_path)?;

    let mut rdr = csv::ReaderBuilder::default().from_path(&path)?;
    let headers = rdr.headers()?.clone();

    let uid_col = headers.iter().position(|h| h.eq_ignore_ascii_case("uid"));
    let email_col = headers.iter().position(|h| h.eq_ignore_ascii_case("email"));

    if uid_col.is_none() && email_col.is_none() {
        bail!("CSV must contain a 'uid' or 'email' column header");
    }

    let use_uid = uid_col.is_some();
    let col_idx = if use_uid {
        uid_col.unwrap()
    } else {
        email_col.unwrap()
    };

    let mut values: Vec<String> = Vec::new();
    for result in rdr.records() {
        let record = result?;
        if let Some(val) = record.get(col_idx) {
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() {
                values.push(trimmed);
            }
        }
    }

    if values.is_empty() {
        bail!("No entries found in CSV");
    }

    let uids: Vec<String> = if use_uid {
        values
    } else {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} Looking up UIDs... ({pos} resolved)")
                .unwrap(),
        );
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let mut resolved = Vec::with_capacity(values.len());
        let mut not_found: Vec<String> = Vec::new();
        for email in &values {
            let ids = UserIdentifiers::builder().with_email(email.clone()).build();
            match auth
                .get_user(ids)
                .await
                .into_anyhow()
                .context(format!("Failed to look up {email}"))?
            {
                Some(user) => resolved.push(user.uid),
                None => not_found.push(email.clone()),
            }
            spinner.inc(1);
        }
        spinner.finish_and_clear();

        if !not_found.is_empty() {
            eprintln!(
                "Warning: {} email(s) not found and will be skipped: {}",
                not_found.len(),
                not_found
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if not_found.len() > 5 {
                eprintln!("  ... and {} more", not_found.len() - 5);
            }
        }

        resolved
    };

    if uids.is_empty() {
        bail!("No users to delete after resolving identifiers");
    }

    if cli.dry_run {
        render_message(&format!("Dry run: would delete {} user(s):", uids.len()));
        for uid in &uids {
            render_message(&format!("  {uid}"));
        }
        return Ok(());
    }

    if !confirm(
        &format!("Delete {} user(s)? This cannot be undone.", uids.len()),
        cli.yes,
    )? {
        render_message("Cancelled.");
        return Ok(());
    }

    let total = uids.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} deleted")
            .unwrap()
            .progress_chars("=> "),
    );

    for batch in uids.chunks(1000) {
        tracing::debug!("Deleting batch of {} users", batch.len());
        auth.delete_users(batch.to_vec(), true)
            .await
            .into_anyhow()
            .context("Failed to delete user batch")?;
        pb.inc(batch.len() as u64);
    }
    pb.finish_and_clear();

    render_success(&format!("Deleted {} user(s).", total));
    Ok(())
}

async fn list(cli: &Cli, limit: Option<usize>) -> Result<()> {
    let auth = build_auth(cli).await?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Fetching users... ({msg})")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut page = None;
    let max = limit.unwrap_or(usize::MAX);

    loop {
        let result = auth
            .list_users(1000, page)
            .await
            .into_anyhow()
            .context("Failed to list users")?;
        match result {
            Some(user_list) => {
                for user in &user_list.users {
                    if rows.len() >= max {
                        break;
                    }
                    let last_login = user_epoch_ms(user, "last_login_at")
                        .map(|dt| format_epoch_ms(&dt))
                        .unwrap_or_else(|| "Never".to_string());
                    rows.push(vec![
                        user.uid.clone(),
                        user.email.clone().unwrap_or_default(),
                        user.disabled.unwrap_or(false).to_string(),
                        last_login,
                    ]);
                }
                spinner.set_message(format!("{} users", rows.len()));
                if rows.len() >= max {
                    break;
                }
                page = Some(user_list);
            }
            None => break,
        }
    }

    spinner.finish_and_clear();

    render_table(
        &cli.format,
        &["UID", "Email", "Disabled", "Last Login"],
        &rows,
    );

    Ok(())
}

async fn list_inactive(cli: &Cli, days: u64) -> Result<()> {
    let auth = build_auth(cli).await?;

    let threshold_ms = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        - (days as i64 * 86_400 * 1000);

    let threshold_dt = OffsetDateTime::from_unix_timestamp_nanos(threshold_ms as i128 * 1_000_000)
        .map_err(|e| anyhow!("Invalid threshold timestamp: {e}"))?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Scanning users... ({msg})")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut scanned: usize = 0;
    let mut page = None;

    loop {
        let result = auth
            .list_users(1000, page)
            .await
            .into_anyhow()
            .context("Failed to list users during inactivity scan")?;
        match result {
            Some(user_list) => {
                for user in &user_list.users {
                    scanned += 1;
                    let relevant_dt = user_epoch_ms(user, "last_login_at")
                        .or_else(|| user_epoch_ms(user, "created_at"));

                    let is_inactive = match relevant_dt {
                        Some(dt) => dt < threshold_dt,
                        None => true,
                    };

                    if is_inactive {
                        let last_login = user_epoch_ms(user, "last_login_at")
                            .map(|dt| format_epoch_ms(&dt))
                            .unwrap_or_else(|| "Never".to_string());
                        rows.push(vec![
                            user.uid.clone(),
                            user.email.clone().unwrap_or_default(),
                            user.disabled.unwrap_or(false).to_string(),
                            last_login,
                        ]);
                    }
                }
                spinner.set_message(format!("scanned {scanned}, found {} inactive", rows.len()));
                page = Some(user_list);
            }
            None => break,
        }
    }

    spinner.finish_and_clear();

    render_message(&format!(
        "Found {} user(s) inactive for more than {} days (out of {} total).",
        rows.len(),
        days,
        scanned,
    ));
    render_table(
        &cli.format,
        &["UID", "Email", "Disabled", "Last Login"],
        &rows,
    );

    Ok(())
}

async fn count(cli: &Cli) -> Result<()> {
    let auth = build_auth(cli).await?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Counting users... ({msg})")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut total: usize = 0;
    let mut page = None;

    loop {
        let result = auth
            .list_users(1000, page)
            .await
            .into_anyhow()
            .context("Failed to list users during count")?;
        match result {
            Some(user_list) => {
                total += user_list.users.len();
                spinner.set_message(format!("{total}"));
                page = Some(user_list);
            }
            None => break,
        }
    }

    spinner.finish_and_clear();
    render_message(&format!("Total users: {total}"));

    Ok(())
}
