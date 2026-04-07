use anyhow::{Result, bail};

pub fn resolve_email(email: Option<String>) -> Result<String> {
    if let Some(e) = email {
        return Ok(e);
    }
    if console::user_attended() {
        let input: String = dialoguer::Input::new()
            .with_prompt("Email")
            .interact_text()?;
        Ok(input)
    } else {
        bail!("--email is required (non-interactive mode)")
    }
}

pub fn resolve_uid(uid: Option<String>) -> Result<String> {
    if let Some(u) = uid {
        return Ok(u);
    }
    if console::user_attended() {
        let input: String = dialoguer::Input::new().with_prompt("UID").interact_text()?;
        Ok(input)
    } else {
        bail!("--uid is required (non-interactive mode)")
    }
}

pub fn resolve_email_or_uid(email: Option<String>, uid: Option<String>) -> Result<EmailOrUid> {
    if let Some(e) = email {
        return Ok(EmailOrUid::Email(e));
    }
    if let Some(u) = uid {
        return Ok(EmailOrUid::Uid(u));
    }
    if console::user_attended() {
        let items = vec!["Email", "UID"];
        let selection = dialoguer::Select::new()
            .with_prompt("Lookup by")
            .items(&items)
            .default(0)
            .interact()?;
        match selection {
            0 => {
                let email: String = dialoguer::Input::new()
                    .with_prompt("Email")
                    .interact_text()?;
                Ok(EmailOrUid::Email(email))
            }
            _ => {
                let uid: String = dialoguer::Input::new().with_prompt("UID").interact_text()?;
                Ok(EmailOrUid::Uid(uid))
            }
        }
    } else {
        bail!("--email or --uid is required (non-interactive mode)")
    }
}

pub enum EmailOrUid {
    Email(String),
    Uid(String),
}

pub fn resolve_string(opt: Option<String>, prompt_text: &str) -> Result<String> {
    if let Some(s) = opt {
        return Ok(s);
    }
    if console::user_attended() {
        let input: String = dialoguer::Input::new()
            .with_prompt(prompt_text)
            .interact_text()?;
        Ok(input)
    } else {
        bail!(
            "--{} is required (non-interactive mode)",
            prompt_text.to_lowercase()
        )
    }
}

pub fn resolve_optional_string(opt: Option<String>, prompt_text: &str) -> Result<Option<String>> {
    if let Some(s) = opt {
        return Ok(Some(s));
    }
    if console::user_attended() {
        let input: String = dialoguer::Input::new()
            .with_prompt(format!("{prompt_text} (optional, press Enter to skip)"))
            .allow_empty(true)
            .interact_text()?;
        if input.is_empty() {
            Ok(None)
        } else {
            Ok(Some(input))
        }
    } else {
        Ok(None)
    }
}

pub fn resolve_password(password: Option<String>) -> Result<Option<String>> {
    if let Some(p) = password {
        return Ok(Some(p));
    }
    if console::user_attended() {
        let input: String = dialoguer::Password::new()
            .with_prompt("Password (leave blank to auto-generate)")
            .allow_empty_password(true)
            .interact()?;
        if input.is_empty() {
            Ok(None)
        } else {
            Ok(Some(input))
        }
    } else {
        Ok(None)
    }
}

pub fn resolve_csv_path(path: Option<String>) -> Result<String> {
    if let Some(p) = path {
        return Ok(p);
    }
    if console::user_attended() {
        let input: String = dialoguer::Input::new()
            .with_prompt("Path to CSV")
            .interact_text()?;
        Ok(input)
    } else {
        bail!("--csv is required (non-interactive mode)")
    }
}

pub fn resolve_select(opt: Option<String>, prompt_text: &str, items: &[String]) -> Result<String> {
    if let Some(s) = opt {
        return Ok(s);
    }
    if items.is_empty() {
        bail!("No items to select from");
    }
    if console::user_attended() {
        let selection = dialoguer::Select::new()
            .with_prompt(prompt_text)
            .items(items)
            .default(0)
            .interact()?;
        Ok(items[selection].clone())
    } else {
        bail!("Selection required (non-interactive mode)")
    }
}

pub fn confirm(prompt_text: &str, yes_flag: bool) -> Result<bool> {
    if yes_flag {
        return Ok(true);
    }
    if console::user_attended() {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(prompt_text)
            .default(false)
            .interact()?;
        Ok(confirmed)
    } else {
        bail!("Confirmation required but running non-interactively. Use --yes to skip.")
    }
}
