use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::env;


struct Cell {
    value: u8,
    candidates: Vec<bool>,

    blk_id: usize,
}

impl Cell {
    fn new(row: usize, col: usize) -> Cell {
        Cell {
            value: 0,
            candidates: vec![true;9],

            blk_id: 3*row+col,
        }
    }
}

struct SudokuGrid {
    data: Vec< Vec<Cell> >,

    row_counters: Vec< Vec<u8> >,
    col_counters: Vec< Vec<u8> >,
    blk_counters: Vec< Vec<u8> >,
}

// Initialize containers
impl SudokuGrid {
    fn new()->SudokuGrid {
        let mut init1: Vec< Vec<Cell> > = Vec::new();
        let mut i_row1: Vec< Vec<u8> > = Vec::new();
        let mut i_col1: Vec< Vec<u8> > = Vec::new();
        let mut i_blk1: Vec< Vec<u8> > = Vec::new();
        for i in 0..9 {
            let mut init2: Vec<Cell> = Vec::new();
            let i_row2: Vec<u8> = vec![9;9];
            let i_col2: Vec<u8> = vec![9;9];
            let i_blk2: Vec<u8> = vec![9;9];
            for j in 0..9 {
                init2.push( Cell::new(i/3,j/3) );
            }
            init1.push( init2 );
            i_row1.push( i_row2 );
            i_col1.push( i_col2 );
            i_blk1.push( i_blk2 );
        }
        SudokuGrid {
            data: init1,
            row_counters: i_row1,
            col_counters: i_col1,
            blk_counters: i_blk1,
        }
    }

    fn set_val(&mut self, row: usize, col: usize, val: u8) {
        // Set value and check block id and value id
        self.data[row][col].value = val;
        let blk_loc = self.data[row][col].blk_id;
        let val_loc: usize = val as usize - 1;

        // Substract one from all counters, if the set value was a candidate
        for i in 0..9 {
            if self.data[row][col].candidates[i] {
                self.row_counters[row][i] -= 1;
                self.col_counters[col][i] -= 1;
                self.blk_counters[blk_loc][i] -= 1;
            }
            self.data[row][col].candidates[i] = false;
        }

        // Substract the effect of the current value from rows, cols and blocks
        let mut blk_row = self.data[row][col].blk_id/3;
        let mut blk_col = self.data[row][col].blk_id%3;
        blk_row = blk_row*3;
        blk_col = blk_col*3;
        for i in 0..9 {
            if self.data[row][i].candidates[val_loc] {
                self.row_counters[row][val_loc] -= 1;
                self.col_counters[i][val_loc] -= 1;
                self.blk_counters[self.data[row][i].blk_id][val_loc] -= 1;
                self.data[row][i].candidates[val_loc] = false;
            }
            if self.data[i][col].candidates[val_loc] {
                self.row_counters[i][val_loc] -= 1;
                self.col_counters[col][val_loc] -= 1;
                self.blk_counters[self.data[i][col].blk_id][val_loc] -= 1;
                self.data[i][col].candidates[val_loc] = false;
            }
            let irow = i/3;
            let icol = i%3;
            if self.data[blk_row+irow][blk_col+icol].candidates[val_loc] {
                self.row_counters[blk_row+irow][val_loc] -= 1;
                self.col_counters[blk_col+icol][val_loc] -= 1;
                self.blk_counters[self.data[row][col].blk_id][val_loc] -= 1;
                self.data[blk_row+irow][blk_col+icol].candidates[val_loc] = false;
            }
        }
    }

    //fn col_exclusion(&mut self) {
    //    for i in 0..81 {
    //        if self.col_counters[i]==1 {
    //            let num = i%9;
    //            let col = i/9;
    //            for j in 0..9 {
    //                if self.data[j][col].candidates[num] {
    //                    //self.data[j][col].value = num as u32 + 1;
    //                }
    //            }
    //            self.col_counters[i]=0;
    //        }
    //    }
    //}

    //fn blk_exclusion(&mut self) {
    //    for blk_r in 0..3 {
    //    for blk_c in 0..3 {
    //        let c_loc = 9*( 3*blk_r + blk_c );
    //        for num in 0..9 {
    //            if self.blk_counters[c_loc+num]==1 {
    //                for row in 0..3 {
    //                for col in 0..3 {
    //                    if self.data[blk_r*3+row][blk_c*3+col].candidates[num] {
    //                        self.data[blk_r*3+row][blk_c*3+col].value = num as u32 + 1;
    //                    }
    //                }
    //                }
    //            }
    //        }
    //    }
    //    }
    //}

    //fn check_col(&mut self, row: usize, col: usize) {
    //    for i in 0..9 {
    //        let loc = col*9+i;
    //        if self.col_counters[loc] != 0 {
    //            self.col_counters[col*9+i] -= 1;
    //        }
    //    }
    //    let type_loc: usize = self.data[row][col].value as usize - 1;
    //    self.col_counters[col*9+type_loc]=0;

