#![feature(exit_status_error)]

mod action;
mod command;

use std::path::Path;

use color_eyre::eyre::bail;
use color_eyre::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    command: RunnerCommand,
    #[structopt(flatten)]
    target: TargetOpts,
}

#[derive(Debug, StructOpt)]
enum RunnerCommand {
    /// Build the kernel binary.
    Build,
    /// Remove build artifacts.
    Clean,
    /// Build the kernel binary, then run the kernel in QEMU.
    Qemu {
        /// Should QEMU open a GDB server?
        #[structopt(long, short)]
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

#[derive(Debug, StructOpt)]
struct TargetOpts {
    /// Use a debug build (default).
    #[structopt(long, global = true)]
    debug: bool,
    /// Use a release build.
    #[structopt(long, global = true)]
    release: bool,
}

impl TargetOpts {
    fn as_target(&self) -> Result<Target> {
        let Self { debug, release } = *self;
        if debug && release {
            bail!("can't specify both debug and release as target");
        } else if release {
            Ok(Target::Release)
        } else {
            // Default for all other cases (debug specified or no flags specified)
            Ok(Target::Debug)
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let Opt { target, command } = Opt::from_args();
    let target = target.as_target()?;
    let kernel = Path::new("target/aarch64-unknown-none")
        .join(target.cargo_profile_dir())
        .join("kernel");

    let build = || -> Result<()> {
        action::step("build");
        action::invoke(command::make("build").directory("a53/"))?;
        action::invoke(
            command::make("build")
                .directory("kernel/")
                .variable("CARGOFLAGS", target.cargo_profile_flag()),
        )?;

        Ok(())
    };

    let clean = || -> Result<()> {
        action::step("clean");
        action::invoke(command::make("clean").directory("a53/"))?;
        action::invoke(command::make("clean").directory("kernel/"))?;

        Ok(())
    };

    let qemu = |debugger| -> Result<()> {
        let qemuflags = if debugger { "-S -s" } else { "" };
        let kernel = Path::new("..").join(&kernel);

        action::step("qemu");
        action::invoke(
            command::make("run-kernel")
                .directory("qemu/")
                .variable("QEMUFLAGS", qemuflags)
                .variable("KERNEL", kernel.to_str().unwrap()),
        )?;

        Ok(())
    };

    let gdb = || -> Result<()> {
        action::step("gdb");
        action::invoke(
            command::gdb(kernel.to_str().unwrap())
                .arg("-ex")
                .arg("target remote localhost:1234"),
        )?;

        Ok(())
    };

    match command {
        RunnerCommand::Build => build(),
        RunnerCommand::Clean => clean(),
        RunnerCommand::Qemu { debugger } => build().and_then(|_| qemu(debugger)),
        RunnerCommand::Gdb => gdb(),
    }?;

    action::done();
    Ok(())
}
