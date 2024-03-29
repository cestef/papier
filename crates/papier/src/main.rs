#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod config;
pub mod tui;
pub mod utils;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;
use edtui::{actions::Execute, EditorState};
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    utils::{initialize_logging, initialize_panic_handler, version},
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum PapierAction {
    #[default]
    Quit,
    QuitAll,
    Save,
    SaveAll,
    SaveAs(String),
    PreviousBuffer,
    NextBuffer,
    Open(String),
    ToggleExplorer,
    ToggleLogger,
}

impl Execute for PapierAction {
    fn execute(&mut self, state: &mut EditorState) {}
}

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate, args.files)?;
    app.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
