use std::process::Child;

use crate::show::construct_position::Position;
use crate::tmux::Tmux;
use anyhow::{anyhow, Result};

static NEW_PANE: &str = "new-pane";

fn centered_position(size: &str, dimension: &str) -> Result<String> {
    if let Some(percent) = size.strip_suffix('%') {
        let percent = percent
            .parse::<u16>()
            .map_err(|_| anyhow!("invalid pane size: {size}"))?;
        if !(1..=100).contains(&percent) {
            return Err(anyhow!("pane percentage must be between 1 and 100: {size}"));
        }
        return Ok(format!("{}%", (100 - percent) / 2));
    }

    let cells = size
        .parse::<u16>()
        .map_err(|_| anyhow!("invalid pane size: {size}"))?;
    if cells == 0 {
        return Err(anyhow!("pane size must be greater than zero: {size}"));
    }
    Ok(format!("#{{e|/:#{{e|-:#{{{dimension}}},{size}}},2}}"))
}

fn edge_position(size: &str, dimension: &str) -> Result<String> {
    if let Some(percent) = size.strip_suffix('%') {
        let percent = percent
            .parse::<u16>()
            .map_err(|_| anyhow!("invalid pane size: {size}"))?;
        if !(1..=100).contains(&percent) {
            return Err(anyhow!("pane percentage must be between 1 and 100: {size}"));
        }
        return Ok(format!("{}%", 100 - percent));
    }

    size.parse::<u16>()
        .map_err(|_| anyhow!("invalid pane size: {size}"))?;
    Ok(format!("#{{e|-:#{{{dimension}}},{size}}}"))
}

impl Tmux {
    fn construct_pane_arguments(position: &Position) -> Result<Vec<String>> {
        let width = position
            .w
            .as_deref()
            .ok_or_else(|| anyhow!("pane width is required"))?;
        let height = position
            .h
            .as_deref()
            .ok_or_else(|| anyhow!("pane height is required"))?;

        Ok(vec![
            "-x".to_string(),
            width.to_string(),
            "-y".to_string(),
            height.to_string(),
            "-X".to_string(),
            match position.x.as_str() {
                "C" => centered_position(width, "window_width")?,
                "R" => edge_position(width, "window_width")?,
                x => x.to_string(),
            },
            "-Y".to_string(),
            match position.y.as_str() {
                "C" => centered_position(height, "window_height")?,
                "P" => edge_position(height, "window_height")?,
                y => y.to_string(),
            },
        ])
    }

    pub fn new_pane(
        &self,
        command: String,
        position: &Position,
        close_after_command: bool,
    ) -> Result<Child> {
        let mut arguments = vec![NEW_PANE.to_string()];
        arguments.append(&mut Self::construct_pane_arguments(position)?);
        if !close_after_command {
            arguments.push("-k".to_string());
        }
        arguments.push(command);
        self._run(arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::{centered_position, edge_position, Tmux};
    use crate::show::construct_position::Position;

    #[test]
    fn pane_geometry_uses_popup_position() {
        assert_eq!(centered_position("60%", "window_width").unwrap(), "20%");
        assert_eq!(edge_position("60%", "window_width").unwrap(), "40%");
        assert_eq!(
            centered_position("40", "window_width").unwrap(),
            "#{e|/:#{e|-:#{window_width},40},2}"
        );

        let arguments = Tmux::construct_pane_arguments(&Position {
            x: "R".to_string(),
            y: "P".to_string(),
            w: Some("60%".to_string()),
            h: Some("70%".to_string()),
        })
        .unwrap();

        assert_eq!(
            arguments,
            ["-x", "60%", "-y", "70%", "-X", "40%", "-Y", "30%"]
        );

        let arguments = Tmux::construct_pane_arguments(&Position {
            x: "12".to_string(),
            y: "3".to_string(),
            w: Some("40".to_string()),
            h: Some("20".to_string()),
        })
        .unwrap();

        assert_eq!(arguments, ["-x", "40", "-y", "20", "-X", "12", "-Y", "3"]);
    }
}
