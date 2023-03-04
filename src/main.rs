use std::{io, process};

mod args;
mod item;
mod processor;

use args::Args;

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> io::Result<()> {
    let context = args.as_processing_context();
    args.items().try_for_each(|item| item.process(&context))
}
