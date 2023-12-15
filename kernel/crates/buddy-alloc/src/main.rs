#![cfg(feature = "cli")]
use std::io::{self, Write};
use std::{env, fs};

use buddy_alloc::tree::Tree;

enum Command<'l> {
    One(&'l str),
    Two(&'l str, &'l str),
}

enum Action {
    Continue,
    Quit,
}

fn main() {
    let args = env::args();
    let depth = args
        .skip(1)
        .next()
        .ok_or("expected tree depth as first command line argument")
        .and_then(|depth| depth.parse().map_err(|_| "could not parse depth"));

    let depth = match depth {
        Ok(depth) => depth,
        Err(e) => {
            println!("error: {e}");
            return;
        }
    };

    // 64 bytes should be enough for anyone
    let mut storage = [0; 64];
    let mut tree = Tree::new(&mut storage, depth);

    loop {
        print!("> ");
        io::stdout()
            .flush()
            .expect("flushing stdout should succeed");

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("read_line should succeed");

        let line = line.trim();
        let command = match line.split_once(" ") {
            None => Command::One(&line),
            Some((command, arg)) => Command::Two(command, arg),
        };

        match run_command(command, &mut tree) {
            Ok(Action::Continue) => {}
            Ok(Action::Quit) => break,
            Err(e) => println!("error: {e}"),
        }
    }
}

fn run_command(command: Command, tree: &mut Tree) -> Result<Action, &'static str> {
    let dot_path = env::temp_dir().join("buddy-alloc.dot");

    match command {
        Command::One("help") => {
            println!("commands:");
            println!("  exit|quit|q");
            println!("  show");
            println!("  malloc <size in blocks>");
            println!("  free <offset>");
        }
        Command::One("exit" | "quit" | "q") => return Ok(Action::Quit),
        Command::One("show") => {
            opener::open(&dot_path).map_err(|_| "could not open dot file")?;

            println!("opened {} in system dot viewer", dot_path.display());
        }
        Command::Two("malloc", size) => {
            let size = size.parse().map_err(|_| "could not parse size")?;
            let allocation = tree.allocate(size).map_err(|_| "out of memory")?;

            println!(
                "allocated {} block{} (requested {}) at offset {}",
                allocation.size,
                if size != 1 { "s" } else { "" },
                size,
                allocation.offset
            );
        }
        Command::Two("free", offset) => {
            let offset = offset.parse().map_err(|_| "could not parse offset")?;
            tree.free(offset).map_err(|_| "double free")?;

            println!("freed allocation at offset {}", offset);
        }
        _ => return Err("unknown command"),
    };

    let dot = format!("{}", tree.dot());
    fs::write(&dot_path, dot).map_err(|_| "could not write dot file")?;

    Ok(Action::Continue)
}
