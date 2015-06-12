use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::env;


struct Cell {
    value: u8,
    candidates: Vec<bool>,
    candidate_amnt: u8,

    blk_id: usize,
}

impl Cell {
    fn new(row: usize, col: usize) -> Cell {
        Cell {
            value: 0,
            candidates: vec![true;9],
            candidate_amnt: 9,

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
            self.data[row][col].candidate_amnt = 0;
        }

        // Substract the effect of the current value from rows, cols and blocks
        let mut blk_row = self.data[row][col].blk_id/3;
        let mut blk_col = self.data[row][col].blk_id%3;
        blk_row = blk_row*3;
        blk_col = blk_col*3;
        for i in 0..9 {
            self.flip_val(row,i,val_loc);
            self.flip_val(i,col,val_loc);
            let irow = i/3;
            let icol = i%3;
            self.flip_val(blk_row+irow,blk_col+icol,val_loc);
        }
    }

    fn flip_val(&mut self, row: usize, col: usize, val_loc: usize) -> bool {
        if self.data[row][col].candidates[val_loc] {
            self.row_counters[row][val_loc] -= 1;
            self.col_counters[col][val_loc] -= 1;
            self.blk_counters[self.data[row][col].blk_id][val_loc] -= 1;
            self.data[row][col].candidates[val_loc] = false;
            self.data[row][col].candidate_amnt -= 1;
            return true;
        }
        false
    }

    fn update(&mut self) -> bool {
        let mut sets = 0;
        for i in 0..9 {
            for val_loc in 0..9 {
                if self.row_counters[i][val_loc]==1 {
                    sets += 1;
                    for col in 0..9 {
                        if self.data[i][col].candidates[val_loc] {
                            self.set_val(i,col,val_loc as u8+1);
                            break;
                        }
                    }
                }

                if self.col_counters[i][val_loc]==1 {
                    sets += 1;
                    for row in 0..9 {
                        if self.data[row][i].candidates[val_loc] {
                            self.set_val(row,i,val_loc as u8+1);
                            break;
                        }
                    }
                }

                if self.blk_counters[i][val_loc]==1 {
                    sets += 1;
                    let mut blk_row = i/3;
                    let mut blk_col = i%3;
                    blk_row = blk_row*3;
                    blk_col = blk_col*3;
                    for rowcol in 0..9 {
                        let row = blk_row+rowcol/3;
                        let col = blk_col+rowcol%3;
                        if self.data[row][col].candidates[val_loc] {
                            self.set_val(row,col,val_loc as u8+1);
                            break;
                        }
                    }
                }
            }
        }

        if sets==0 {
            sets += self.singles();
        }

        if sets==0 {
            if !self.advanced_update() { return false; }
        }
        true
    }

    fn singles(&mut self) -> usize {
        let mut sets = 0;
        for row in 0..9 {
        for col in 0..9 {
            if self.data[row][col].candidate_amnt==0 { continue; }
            if self.data[row][col].candidate_amnt==1 {
                sets += 1;
                for val_loc in 0..9 {
                    if self.data[row][col].candidates[val_loc] {
                        self.set_val(row,col,val_loc as u8+1);
                    }
                }
            }
        }
        }
        sets
    }

