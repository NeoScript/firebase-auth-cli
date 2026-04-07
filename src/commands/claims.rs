use std::collections::BTreeMap;

use anyhow::{Result, bail};
use indicatif::ProgressBar;
use rs_firebase_admin_sdk::auth::{Claims, FirebaseAuthService, UserIdentifiers, UserUpdate};
use serde_json::Value;

use crate::config::resolve_connection;
use crate::firebase::{AuthBackend, init_firebase};
use crate::output::{render_json_value, render_message, render_table};
use crate::prompt::{confirm, resolve_email, resolve_string};
use crate::{Cli, ClaimsCommand};

pub async fn run(cli: &Cli, command: &ClaimsCommand) -> Result<()> {
    match command {
        ClaimsCommand::Get { email } => get(cli, email.clone()).await,
        ClaimsCommand::Merge { key, value, email } => {
            merge(cli, key.clone(), value.clone(), email.clone()).await
        }
        ClaimsCommand::Remove { key, email } => remove(cli, key.clone(), email.clone()).await,
        ClaimsCommand::Clear { email } => clear(cli, email.clone()).await,
        ClaimsCommand::Find {
            key,
            value,
            exclusive,
        } => find(cli, key.clone(), value.clone(), *exclusive).await,
    }
}

fn parse_claim_value(raw: &str) -> Value {
    if raw.starts_with('{') || raw.starts_with('[') {
        if let Ok(v) = serde_json::from_str(raw) {
            return v;
        }
    }
    if raw == "true" {
        return Value::Bool(true);
    }
    if raw == "false" {
        return Value::Bool(false);
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Value::Number(i.into());
    }
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    Value::String(raw.to_string())
}

fn claims_to_map(claims: &Option<Claims>) -> BTreeMap<String, Value> {
    match claims {
        Some(c) => c.get().clone(),
        None => BTreeMap::new(),
    }
}

