use anyhow::Result;

use crate::{Cli, UsersCommand};

pub async fn run(cli: &Cli, command: &UsersCommand) -> Result<()> {
    match command {
        UsersCommand::Get { email, uid } => get(cli, email.clone(), uid.clone()).await,
        UsersCommand::Create {
            email,
            password,
            display_name,
        } => {
            create(cli, email.clone(), password.clone(), display_name.clone()).await
        }
        UsersCommand::Disable { email } => disable(cli, email.clone()).await,
        UsersCommand::Enable { email } => enable(cli, email.clone()).await,
        UsersCommand::Remove { csv } => remove(cli, csv.clone()).await,
        UsersCommand::List { limit } => list(cli, *limit).await,
        UsersCommand::ListInactive { days } => list_inactive(cli, *days).await,
        UsersCommand::Count => count(cli).await,
    }
}

async fn get(_cli: &Cli, _email: Option<String>, _uid: Option<String>) -> Result<()> {
    todo!()
}

async fn create(
    _cli: &Cli,
    _email: Option<String>,
    _password: Option<String>,
    _display_name: Option<String>,
) -> Result<()> {
    todo!()
}

async fn disable(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn enable(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn remove(_cli: &Cli, _csv: Option<String>) -> Result<()> {
    todo!()
}

async fn list(_cli: &Cli, _limit: Option<usize>) -> Result<()> {
    todo!()
}

async fn list_inactive(_cli: &Cli, _days: u64) -> Result<()> {
    todo!()
}

async fn count(_cli: &Cli) -> Result<()> {
    todo!()
}
