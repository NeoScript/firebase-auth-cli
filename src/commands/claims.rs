use anyhow::Result;

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

async fn get(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn merge(
    _cli: &Cli,
    _key: Option<String>,
    _value: Option<String>,
    _email: Option<String>,
) -> Result<()> {
    todo!()
}

async fn remove(_cli: &Cli, _key: Option<String>, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn clear(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn find(
    _cli: &Cli,
    _key: String,
    _value: Option<String>,
    _exclusive: bool,
) -> Result<()> {
    todo!()
}
