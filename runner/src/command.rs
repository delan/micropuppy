use std::process::Command;

use color_eyre::Result;

use crate::runner::IntoCommand;
use crate::Binaries;

pub struct Make {
    target: String,
    directory: Option<String>,
    variables: Vec<(String, String)>,
}

impl IntoCommand for &mut Make {
    fn into_command(self, _binaries: &Binaries) -> Result<Command> {
        // We're forced away from the full builder syntax because we need to return the owned
        // Command, not the &mut Command that the builder methods return.
        let mut command = Command::new("make");
        command.arg(&self.target);

        if let Some(directory) = &self.directory {
            command.args(["-C", directory]);
        }

        for (name, value) in &self.variables {
            command.arg(format!("{name}={value}"));
        }

        Ok(command)
    }
}

pub fn make(target: impl Into<String>) -> Make {
    Make {
        target: target.into().to_string(),
        directory: None,
        variables: vec![],
    }
}

impl Make {
    pub fn directory(&mut self, directory: impl Into<String>) -> &mut Self {
        self.directory = Some(directory.into());
        self
    }

    pub fn variable(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.push((name.into(), value.into()));
        self
    }
}

pub struct Gdb {
    binary: String,
    args: Vec<String>,
}

impl IntoCommand for &mut Gdb {
    fn into_command(self, binaries: &Binaries) -> Result<Command> {
        // We're forced away from the full builder syntax because we need to return the owned
        // Command, not the &mut Command that the builder methods return.
        let mut command = Command::new(&binaries.gdb);
        command.args(&self.args).arg(&self.binary);

        Ok(command)
    }
}

pub fn gdb(binary: impl Into<String>) -> Gdb {
    Gdb {
        binary: binary.into(),
        args: vec![],
    }
}

impl Gdb {
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }
}
