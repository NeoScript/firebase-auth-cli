use anyhow::Result;

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
    todo!()
}

async fn add(
    _cli: &Cli,
    _name: Option<String>,
    _project: Option<String>,
    _credentials: Option<String>,
    _emulator_host: Option<String>,
) -> Result<()> {
    todo!()
}

async fn remove(_cli: &Cli, _name: Option<String>) -> Result<()> {
    todo!()
}

async fn default(_cli: &Cli, _name: Option<String>) -> Result<()> {
    todo!()
}

async fn list(_cli: &Cli) -> Result<()> {
    todo!()
}

async fn show(_cli: &Cli, _name: Option<String>) -> Result<()> {
    todo!()
}

async fn which(_cli: &Cli) -> Result<()> {
    todo!()
}

async fn path(_cli: &Cli) -> Result<()> {
    todo!()
}