    // Seek for blocks with only two candidates for a value and possibly eliminate rows/cols.
    fn advanced_update(&mut self) -> bool {
        let mut counter = 0;
        for blk in 0..9 {
        for val_loc in 0..9 {
            if self.blk_counters[blk][val_loc]==2 {
                let mut blk_row = blk/3;
                let mut blk_col = blk%3;
                blk_row = blk_row*3;
                blk_col = blk_col*3;

                let mut row_id = 9;
                let mut col_id = 9;
                for i in 0..9 {
                    let row = blk_row+i/3;
                    let col = blk_col+i%3;
                    if self.data[row][col].candidates[val_loc] {
                        if row_id == 9 {
                            row_id = row;
                            col_id = col;
                        } else {
                            if row==row_id {
                                for j in 0..blk_col {
                                    if self.flip_val(row,j,val_loc) { counter += 1; }
                                }
                                for j in blk_col+3..9 {
                                    if self.flip_val(row,j,val_loc) { counter += 1; }
                                }
                            } else if col==col_id {
                                for j in 0..blk_row {
                                    if self.flip_val(j,col,val_loc) { counter += 1; }
                                }
                                for j in blk_row+3..9 {
                                    if self.flip_val(j,col,val_loc) { counter += 1; }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        }
        if counter>0 { return true; }
        false
    }

    // If sanity check is turned on, counts the sum in each row, column and block.
    // This is on average a trustworthy sanity check.
    fn is_complete(&self,sanity_check: bool) -> bool {
        if sanity_check {
            for rowcol in 0..9 {
                let mut row_count = 0;
                let mut col_count = 0;

                let mut blk_count = 0;
                let mut blk_row = rowcol/3;
                let mut blk_col = rowcol%3;

                blk_row = blk_row*3;
                blk_col = blk_col*3;
                for colrow in 0..9 {
                    row_count += self.data[rowcol][colrow].value;
                    col_count += self.data[colrow][rowcol].value;
                    let row = blk_row+colrow/3;
                    let col = blk_col+colrow%3;
                    blk_count += self.data[row][col].value;
                }
                if row_count!=45 || col_count!=45 || blk_count!=45 {
                    return false;
                }
            }
        }

        for row in 0..9 {
        for col in 0..9 {
            if self.data[row][col].value==0 {
                return false;
            }
        }
        }

        true
    }

    fn dualism(&self) -> Vec<usize> {
        for row in 0..9 {
            let mut count = 0;
            let mut strategy: Vec<usize> = Vec::new();
            for val_loc in 0..9 {
                if self.row_counters[row][val_loc]==2 {
                    count += 1;
                    strategy.push(val_loc);
                } else if self.row_counters[row][val_loc]!=0 {
                    count = 0;
                    break;
                }
            }
            if count==2 {
                for col in 0..9 {
                    if self.data[row][col].value==0 {
                        strategy.push(row);
                        strategy.push(col);
                    }
                }
                return strategy;
            }
        }
        for col in 0..9 {
            let mut count = 0;
            let mut strategy: Vec<usize> = Vec::new();
            for val_loc in 0..9 {
                if self.col_counters[col][val_loc]==2 {
                    count += 1;
                    strategy.push(val_loc);
                } else if self.col_counters[col][val_loc]!=0 {
                    count = 0;
                    break;
                }
            }
            if count==2 {
                for row in 0..9 {
                    if self.data[row][col].value==0 {
                        strategy.push(row);
                        strategy.push(col);
                    }
                }
                return strategy;
            }
        }
        for blk in 0..9 {
            let mut strategy: Vec<usize> = Vec::new();
            let mut probe: Vec<usize> = Vec::new();
            for val_loc in 0..9 {
                if self.blk_counters[blk][val_loc]==2 {
                    probe.push(val_loc);
                }
            }
            if probe.len()>1 {
                let mut blk_row = blk/3;
                let mut blk_col = blk%3;

                blk_row = blk_row*3;
                blk_col = blk_col*3;
                let mut pos1 = -1;
                let mut pos2 = -1;
                for val1 in 0..probe.len()-1 {
                    for colrow in 0..9 {
                        let row = blk_row+colrow/3;
                        let col = blk_col+colrow%3;
                        if self.data[row][col].candidates[probe[val1]] {
                            if pos1 == -1 {
                                pos1 = colrow;
                            } else {
                                pos2 = colrow;
                                break;
                            }
                        }
                    }
                    let mut pos3 = -1;
                    let mut pos4 = -1;
                    for val2 in val1+1..probe.len() {
                        for colrow2 in 0..9 {
                            let row2 = blk_row+colrow2/3;
                            let col2 = blk_col+colrow2%3;
                            if self.data[row2][col2].candidates[probe[val2]] {
                                if pos3 == -1 {
                                    pos3 = colrow2;
                                } else {
                                    pos4 = colrow2;
                                    break;
                                }
                            }
                        }
                        if pos3==pos1 && pos4==pos2 {
                            strategy.push(probe[val1]);
                            strategy.push(probe[val2]);
                            strategy.push( blk_row+pos3/3 );
                            strategy.push( blk_col+pos3%3 );
                            strategy.push( blk_row+pos4/3 );
                            strategy.push( blk_col+pos4%3 );
                            break;
                        }
                    }
                    if strategy.len()==6 { return strategy; }
                }
            }
        }
        println!("");
        vec![0]
    }

    fn corner_val(&self) -> u32 {
        let mut val = self.data[0][2].value as u32;
        val += (self.data[0][1].value as u32)*10;
        val += (self.data[0][0].value as u32)*100;
        val
    }

    fn print(&self) {
        print!( "\n" );
        for i in 0..9 {
            for j in 0..9 {
                print!( "{}", self.data[i][j].value);
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

    let mut success_count = 0;
    let mut corner_sum = 0;
    for sudoku_idx in 0..sudokus.len() {
        let success = sudoku_loop( &mut sudokus[sudoku_idx],1 );
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

fn sudoku_copy(sudoku1 : &mut SudokuGrid, sudoku2 : &mut SudokuGrid) {
    for row in 0..9 {
    for col in 0..9 {
        if sudoku2.data[row][col].value == 0 && sudoku1.data[row][col].value != 0 {
            sudoku2.set_val( row, col, sudoku1.data[row][col].value );
        }
    }
    }
}

fn sudoku_loop(sudoku : &mut SudokuGrid, depth: usize) -> bool {
    while !sudoku.is_complete(false) {
        if !sudoku.update() {
            if depth<1 { return false; }
            let dual = sudoku.dualism();
            if dual.len()!=6 { 
                println!("Failure1!");
                return false; 
            }
            let mut probe_sudoku1 : SudokuGrid = SudokuGrid::new();
            sudoku_copy( sudoku, &mut probe_sudoku1 );
            probe_sudoku1.set_val( dual[2], dual[3], dual[0] as u8 + 1 );
            probe_sudoku1.set_val( dual[4], dual[5], dual[1] as u8 + 1 );
            if sudoku_loop( &mut probe_sudoku1, depth-1 ) {
                sudoku_copy( &mut probe_sudoku1, sudoku );
            } else {
                let mut probe_sudoku2 : SudokuGrid = SudokuGrid::new();
                sudoku_copy( sudoku, &mut probe_sudoku2 );
                probe_sudoku2.set_val( dual[2], dual[3], dual[1] as u8 + 1 );
                probe_sudoku2.set_val( dual[4], dual[5], dual[0] as u8 + 1 );
                if sudoku_loop( &mut probe_sudoku2, depth-1 ) {
                    sudoku_copy( &mut probe_sudoku2, sudoku );
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
