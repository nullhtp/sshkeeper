mod model;
mod ssh;
mod storage;
mod ui;

use anyhow::Result;
use storage::{TomlStorage, TransferHistory};
use ui::App;

fn main() -> Result<()> {
    // Install panic hook that restores the terminal
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        default_hook(info);
    }));

    let storage = TomlStorage::new()?;
    let connections = storage.load()?;
    let transfer_history = TransferHistory::load()?;

    let mut terminal = ratatui::init();
    let mut app = App::new(storage, connections, transfer_history);
    let result = app.run(&mut terminal);
    ratatui::restore();

    result
}
