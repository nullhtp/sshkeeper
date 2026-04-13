use crate::model::Connection;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    SystemInfo,
    ServiceManagement,
    Network,
    Logs,
    Maintenance,
    Docker,
}

impl ActionCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::SystemInfo => "System Info",
            Self::ServiceManagement => "Service Management",
            Self::Network => "Network",
            Self::Logs => "Logs",
            Self::Maintenance => "Maintenance",
            Self::Docker => "Docker",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Confirm is part of the public API for user-defined actions
pub enum ParamType {
    Text { default: &'static str },
    Select { fetch_command: &'static str },
    Confirm,
}

#[derive(Debug, Clone)]
pub struct ActionParam {
    pub key: &'static str,
    pub label: &'static str,
    pub param_type: ParamType,
}

#[derive(Debug, Clone)]
pub struct QuickAction {
    pub name: &'static str,
    pub category: ActionCategory,
    pub description: &'static str,
    pub command_template: &'static str,
    pub params: Vec<ActionParam>,
    pub confirm_message: Option<&'static str>,
}

impl QuickAction {
    pub fn has_params(&self) -> bool {
        !self.params.is_empty()
    }

    /// Build the final command by substituting shell-escaped param values.
    pub fn build_command(&self, values: &[(String, String)]) -> String {
        let mut cmd = self.command_template.to_string();
        for (key, val) in values {
            let escaped = shell_escape(val);
            cmd = cmd.replace(&format!("{{{key}}}"), &escaped);
        }
        cmd
    }
}

/// Build the ssh command to run a remote command on a connection.
pub fn build_ssh_command(conn: &Connection, remote_cmd: &str) -> Command {
    let mut cmd = super::SystemSshBackend::build_command(conn);
    cmd.arg(remote_cmd);
    cmd
}

/// Shell-escape a value to prevent injection.
pub fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    // If it's safe, return as-is
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' || c == ':')
    {
        return s.to_string();
    }
    // Wrap in single quotes, escaping existing single quotes
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[allow(clippy::too_many_lines)]
pub fn build_actions() -> Vec<QuickAction> {
    vec![
        // === System Info ===
        QuickAction {
            name: "Disk Usage",
            category: ActionCategory::SystemInfo,
            description: "Show disk space usage",
            command_template: "df -h",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Memory Usage",
            category: ActionCategory::SystemInfo,
            description: "Show RAM and swap usage",
            command_template: "free -h",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "System Uptime",
            category: ActionCategory::SystemInfo,
            description: "Show how long the server has been running",
            command_template: "uptime",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Top Processes (CPU)",
            category: ActionCategory::SystemInfo,
            description: "Top 20 processes by CPU usage",
            command_template: "ps aux --sort=-%cpu | head -20",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Top Processes (Memory)",
            category: ActionCategory::SystemInfo,
            description: "Top 20 processes by memory usage",
            command_template: "ps aux --sort=-%mem | head -20",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "OS & Kernel Info",
            category: ActionCategory::SystemInfo,
            description: "Show OS release and kernel version",
            command_template: "uname -a && cat /etc/os-release",
            params: vec![],
            confirm_message: None,
        },
        // === Service Management ===
        QuickAction {
            name: "Restart Service",
            category: ActionCategory::ServiceManagement,
            description: "Restart a running service",
            command_template: "sudo systemctl restart {service}",
            params: vec![ActionParam {
                key: "service",
                label: "Service",
                param_type: ParamType::Select {
                    fetch_command: "systemctl list-units --type=service --state=running --no-legend | awk '{print $1}'",
                },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Stop Service",
            category: ActionCategory::ServiceManagement,
            description: "Stop a running service",
            command_template: "sudo systemctl stop {service}",
            params: vec![ActionParam {
                key: "service",
                label: "Service",
                param_type: ParamType::Select {
                    fetch_command: "systemctl list-units --type=service --state=running --no-legend | awk '{print $1}'",
                },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Start Service",
            category: ActionCategory::ServiceManagement,
            description: "Start an inactive service",
            command_template: "sudo systemctl start {service}",
            params: vec![ActionParam {
                key: "service",
                label: "Service",
                param_type: ParamType::Select {
                    fetch_command: "systemctl list-units --type=service --state=inactive --no-legend | awk '{print $1}'",
                },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Service Status",
            category: ActionCategory::ServiceManagement,
            description: "View status of a service",
            command_template: "systemctl status {service}",
            params: vec![ActionParam {
                key: "service",
                label: "Service",
                param_type: ParamType::Select {
                    fetch_command: "systemctl list-units --type=service --no-legend | awk '{print $1}'",
                },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Service Logs",
            category: ActionCategory::ServiceManagement,
            description: "View recent logs for a service",
            command_template: "journalctl -u {service} -n {lines} --no-pager",
            params: vec![
                ActionParam {
                    key: "service",
                    label: "Service",
                    param_type: ParamType::Select {
                        fetch_command: "systemctl list-units --type=service --no-legend | awk '{print $1}'",
                    },
                },
                ActionParam {
                    key: "lines",
                    label: "Lines",
                    param_type: ParamType::Text { default: "50" },
                },
            ],
            confirm_message: None,
        },
        // === Network ===
        QuickAction {
            name: "Listening Ports",
            category: ActionCategory::Network,
            description: "Show all listening TCP ports",
            command_template: "ss -tlnp",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Active Connections",
            category: ActionCategory::Network,
            description: "Show active TCP connections",
            command_template: "ss -tnp",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Firewall Rules",
            category: ActionCategory::Network,
            description: "Show firewall rules (iptables or ufw)",
            command_template: "sudo iptables -L -n --line-numbers 2>/dev/null || sudo ufw status verbose",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Public IP",
            category: ActionCategory::Network,
            description: "Show the server's public IP address",
            command_template: "curl -s ifconfig.me && echo",
            params: vec![],
            confirm_message: None,
        },
        // === Logs ===
        QuickAction {
            name: "System Logs",
            category: ActionCategory::Logs,
            description: "View recent system journal entries",
            command_template: "journalctl -n {lines} --no-pager",
            params: vec![ActionParam {
                key: "lines",
                label: "Lines",
                param_type: ParamType::Text { default: "50" },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Auth Logs",
            category: ActionCategory::Logs,
            description: "View recent SSH auth log entries",
            command_template: "journalctl -u sshd -n 30 --no-pager",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Search Logs",
            category: ActionCategory::Logs,
            description: "Search system logs by pattern",
            command_template: "journalctl --no-pager -n 200 | grep -i {pattern}",
            params: vec![ActionParam {
                key: "pattern",
                label: "Search pattern",
                param_type: ParamType::Text { default: "" },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Dmesg Errors",
            category: ActionCategory::Logs,
            description: "Show recent kernel errors and warnings",
            command_template: "dmesg --level=err,warn | tail -30",
            params: vec![],
            confirm_message: None,
        },
        // === Maintenance ===
        QuickAction {
            name: "Check Updates",
            category: ActionCategory::Maintenance,
            description: "Check for available package updates",
            command_template: "apt list --upgradable 2>/dev/null || yum check-update 2>/dev/null || dnf check-update 2>/dev/null",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Reboot Server",
            category: ActionCategory::Maintenance,
            description: "Reboot the server immediately",
            command_template: "sudo reboot",
            params: vec![],
            confirm_message: Some("This will REBOOT the server. Are you sure?"),
        },
        // === Docker ===
        QuickAction {
            name: "List Running Containers",
            category: ActionCategory::Docker,
            description: "Show all running Docker containers",
            command_template: "docker ps --format 'table {{.Names}}\\t{{.Status}}\\t{{.Ports}}'",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Container Logs",
            category: ActionCategory::Docker,
            description: "View recent logs of a Docker container",
            command_template: "docker logs --tail {lines} {container}",
            params: vec![
                ActionParam {
                    key: "container",
                    label: "Container",
                    param_type: ParamType::Select {
                        fetch_command: "docker ps --format '{{.Names}}'",
                    },
                },
                ActionParam {
                    key: "lines",
                    label: "Lines",
                    param_type: ParamType::Text { default: "100" },
                },
            ],
            confirm_message: None,
        },
        QuickAction {
            name: "Restart Container",
            category: ActionCategory::Docker,
            description: "Restart a running Docker container",
            command_template: "docker restart {container}",
            params: vec![ActionParam {
                key: "container",
                label: "Container",
                param_type: ParamType::Select {
                    fetch_command: "docker ps --format '{{.Names}}'",
                },
            }],
            confirm_message: None,
        },
        QuickAction {
            name: "Docker Disk Usage",
            category: ActionCategory::Docker,
            description: "Show Docker disk space usage",
            command_template: "docker system df",
            params: vec![],
            confirm_message: None,
        },
        QuickAction {
            name: "Stop All Containers",
            category: ActionCategory::Docker,
            description: "Stop all running Docker containers",
            command_template: "docker stop $(docker ps -q)",
            params: vec![],
            confirm_message: Some("This will STOP ALL running containers. Are you sure?"),
        },
        QuickAction {
            name: "Prune Unused Images",
            category: ActionCategory::Docker,
            description: "Remove all unused Docker images to free space",
            command_template: "docker image prune -af",
            params: vec![],
            confirm_message: Some("This will remove ALL unused Docker images. Are you sure?"),
        },
    ]
}
