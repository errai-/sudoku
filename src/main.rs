use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 { 
        println!("Insert sudoku data file name as a command line parameter");
        return; 
    }

    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let mut lines = String::new();
    match file.read_to_string(&mut lines) {
        Err(why) => panic!("could not read {}: {}", display, Error::description(&why)),
        Ok(_) => print!("{} contains:\n{}", display, lines),
    }
}
