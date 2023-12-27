#![feature(exit_status_error)]

mod command;
mod runner;

use std::env::{self, VarError};
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{bail, Context};
use color_eyre::Result;

use crate::runner::Runner;

#[derive(Parser, Debug)]
struct RunnerArgs {
    #[command(subcommand)]
    command: RunnerCommand,
    #[command(flatten)]
    target: TargetArgs,
    #[command(flatten)]
    binaries: BinaryArgs,
}

#[derive(Subcommand, Debug)]
enum RunnerCommand {
    /// Build the kernel binary.
    Build,
    /// Run tests for platform-independent packages.
    Test,
    /// Remove build artifacts.
    Clean,
    /// Build the kernel binary, then run the kernel in QEMU.
    Qemu {
        /// Should QEMU open a GDB server?
        #[arg(long, short)]
        debugger: bool,
    },
    /// Run GDB, configured to attach to QEMU.
    Gdb,
}

#[derive(Debug)]
enum Target {
    Debug,
    Release,
}

impl Target {
    fn cargo_profile_flag(&self) -> &'static str {
        match self {
            // Cargo does not accept --debug, nor --dev
            Self::Debug => "",
            Self::Release => "--release",
        }
    }

    fn cargo_profile_dir(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Target")]
struct TargetArgs {
    /// Use a debug build (default).
    #[arg(long, global = true)]
    debug: bool,
    /// Use a release build.
    #[arg(long, global = true)]
    release: bool,
}

impl TargetArgs {
    fn as_target(&self) -> Result<Target> {
        let Self { debug, release } = *self;
        if debug && release {
            // TODO: encode this through clap
            bail!("can't specify both debug and release as target");
        } else if release {
            Ok(Target::Release)
        } else {
            // Default for all other cases (debug specified or no flags specified)
            Ok(Target::Debug)
        }
    }
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Binaries")]
struct BinaryArgs {
    /// Path to a GDB which supports aarch64. [default: $GDB, otherwise `gdb`]
    #[arg(long, global = true)]
    gdb: Option<PathBuf>,
}

impl BinaryArgs {
    fn resolve(
        arg: Option<PathBuf>,
        name: &str,
        default_path: impl Into<PathBuf>,
    ) -> Result<PathBuf> {
        if let Some(path) = arg {
            Ok(path)
        } else {
            match env::var(name) {
                Ok(path) => Ok(PathBuf::from(path)),
                Err(VarError::NotPresent) => Ok(default_path.into()),
                Err(err) => Err(err)
                    .wrap_err_with(|| format!("failed to read environment varaible ${name}")),
            }
        }
    }

    fn into_binaries(self) -> Result<Binaries> {
        Ok(Binaries {
            gdb: Self::resolve(self.gdb, "GDB", "gdb")?,
        })
    }
}

#[derive(Debug)]
struct Binaries {
    gdb: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let RunnerArgs {
        command,
        target,
        binaries,
    } = RunnerArgs::parse();

    let target = target.as_target()?;
    let binaries = binaries.into_binaries()?;
    let kernel = Path::new("target/aarch64-unknown-none")
        .join(target.cargo_profile_dir())
        .join("kernel");

    let runner = Runner::new(binaries);

    let build = || -> Result<()> {
        runner.step("build");
        runner.run(
            command::make("build")
                .directory("kernel/")
                .variable("CARGOFLAGS", target.cargo_profile_flag()),
        )?;

        Ok(())
    };

    let test = || -> Result<()> {
        let mut flags = vec![target.cargo_profile_flag()];
        for package in ["buddy-alloc"] {
            flags.push("-p");
            flags.push(package);
        }

        runner.step("test");
        runner.run(
            command::make("test")
                .directory("kernel/")
                .variable("CARGOFLAGS", flags.join(" ")),
        )?;

        Ok(())
    };

    let clean = || -> Result<()> {
        runner.step("clean");
        runner.run(command::make("clean").directory("kernel/"))?;

        Ok(())
    };

    let qemu = |debugger| -> Result<()> {
        let qemuflags = if debugger { "-S -s" } else { "" };
        let kernel = Path::new("..").join(&kernel);

        runner.step("qemu");
        runner.exec(
            command::make("run-kernel")
                .directory("qemu/")
                .variable("QEMUFLAGS", qemuflags)
                .variable("KERNEL", kernel.to_str().unwrap()),
        )?;

        Ok(())
    };

    let gdb = || -> Result<()> {
        runner.step("gdb");
        runner.exec(
            command::gdb(kernel.to_str().unwrap())
                .arg("-ex")
                .arg("target remote localhost:1234"),
        )?;

        Ok(())
    };

    match command {
        RunnerCommand::Build => build(),
        RunnerCommand::Test => test(),
        RunnerCommand::Clean => clean(),
        RunnerCommand::Qemu { debugger } => build().and_then(|_| qemu(debugger)),
        RunnerCommand::Gdb => gdb(),
    }?;

    runner.done();
    Ok(())
}
