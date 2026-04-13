use crate::model::{Connection, ConnectionStore};
use crate::ssh::{SshBackend, SystemSshBackend};
use crate::storage::{TomlStorage, TransferHistory};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::time::Duration;

use super::browse::BrowseState;
use super::detail::DetailState;
use super::editor::{EditorMode, EditorState};
use super::transfer::{TransferAction, TransferScreen};

pub enum Screen {
    Browse(BrowseState),
    Detail(DetailState),
    Editor(EditorState),
    Transfer {
        conn_id: String,
        state: Box<TransferScreen>,
    },
}

pub struct App {
    pub store: ConnectionStore,
    pub storage: TomlStorage,
    pub transfer_history: TransferHistory,
    pub screen: Screen,
    pub ssh_backend: SystemSshBackend,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

#[allow(clippy::unnecessary_wraps)]
impl App {
    pub fn new(
        storage: TomlStorage,
        connections: Vec<Connection>,
        transfer_history: TransferHistory,
    ) -> Self {
        Self {
            store: ConnectionStore::new(connections),
            storage,
            transfer_history,
            screen: Screen::Browse(BrowseState::new()),
            ssh_backend: SystemSshBackend,
            status_message: None,
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key, terminal)?;
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match &mut self.screen {
            Screen::Browse(state) => {
                state.render(frame, &self.store, self.status_message.as_deref());
            }
            Screen::Detail(state) => state.render(frame, &self.store),
            Screen::Editor(state) => state.render(frame),
            Screen::Transfer { state, .. } => state.render(frame),
        }
    }

    fn handle_key(&mut self, key: KeyEvent, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        // Global Ctrl+C quit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return Ok(());
        }

        match &mut self.screen {
            Screen::Browse(state) => match state.handle_key(key, &self.store) {
                BrowseAction::None => {}
                BrowseAction::Quit => self.should_quit = true,
                BrowseAction::ViewDetail(id) => {
                    self.screen = Screen::Detail(DetailState::new(id));
                }
                BrowseAction::AddNew => {
                    self.screen = Screen::Editor(EditorState::new_add());
                }
                BrowseAction::Import => {
                    self.do_import();
                }
            },
            Screen::Detail(state) => match state.handle_key(key, &self.store) {
                DetailAction::None => {}
                DetailAction::Back => {
                    self.status_message = None;
                    self.screen = Screen::Browse(BrowseState::new());
                }
                DetailAction::Connect(id) => {
                    self.do_connect(&id, terminal)?;
                }
                DetailAction::Edit(id) => {
                    if let Some(conn) = self.store.find_by_id(&id) {
                        self.screen = Screen::Editor(EditorState::new_edit(conn.clone()));
                    }
                }
                DetailAction::Delete(id) => {
                    if self.store.remove(&id) {
                        self.storage.save(self.store.all())?;
                        self.status_message = Some("Connection deleted.".into());
                        self.screen = Screen::Browse(BrowseState::new());
                    }
                }
                DetailAction::SetupKeyAuth(id) => {
                    self.do_setup_key_auth(&id, terminal)?;
                }
                DetailAction::Transfer(id) => {
                    self.do_open_transfer(&id);
                }
                DetailAction::RunRemoteAction { conn_id, command } => {
                    self.do_run_quick_action(&conn_id, &command, terminal)?;
                }
            },
            Screen::Transfer { conn_id, state } => match state.handle_key(key) {
                TransferAction::None => {}
                TransferAction::Cancel => {
                    let id = conn_id.clone();
                    self.screen = Screen::Detail(DetailState::new(id));
                }
                TransferAction::Execute {
                    local_path,
                    remote_path,
                    direction,
                    recursive,
                } => {
                    let id = conn_id.clone();
                    self.do_transfer(
                        &id,
                        &local_path,
                        &remote_path,
                        direction,
                        recursive,
                        terminal,
                    )?;
                }
            },
            Screen::Editor(state) => match state.handle_key(key) {
                EditorAction::None => {}
                EditorAction::Cancel => {
                    self.screen = Screen::Browse(BrowseState::new());
                }
                EditorAction::Save(conn) => {
                    let conn = *conn;
                    match state.mode {
                        EditorMode::Add => {
                            self.store.add(conn);
                            self.status_message = Some("Connection added.".into());
                        }
                        EditorMode::Edit => {
                            if let Some(existing) = self.store.find_by_id_mut(&conn.id) {
                                *existing = conn;
                            }
                            self.status_message = Some("Connection updated.".into());
                        }
                    }
                    self.storage.save(self.store.all())?;
                    self.screen = Screen::Browse(BrowseState::new());
                }
            },
        }
        Ok(())
    }

