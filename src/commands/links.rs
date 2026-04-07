use anyhow::Result;

use crate::{Cli, LinksCommand};

pub async fn run(cli: &Cli, command: &LinksCommand) -> Result<()> {
    match command {
        LinksCommand::PasswordReset { email } => password_reset(cli, email.clone()).await,
        LinksCommand::EmailVerify { email } => email_verify(cli, email.clone()).await,
        LinksCommand::SignIn { email } => sign_in(cli, email.clone()).await,
    }
}

async fn password_reset(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn email_verify(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}

async fn sign_in(_cli: &Cli, _email: Option<String>) -> Result<()> {
    todo!()
}
