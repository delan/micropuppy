use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use owo_colors::OwoColorize;

trait ToStrOrError {
    fn to_str_or_err(&self) -> Result<&str>;
}

impl ToStrOrError for OsStr {
    fn to_str_or_err(&self) -> Result<&str> {
        self.to_str()
            .ok_or_else(|| eyre!("OsStr should be valid utf-8"))
    }
}

impl ToStrOrError for Path {
    fn to_str_or_err(&self) -> Result<&str> {
        self.to_str()
            .ok_or_else(|| eyre!("Path should be valid utf-8"))
    }
}

pub fn step(name: &str) {
    eprintln!("{}", format!("🧾 running step `{name}`").bold());
}

pub trait IntoCommand {
    fn into_command(self) -> Result<Command>;
}

pub fn invoke(command: impl IntoCommand) -> Result<()> {
    let mut command = command.into_command()?;

    let program = command.get_program();
    let args = command.get_args();
    let command_line = std::iter::once(program)
        .chain(args)
        .map(|s| s.to_str_or_err())
        .map(|s| {
            s.map(|s| {
                // If an argument contains any whitespace, surround it with quotes. This is just a
                // human-readable string, so it doesn't need to be perfectly correct with respect to
                // shell parsing.
                if s.contains(char::is_whitespace) {
                    format!("\"{}\"", s.replace('\"', "\\\""))
                } else {
                    s.to_string()
                }
            })
        })
        .collect::<Result<Vec<_>>>()?
        .join(" ");

    if let Some(current_dir) = command.get_current_dir() {
        let current_dir = current_dir.to_str_or_err()?;
        eprintln!("⭐ invoking `{}` in {}", command_line, current_dir);
    } else {
        eprintln!("⭐ invoking `{}`", command_line);
    }

    command.spawn()?.wait()?.exit_ok()?;

    Ok(())
}

pub fn done() {
    eprintln!("{}", "💜 done!".bold());
}