    fn do_connect(&mut self, id: &str, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        let conn = match self.store.find_by_id(id) {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        // Suspend TUI
        ratatui::restore();
        println!("Connecting: {}", conn.ssh_command());
        println!();

        let result = self.ssh_backend.connect(&conn);

        // Resume TUI
        *terminal = ratatui::init();

        match result {
            Ok(()) => {
                self.status_message = Some(format!("Disconnected from {}", conn.name));
            }
            Err(e) => {
                self.status_message = Some(format!("SSH error: {e}"));
            }
        }
        self.screen = Screen::Browse(BrowseState::new());
        Ok(())
    }

    fn do_setup_key_auth(
        &mut self,
        id: &str,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        let conn = match self.store.find_by_id(id) {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        // Suspend TUI
        ratatui::restore();
        println!("Setting up key-based auth for: {}\n", conn.name);

        let result = crate::ssh::key_setup::setup_key_auth(&conn);

        println!("\nPress Enter to continue...");
        let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8]);

        // Resume TUI
        *terminal = ratatui::init();

        match result {
            Ok(key_result) => {
                // Update connection with identity file
                if let Some(existing) = self.store.find_by_id_mut(&conn.id) {
                    existing.identity_file = Some(key_result.private_key_path);
                    existing.updated_at = chrono::Utc::now();
                }
                self.storage.save(self.store.all())?;
                self.status_message = Some(format!("Key auth configured for {}", conn.name));
            }
            Err(e) => {
                self.status_message = Some(format!("Key setup failed: {e}"));
            }
        }
        self.screen = Screen::Detail(DetailState::new(conn.id));
        Ok(())
    }

    fn do_open_transfer(&mut self, id: &str) {
        if let Err(e) = crate::ssh::transfer::validate_scp() {
            self.status_message = Some(e.to_string());
            return;
        }
        let conn = match self.store.find_by_id(id) {
            Some(c) => c.clone(),
            None => return,
        };
        self.screen = Screen::Transfer {
            conn_id: id.to_string(),
            state: Box::new(TransferScreen::new(conn)),
        };
    }

    fn do_transfer(
        &mut self,
        conn_id: &str,
        local_path: &str,
        remote_path: &str,
        direction: crate::ssh::transfer::TransferDirection,
        recursive: bool,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        let conn = match self.store.find_by_id(conn_id) {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        let mut cmd = crate::ssh::transfer::build_scp_command(
            &conn,
            local_path,
            remote_path,
            direction,
            recursive,
        );

        let dir_label = match direction {
            crate::ssh::transfer::TransferDirection::Upload => "Uploading",
            crate::ssh::transfer::TransferDirection::Download => "Downloading",
        };

        // Suspend TUI
        ratatui::restore();
        println!(
            "{}: {} ↔ {}:{}",
            dir_label, local_path, conn.host, remote_path
        );
        println!();

        let result = cmd.status();

        // Resume TUI
        *terminal = ratatui::init();

        match result {
            Ok(status) if status.success() => {
                // Record in history
                let entry = crate::storage::transfer_history::TransferEntry::new(
                    direction,
                    local_path.to_string(),
                    remote_path.to_string(),
                    recursive,
                );
                self.transfer_history.push(conn_id, entry);
                let _ = self.transfer_history.save();

                self.status_message = Some(format!("Transfer complete: {local_path}"));
                self.screen = Screen::Detail(DetailState::new(conn_id.to_string()));
            }
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                self.status_message = Some(format!("Transfer failed (exit code {code})"));
                // Stay on transfer screen — rebuild it
                self.screen = Screen::Transfer {
                    conn_id: conn_id.to_string(),
                    state: Box::new(TransferScreen::new(conn.clone())),
                };
            }
            Err(e) => {
                self.status_message = Some(format!("Transfer error: {e}"));
                self.screen = Screen::Transfer {
                    conn_id: conn_id.to_string(),
                    state: Box::new(TransferScreen::new(conn.clone())),
                };
            }
        }
        Ok(())
    }

    fn do_run_quick_action(
        &mut self,
        conn_id: &str,
        command: &str,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        let conn = match self.store.find_by_id(conn_id) {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        // Suspend TUI
        ratatui::restore();
        println!("Running on {}: {}\n", conn.name, command);

        let mut cmd = crate::ssh::actions::build_ssh_command(&conn, command);
        let result = cmd.status();

        println!("\nPress Enter to continue...");
        let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8]);

        // Resume TUI
        *terminal = ratatui::init();

        match result {
            Ok(status) if status.success() => {
                self.status_message = Some("Action completed.".into());
            }
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                self.status_message = Some(format!("Action failed (exit code {code})"));
            }
            Err(e) => {
                self.status_message = Some(format!("Action error: {e}"));
            }
        }
        self.screen = Screen::Detail(DetailState::new(conn_id.to_string()));
        Ok(())
    }

    fn do_import(&mut self) {
        match crate::storage::import_ssh_config(self.store.all()) {
            Ok(result) => {
                let count = result.imported.len();
                let skipped = result.skipped_duplicates.len();
                for conn in result.imported {
                    self.store.add(conn);
                }
                if let Err(e) = self.storage.save(self.store.all()) {
                    self.status_message = Some(format!("Import save error: {e}"));
                    return;
                }
                let msg = if skipped > 0 {
                    format!("Imported {count} connections, {skipped} duplicates skipped")
                } else {
                    format!("Imported {count} connections")
                };
                self.status_message = Some(msg);
            }
            Err(e) => {
                self.status_message = Some(format!("Import failed: {e}"));
            }
        }
    }
}

pub enum BrowseAction {
    None,
    Quit,
    ViewDetail(String),
    AddNew,
    Import,
}

pub enum DetailAction {
    None,
    Back,
    Connect(String),
    Edit(String),
    Delete(String),
    SetupKeyAuth(String),
    Transfer(String),
    RunRemoteAction { conn_id: String, command: String },
}

pub enum EditorAction {
    None,
    Cancel,
    Save(Box<Connection>),
}
