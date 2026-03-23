mod consul;
mod structs;
mod ui;

use std::env;
use crate::structs::{AppState, Service};

fn main() -> anyhow::Result<()> {
    color_eyre::install().map_err(|err| anyhow::anyhow!(err))?;
    let env = env::args().nth(1).unwrap_or("dev".to_string());

    ratatui::run(|_terminal| {
        let mut app = AppState::new(env);
        ratatui::run(|terminal| app.run(terminal))
    })
}