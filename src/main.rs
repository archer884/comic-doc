use std::{io, process};

mod args;
mod item;

use args::Args;

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> io::Result<()> {
    let context = args.as_processing_context();

    for item in args.items() {
        item.process(&context)?;
    }

    Ok(())
}