    //    let val: usize = self.data[row][col].value as usize;
    //    for i in 0..9 {
    //        if self.data[i][col].value==0 {
    //            if self.data[i][col].candidates[val-1]==true {
    //                let blk_r = i/3;
    //                let blk_c = col/3;
    //                if self.blk_counters[9*(3*blk_r+blk_c)+i] != 0 {
    //                    self.blk_counters[9*(3*blk_r+blk_c)+i] -= 1;
    //                }
    //            }
    //            self.data[i][col].candidates[val-1]=false;
    //        }
    //    }
    //}

    //fn check_row(&mut self, row: usize, col: usize) {
    //    for i in 0..9 {
    //        let loc = row*9+i;
    //        if self.row_counters[loc] != 0 {
    //            self.row_counters[row*9+i] -= 1;
    //        }
    //    }
    //    let type_loc: usize = self.data[row][col].value as usize - 1;
    //    self.row_counters[row*9+type_loc]=0;
    //    
    //    let val: usize = self.data[row][col].value as usize;
    //    for i in 0..9 {
    //        if self.data[row][i].value==0 {
    //            self.data[row][i].candidates[val-1]=false;
    //        }
    //    }
    //}

    //fn check_blk(&mut self, row: usize, col: usize) {
    //    let blk_r = row/3;
    //    let blk_c = col/3;

    //    let loc = 9*(3*blk_r+blk_c);
    //    for i in 0..9 {
    //        if self.blk_counters[loc+i] != 0 {
    //            self.blk_counters[loc+i] -= 1;
    //        }
    //    }
    //    let type_loc: usize = self.data[row][col].value as usize - 1;
    //    self.blk_counters[loc+type_loc]=0;

    //    let val: usize = self.data[row][col].value as usize;
    //    for row in 0..3 {
    //    for col in 0..3 {
    //        if self.data[3*blk_r+row][3*blk_c+col].value==0 {
    //            if self.data[3*blk_r+row][3*blk_c+col].candidates[val-1]==true {
    //                self.blk_counters[loc+val-1];
    //            }
    //            self.data[3*blk_r+row][3*blk_c+col].candidates[val-1]=false;
    //        }
    //    }
    //    }
    //}


    //fn cand_amount(&mut self, pos: usize) {

    //}

    //fn block_exclusion() {
    //    for r_block in 0..3
    //    for c_block in 0..3
    //        let mut counter = vec![0; 9];
    //        for row in 0..3
    //        for col in 0..3
    //            let loc = (3*r_block+row)*9+3*c_block+col;
    //            for 
    //        }
    //        }
    //    }
    //    }
    //}

    fn update(&mut self) {
        //for i in 0..9 {
        //    for j in 0..9 {
        //        if self.data[i][j].value!=0 && !self.data[i][j].stabilized {
        //            for checker in 0..9 {
        //                self.data[i][j].candidates[checker] = false;
        //            }
                    //self.check_col(i,j);
                    //self.check_blk(i,j);
                    //self.check_row(i);
                    //self.check_block(i);
                    //self.col_exclusion();
                    //self.blk_exclusion();
                    //self.block_exclusion();
                    //self.data[i].stabilized = true;
        //        }
        //    }
        //}

        //for i in 0..9 {
        //    //print!("{} ",self.blk_counters[9+i]);
        //}
    }

    fn print(&self) {
        print!( "\n" );
        for i in 0..9 {
            for j in 0..9 {
                let mut val = 0;
                if self.data[i][j].candidates[0] { val = 1; }
                print!( "{},{} ", self.data[i][j].value, val);
                if (j+1)%3==0 {
                    print!( " " );
                }
            }
            print!("\n");

            if (i+1)%3==0 {
                print!("\n");
            }
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

    let mut sudokus : Vec<SudokuGrid> = Vec::new();
    reader( &lines, read_amount, &mut sudokus );

    sudokus[0].print();
    sudokus[0].update();
    sudokus[0].print();
    sudokus[0].update();
    sudokus[0].print();
}

fn reader( data: &str, read_amount: usize, sudokus: &mut Vec<SudokuGrid> ) {
    let mut sudoku_num = 1;
    let mut row = 0;
    let mut col = 0;

    for sudoku_char in data.chars() {
        if sudoku_char == '\n' {
            if row == 0 {
                sudokus.push( SudokuGrid::new() );
            }
            row += 1;
            col = 0;
        } else if row != 0 {
            let num = sudoku_char.to_digit(10).unwrap();
            if num != 0 {
                sudokus[sudoku_num-1].set_val(row-1,col, num as u8);
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
