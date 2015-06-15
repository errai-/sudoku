use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::env;

pub use indices::Indices;
pub use sudokucell::SudokuCell;
pub use sudokugrid::SudokuGrid;
pub mod indices;
pub mod sudokucell;
pub mod sudokugrid;


// Call update for a sudoku until it is finished.
// If an unsuccessful update is encountered, dualism is called (see above)
fn sudoku_loop(sudoku : &mut SudokuGrid, depth: usize, indices : &Indices) -> bool {
    while !sudoku.is_complete(false) {
        if !sudoku.update() {
            if depth<1 { return false; }
            let dual = sudoku.dualism();
            if dual.len()!=6 { 
                println!("Failure1!");
                return false; 
            }
            let mut probe_sudoku1 : SudokuGrid = sudoku.clone();
            probe_sudoku1.set_cell( dual[2], dual[3], dual[0] as u8 + 1);
            probe_sudoku1.set_cell( dual[4], dual[5], dual[1] as u8 + 1);
            if sudoku_loop( &mut probe_sudoku1, depth-1, indices ) {
                *sudoku = probe_sudoku1.clone();
            } else {
                let mut probe_sudoku2 : SudokuGrid = sudoku.clone();
                probe_sudoku2.set_cell( dual[2], dual[3], dual[1] as u8 + 1);
                probe_sudoku2.set_cell( dual[4], dual[5], dual[0] as u8 + 1);
                if sudoku_loop( &mut probe_sudoku2, depth-1, indices ) {
                    *sudoku = probe_sudoku2.clone();
                } else {
                    println!("Failure2!");
                    return false;
                }
            }
            break;
        }
    }
    if !sudoku.is_complete(true) {
        println!("Failure3");
        return false;
    }
    true
}

// Read in the text data.
// The text file always has first an arbitrary row and then nine sudoku rows
// Containing the initial setup.
fn reader<'a>( data: &str, read_amount: usize, sudokus: &mut Vec<SudokuGrid<'a> >, indices : &'a Indices ) {
    let mut sudoku_num = 1;
    let mut row = 0;
    let mut col = 0;

    for sudoku_char in data.chars() {
        if sudoku_char == '\n' {
            if row == 0 {
                sudokus.push( SudokuGrid::new(indices) );
            }
            row += 1;
            col = 0;
        } else if row != 0 {
            let num = sudoku_char.to_digit(10).unwrap();
            if num != 0 {
                sudokus[sudoku_num-1].set_cell(row-1,col, num as u8);
            }
            col += 1;
        }

        if row > 9 {
            row = 0;
            sudoku_num += 1;
            if sudoku_num > read_amount { break; }
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    // Try to obtain input file name from command line params.
    if args.len() < 2 {
        println!("Insert sudoku data file name as a command line parameter");
        return; 
    }
    let path = Path::new(&args[1]);
    let display = path.display();

    // Check how many sudoku entries are to be read from command line param.s
    let read_amount : usize = if args.len() > 2 {
        match args[2].parse::<usize>() {
            Err(why) => panic!("not a number amount: {}", Error::description(&why)),
            Ok(read_amount) => read_amount,
        }
    } else {
        10
    };

    // Open file and read it to a string
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };
    let mut lines = String::new();
    match file.read_to_string(&mut lines) {
        Err(why) => panic!("could not read {}: {}", display, Error::description(&why)),
        Ok(_) => {},
    }

    let indices : Indices = Indices::new();
    
    let mut sudokus : Vec<SudokuGrid> = Vec::new();
    reader( &lines, read_amount, &mut sudokus, &indices );

    let mut success_count = 0;
    let mut corner_sum = 0;
    for sudoku_idx in 0..sudokus.len() {
        let success = sudoku_loop( &mut sudokus[sudoku_idx],1,&indices);
        if !success {
            println!("{}",sudoku_idx);
            sudokus[sudoku_idx].print();
        } else {
            success_count += 1;
            corner_sum += sudokus[sudoku_idx].corner_val();
        }
    }
    print!("Success count: {}/{}\n",success_count,sudokus.len());
    print!("Corner sum: {}\n",corner_sum);
}
