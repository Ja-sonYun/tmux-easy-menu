use anyhow::Result;
use std::process::{Child, Command};

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn shell_join(args: &[String]) -> Result<String> {
    let mut quoted = Vec::with_capacity(args.len());
    for arg in args {
        let arg = shlex::try_quote(arg)
            .map_err(|err| anyhow::anyhow!("failed to quote shell argument: {err:?}"))?;
        quoted.push(arg.into_owned());
    }
    Ok(quoted.join(" "))
}

pub fn run_command(command: String) -> Result<String> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn exec_shell(command: String) -> Result<()> {
    use std::os::unix::process::CommandExt;

    Err(Command::new("sh").arg("-c").arg(command).exec().into())
}

pub fn spawn_binary(command: String, args: Vec<String>) -> Result<Child> {
    let mut output = Command::new(command).args(args).spawn()?;
    output.wait()?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::shell_join;

    #[test]
    fn shell_join_quotes_generated_arguments() {
        let args = vec![
            "tmux-menu".to_string(),
            "select".to_string(),
            "--menu".to_string(),
            "a path/with spaces/menu.yaml".to_string(),
            "semi;pipe|hash#quote'".to_string(),
        ];

        let command = shell_join(&args).unwrap();

        assert_eq!(shlex::split(&command).unwrap(), args);
    }
}