fn map_to_json(map: &BTreeMap<String, Value>) -> Value {
    Value::Object(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

fn map_to_claims(map: BTreeMap<String, Value>) -> Claims {
    Claims::from(map)
}

macro_rules! fb_anyhow {
    ($expr:expr) => {
        $expr.map_err(|e| anyhow::anyhow!("{e}"))
    };
}

async fn get(cli: &Cli, email: Option<String>) -> Result<()> {
    let email = resolve_email(email)?;
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let identifiers = UserIdentifiers::builder().with_email(email.clone()).build();
    let user = fb_anyhow!(auth.get_user(identifiers).await)?
        .ok_or_else(|| anyhow::anyhow!("User not found: {email}"))?;

    let claims_map = claims_to_map(&user.custom_claims);
    if claims_map.is_empty() {
        render_message("No custom claims set");
    } else {
        render_json_value(&cli.format, &map_to_json(&claims_map));
    }

    Ok(())
}

async fn merge(
    cli: &Cli,
    key: Option<String>,
    value: Option<String>,
    email: Option<String>,
) -> Result<()> {
    let email = resolve_email(email)?;
    let key = resolve_string(key, "Claim key")?;
    let raw_value = resolve_string(value, "Claim value")?;
    let parsed_value = parse_claim_value(&raw_value);

    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let identifiers = UserIdentifiers::builder().with_email(email.clone()).build();
    let user = fb_anyhow!(auth.get_user(identifiers).await)?
        .ok_or_else(|| anyhow::anyhow!("User not found: {email}"))?;

    let mut claims_map = claims_to_map(&user.custom_claims);
    claims_map.insert(key.clone(), parsed_value);

    if cli.dry_run {
        render_message("Dry run — would set claims to:");
        render_json_value(&cli.format, &map_to_json(&claims_map));
        return Ok(());
    }

    let update = UserUpdate::builder(user.uid)
        .custom_claims(map_to_claims(claims_map))
        .build();
    let updated_user = fb_anyhow!(auth.update_user(update).await)?;

    let updated_map = claims_to_map(&updated_user.custom_claims);
    render_json_value(&cli.format, &map_to_json(&updated_map));

    Ok(())
}

async fn remove(cli: &Cli, key: Option<String>, email: Option<String>) -> Result<()> {
    let email = resolve_email(email)?;
    let key = resolve_string(key, "Claim key")?;

    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let identifiers = UserIdentifiers::builder().with_email(email.clone()).build();
    let user = fb_anyhow!(auth.get_user(identifiers).await)?
        .ok_or_else(|| anyhow::anyhow!("User not found: {email}"))?;

    let mut claims_map = claims_to_map(&user.custom_claims);

    if claims_map.remove(&key).is_none() {
        eprintln!("Claim key '{key}' not found");
        return Ok(());
    }

    if cli.dry_run {
        render_message("Dry run — would set claims to:");
        render_json_value(&cli.format, &map_to_json(&claims_map));
        return Ok(());
    }

    let update = UserUpdate::builder(user.uid)
        .custom_claims(map_to_claims(claims_map))
        .build();
    let updated_user = fb_anyhow!(auth.update_user(update).await)?;

    let updated_map = claims_to_map(&updated_user.custom_claims);
    render_json_value(&cli.format, &map_to_json(&updated_map));

    Ok(())
}

async fn clear(cli: &Cli, email: Option<String>) -> Result<()> {
    let email = resolve_email(email)?;

    if !confirm(
        &format!("Clear ALL custom claims for {email}?"),
        cli.yes,
    )? {
        bail!("Aborted");
    }

    if cli.dry_run {
        render_message(&format!(
            "Dry run — would clear all custom claims for {email}"
        ));
        return Ok(());
    }

    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let identifiers = UserIdentifiers::builder().with_email(email.clone()).build();
    let user = fb_anyhow!(auth.get_user(identifiers).await)?
        .ok_or_else(|| anyhow::anyhow!("User not found: {email}"))?;

    let update = UserUpdate::builder(user.uid)
        .custom_claims(map_to_claims(BTreeMap::new()))
        .build();
    fb_anyhow!(auth.update_user(update).await)?;

    render_message(&format!("Cleared all custom claims for {email}"));

    Ok(())
}

async fn find(cli: &Cli, key: String, value: Option<String>, exclusive: bool) -> Result<()> {
    let conn = resolve_connection(
        &cli.profile,
        &cli.project,
        &cli.credentials,
        &cli.emulator_host,
    )?;
    let auth = init_firebase(AuthBackend::from_resolved(&conn)).await?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Scanning users…");

    let mut matching_rows: Vec<Vec<String>> = Vec::new();
    let mut prev_page: Option<_> = None;
    let target_value = value.as_deref().map(parse_claim_value);

    loop {
        spinner.tick();

        let page = fb_anyhow!(auth.list_users(1000, prev_page).await)?;

        let user_list = match page {
            Some(list) => list,
            None => break,
        };

        for user in &user_list.users {
            let claims = match &user.custom_claims {
                Some(c) => c,
                None => continue,
            };

            let claims_map = claims.get();
            let claim_value = match claims_map.get(&key) {
                Some(v) => v,
                None => continue,
            };

            if let Some(ref target) = target_value {
                if exclusive {
                    if let Value::Array(arr) = claim_value {
                        if arr.len() != 1 || arr[0] != *target {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else if claim_value != target {
                    if let Value::Array(arr) = claim_value {
                        if !arr.contains(target) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            }

            matching_rows.push(vec![
                user.uid.clone(),
                user.email.clone().unwrap_or_default(),
                serde_json::to_string(claim_value).unwrap_or_default(),
            ]);

            spinner.set_message(format!("Scanning users… {} matches", matching_rows.len()));
        }

        match user_list.next_page_token {
            Some(ref token) if !token.is_empty() => prev_page = Some(user_list),
            _ => break,
        }
    }

    spinner.finish_and_clear();

    if matching_rows.is_empty() {
        render_message("No matching users found");
    } else {
        render_message(&format!("Found {} matching user(s)", matching_rows.len()));
        render_table(
            &cli.format,
            &["UID", "Email", "Claims Value"],
            &matching_rows,
        );
    }

    Ok(())
}
