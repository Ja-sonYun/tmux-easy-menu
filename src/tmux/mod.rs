pub mod positions;
pub mod subcommand;

use std::process::{Child, Command};

use crate::shell::spawn_binary;
use crate::show::construct_position::Position;
use anyhow::Result;

pub struct Tmux {
    binary: String,
}

pub struct PopupParent {
    source_client: String,
    client: String,
    pane: String,
    pub working_dir: String,
}

impl Tmux {
    fn _run(&self, arguments: Vec<String>) -> Result<Child> {
        spawn_binary(self.binary.clone(), arguments)
    }

    fn output(&self, arguments: &[&str]) -> Result<String> {
        let output = Command::new(&self.binary).args(arguments).output()?;
        anyhow::ensure!(
            output.status.success(),
            "tmux {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim_end()
        );
        Ok(String::from_utf8(output.stdout)?
            .trim_end_matches('\n')
            .to_string())
    }

    pub fn popup_parent(&self) -> Result<Option<PopupParent>> {
        let target = Self::construct_target_arguments();
        let mut arguments = vec!["display-message", "-p"];
        arguments.extend(target.iter().map(String::as_str));
        arguments.push("#{session_name}\n#{pane_id}\n#{window_id}\n#{client_name}");
        let source = self.output(&arguments)?;
        let mut fields = source.splitn(4, '\n');
        let session = fields.next().unwrap_or_default();
        let pane = fields.next().unwrap_or_default();
        let window = fields.next().unwrap_or_default();
        let client = std::env::var("TMUX_MENU_CLIENT")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| fields.next().unwrap_or_default().to_string());
        for (name, value) in [
            ("TMUX_MENU_ORIGIN_PANE", pane),
            ("TMUX_MENU_ORIGIN_WINDOW", window),
        ] {
            if std::env::var(name).unwrap_or_default().is_empty() {
                std::env::set_var(name, value);
            }
        }

        let key = crate::popup_key(session);
        let owner = self.output(&["show-options", "-gqv", &format!("@tmux_menu_popup_{key}")])?;
        if owner != "1" {
            return Ok(None);
        }
        let outer = self.output(&["show-options", "-gqv", &format!("@popup_client_{key}")])?;
        if outer.is_empty() || outer == client || client.is_empty() {
            return Ok(None);
        }
        let clients = self.output(&["list-clients", "-F", "#{client_name}\t#{pane_id}"])?;
        let Some(outer_pane) = clients.lines().find_map(|line| {
            let (name, pane) = line.split_once('\t')?;
            (name == outer && !pane.is_empty()).then_some(pane)
        }) else {
            return Ok(None);
        };
        let working_dir = self.output(&[
            "display-message",
            "-p",
            "-t",
            outer_pane,
            "#{pane_current_path}",
        ])?;
        anyhow::ensure!(!working_dir.is_empty(), "Parent working directory is empty");
        Ok(Some(PopupParent {
            source_client: client,
            client: outer,
            pane: outer_pane.to_string(),
            working_dir,
        }))
    }

    pub fn leave_popup(&self, parent: &PopupParent) -> Result<()> {
        self.output(&["detach-client", "-t", &parent.source_client])?;
        std::env::set_var("TMUX_MENU_CLIENT", &parent.client);
        std::env::set_var("TMUX_MENU_PANE", &parent.pane);
        Ok(())
    }

    fn construct_border_arguments(border: &str) -> Vec<String> {
        vec!["-b".to_string(), border.to_string()]
    }

    fn construct_client_arguments() -> Vec<String> {
        std::env::var("TMUX_MENU_CLIENT")
            .ok()
            .filter(|value| !value.is_empty())
            .map(|client| vec!["-c".to_string(), client])
            .unwrap_or_default()
    }

    fn construct_target_arguments() -> Vec<String> {
        std::env::var("TMUX_MENU_PANE")
            .ok()
            .filter(|value| !value.is_empty())
            .map(|pane| vec!["-t".to_string(), pane])
            .unwrap_or_default()
    }

    fn construct_position_arguments(position: &Position) -> Vec<String> {
        if let (Some(w), Some(h)) = (position.w.clone(), position.h.clone()) {
            vec![
                "-x".to_string(),
                position.x.to_string(),
                "-y".to_string(),
                position.y.to_string(),
                "-w".to_string(),
                w.to_string(),
                "-h".to_string(),
                h.to_string(),
            ]
        } else {
            vec![
                "-x".to_string(),
                position.x.to_string(),
                "-y".to_string(),
                position.y.to_string(),
            ]
        }
    }

    pub fn new() -> Self {
        Self {
            binary: "tmux".to_string(),
        }
    }
}
