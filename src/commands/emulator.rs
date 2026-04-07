use anyhow::Result;

use crate::{Cli, EmulatorCommand};

pub async fn run(cli: &Cli, command: &EmulatorCommand) -> Result<()> {
    match command {
        EmulatorCommand::ClearUsers => clear_users(cli).await,
        EmulatorCommand::Config => config(cli).await,
    }
}

async fn clear_users(_cli: &Cli) -> Result<()> {
    todo!()
}

async fn config(_cli: &Cli) -> Result<()> {
    todo!()
}
