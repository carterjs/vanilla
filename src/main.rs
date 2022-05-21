use std::{env, fs};

use vanilla::{vm::{VM, self}};

fn main() -> Result<(), vm::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let source = fs::read_to_string(args.get(1).unwrap()).expect("Something went wrong reading the file");

        // Run the entry point
        let mut vm = VM::new();
        vm.interpret(source)?;
    } else {
        println!("Usage: vanilla <file>");
    }

    Ok(())
}