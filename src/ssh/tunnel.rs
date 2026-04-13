use crate::model::Connection;
use crate::model::tunnel::Tunnel;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, Stdio};

use super::SystemSshBackend;

struct TunnelProcess {
    child: Child,
}

pub struct TunnelManager {
    processes: HashMap<String, TunnelProcess>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub fn start(&mut self, connection: &Connection, tunnel: &Tunnel) -> Result<(), String> {
        // Don't start if already running
        if self.is_running(&tunnel.id) {
            return Ok(());
        }

        let mut cmd = SystemSshBackend::build_command(connection);
        cmd.arg("-N");

        // Parse the ssh_flag into separate args
        let flag = tunnel.ssh_flag();
        let mut parts = flag.splitn(2, ' ');
        if let (Some(flag_name), Some(flag_value)) = (parts.next(), parts.next()) {
            cmd.arg(flag_name);
            cmd.arg(flag_value);
        }

        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(child) => {
                self.processes
                    .insert(tunnel.id.clone(), TunnelProcess { child });
                Ok(())
            }
            Err(e) => Err(format!("Failed to start tunnel: {e}")),
        }
    }

    pub fn stop(&mut self, tunnel_id: &str) {
        if let Some(mut proc) = self.processes.remove(tunnel_id) {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
    }

    pub fn is_running(&mut self, tunnel_id: &str) -> bool {
        let Some(proc) = self.processes.get_mut(tunnel_id) else {
            return false;
        };

        match proc.child.try_wait() {
            Ok(None) => true, // still running
            Ok(Some(_)) => {
                // Process exited — clean up
                self.processes.remove(tunnel_id);
                false
            }
            Err(_) => {
                self.processes.remove(tunnel_id);
                false
            }
        }
    }

    pub fn stop_all(&mut self) {
        let ids: Vec<String> = self.processes.keys().cloned().collect();
        for id in ids {
            self.stop(&id);
        }
    }

    pub fn active_count(&mut self) -> usize {
        // Clean up dead processes first
        let ids: Vec<String> = self.processes.keys().cloned().collect();
        for id in &ids {
            self.is_running(id);
        }
        self.processes.len()
    }

    #[allow(dead_code)]
    pub fn get_error(&mut self, tunnel_id: &str) -> Option<String> {
        let proc = self.processes.get_mut(tunnel_id)?;

        // Only read stderr if process has exited
        if let Ok(Some(_)) = proc.child.try_wait() {
            if let Some(mut stderr) = proc.child.stderr.take() {
                let mut buf = String::new();
                let _ = stderr.read_to_string(&mut buf);
                if !buf.is_empty() {
                    return Some(buf);
                }
            }
        }
        None
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}
