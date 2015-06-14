use std::io::prelude::*;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::env;

struct Indices {
    rows: Vec< Vec<(usize,usize)> >,
    cols: Vec< Vec<(usize,usize)> >,
    blks: Vec< Vec<(usize,usize)> >,
}

// Initialize containers
impl Indices {
    fn new()->Indices {
        // Setup vectors for positions in the grid
        let mut row_pos: Vec< Vec<(usize,usize)> > = Vec::new();
        let mut col_pos: Vec< Vec<(usize,usize)> > = Vec::new();
        let mut blk_pos: Vec< Vec<(usize,usize)> > = Vec::new();
        for i in 0..9 {
            let mut row_sub: Vec<(usize,usize)> = Vec::new();
            let mut col_sub: Vec<(usize,usize)> = Vec::new();
            let mut blk_sub: Vec<(usize,usize)> = Vec::new();
            for j in 0..9 {
                row_sub.push( (i,j) );
                col_sub.push( (j,i) );
                let mut blk_row = i/3;
                let mut blk_col = i%3;
                blk_row *= 3;
                blk_col *= 3;
                blk_sub.push( (blk_row+j/3,blk_col+j%3) );
            }
            row_pos.push( row_sub );
            col_pos.push( col_sub );
            blk_pos.push( blk_sub );
        }
        Indices {
            rows: row_pos,
            cols: col_pos,
            blks: blk_pos,
        }
    }
}


struct SudokuCell {
    value: u8,
    candidates: Vec<bool>,
    candidate_amnt: u8,

    blk_id: usize,
}

impl SudokuCell {
    fn new(row: usize, col: usize) -> SudokuCell {
        SudokuCell {
            value: 0,
            candidates: vec![true;9],
            candidate_amnt: 9,

            blk_id: 3*row+col,
        }
    }
}

struct SudokuGrid<'a> {
    data: Vec< Vec<SudokuCell> >,

    row_counters: Vec< Vec<u8> >,
    col_counters: Vec< Vec<u8> >,
    blk_counters: Vec< Vec<u8> >,

    indexing: &'a Indices,
}

// Initialize containers
impl<'a> SudokuGrid<'a> {
    fn new(ind: &'a Indices)->SudokuGrid<'a> {
        let mut init1: Vec< Vec<SudokuCell> > = Vec::new();
        let mut i_row1: Vec< Vec<u8> > = Vec::new();
        let mut i_col1: Vec< Vec<u8> > = Vec::new();
        let mut i_blk1: Vec< Vec<u8> > = Vec::new();
        for i in 0..9 {
            let mut init2: Vec<SudokuCell> = Vec::new();
            let i_row2: Vec<u8> = vec![9;9];
            let i_col2: Vec<u8> = vec![9;9];
            let i_blk2: Vec<u8> = vec![9;9];

            for j in 0..9 {
                init2.push( SudokuCell::new(i/3,j/3) );
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

            indexing: ind,
        }
    }

    fn set_cell(&mut self, row: usize, col: usize, val: u8) {
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
        for icol in self.indexing.rows[row].iter() { self.flip_val(icol.0,icol.1,val_loc); }
        for irow in self.indexing.cols[col].iter() { self.flip_val(irow.0,irow.1,val_loc); }
        for iblk in self.indexing.blks[self.data[row][col].blk_id].iter() {
            self.flip_val( iblk.0, iblk.1, val_loc );
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
        for element in 0..9 {
            for val_loc in 0..9 {
                if self.row_counters[element][val_loc]==1 {
                    sets += 1;
                    let loc = self.update_loop( element, val_loc, 0 );
                    self.set_cell(loc.0,loc.1,val_loc as u8+1);
                }

                if self.col_counters[element][val_loc]==1 {
                    sets += 1;
                    let loc = self.update_loop( element, val_loc, 1 );
                    self.set_cell(loc.0,loc.1,val_loc as u8+1);
                }

                if self.blk_counters[element][val_loc]==1 {
                    sets += 1;
                    let loc = self.update_loop( element, val_loc, 2 );
                    self.set_cell(loc.0,loc.1,val_loc as u8+1);
                }
            }
        }

        // If nothing has worked, check if there is a single cell with no content
        if sets==0 { sets += self.singles(); }

        // If nothing has worked, check if there are aligned possibilities on a
        // row/col/blk. If this does nothing, return false.
        if sets==0 { return self.advanced_update(); }
        true
    }

    fn update_loop(&mut self, element : usize, val_loc : usize, mode : usize) -> (usize,usize){
        let iter = if mode==0 { self.indexing.rows[element].iter() }
              else if mode==1 { self.indexing.cols[element].iter() }
              else if mode==2 { self.indexing.blks[element].iter() }
                         else { self.indexing.rows[element].iter() };
        let mut location : (usize,usize) = (10,10);
        for pos in iter {
            if self.data[pos.0][pos.1].candidates[val_loc] {
                location = *pos;
            }
        }
        location
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
                        self.set_cell(row,col,val_loc as u8+1);
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
        // Loop over blocks and values
        for blk in 0..9 {
        for val_loc in 0..9 {
            if self.blk_counters[blk][val_loc]==2 {
                let mut first_row = 9; let mut first_col = 9;
                // Loop over the current block
                for pos in self.indexing.blks[blk].iter() {
                    // Find the 'pair' and see if they are on the same row/col
                    if self.data[pos.0][pos.1].candidates[val_loc] {
                        if first_row == 9 {
                            first_row = pos.0; first_col = pos.1;
                        } else {
                            if pos.0==first_row {
                                let blk_edge = pos.1-pos.1%3;
                                for j in 0..blk_edge {
                                    if self.flip_val(pos.0,j,val_loc) { counter += 1; }
                                }
                                for j in blk_edge+3..9 {
                                    if self.flip_val(pos.0,j,val_loc) { counter += 1; }
                                }
                            } else if pos.1==first_col {
                                let blk_edge = pos.0-pos.0%3;
                                for j in 0..blk_edge {
                                    if self.flip_val(j,pos.1,val_loc) { counter += 1; }
                                }
                                for j in blk_edge+3..9 {
                                    if self.flip_val(j,pos.1,val_loc) { counter += 1; }
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
            for element_id in 0..9 {
                let row_count : u32 = self.indexing.rows[element_id].iter().map(|&rowcol| self.data[rowcol.0][rowcol.1].value as u32).fold(0, |acc, item| acc + item);
                let col_count : u32 = self.indexing.cols[element_id].iter().map(|&rowcol| self.data[rowcol.0][rowcol.1].value as u32).fold(0, |acc, item| acc + item);
                let blk_count : u32 = self.indexing.blks[element_id].iter().map(|&rowcol| self.data[rowcol.0][rowcol.1].value as u32).fold(0, |acc, item| acc + item);
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

    // Actual body of dualism (below), finds two cells with only two possible values
    // Type: 0 rows, 1 cols, 2 blks
    fn general_dualism(&mut self, rowscols : &Vec< Vec<(usize,usize)> >, mode : u32) -> Vec<usize> {
        for element in 0..9 {
            let mut strategy: Vec<usize> = Vec::new();
            let mut probe: Vec<usize> = Vec::new();
            for val_loc in 0..9 {
                if mode==0 && self.row_counters[element][val_loc]==2 { probe.push(val_loc); }
                if mode==1 && self.col_counters[element][val_loc]==2 { probe.push(val_loc); }
                if mode==2 && self.blk_counters[element][val_loc]==2 { probe.push(val_loc); }
            }
            if probe.len()>1 {
                let mut pos1 : (usize,usize) = (10,0); let mut pos2 : (usize,usize) = (0,0);
                for val1 in 0..probe.len()-1 {
                    for rowcol in rowscols[element].iter() {
                        if self.data[rowcol.0][rowcol.1].candidates[probe[val1]] {
                            if pos1.0 == 10 { pos1 = *rowcol; }
                            else { pos2 = *rowcol; break; }
                        }
                    }
                    let mut pos3 : (usize,usize) = (11,0); let mut pos4 : (usize,usize) = (10,0);
                    for val2 in val1+1..probe.len() {
                        for rowcol in rowscols[element].iter() {
                            if self.data[rowcol.0][rowcol.1].candidates[probe[val2]] {
                                if pos3.0 == 11 { pos3 = *rowcol; }
                                else { pos4 = *rowcol; break; }
                            }
                        }
                        if pos3==pos1 && pos4==pos2 {
                            strategy.push(probe[val1]); strategy.push(probe[val2]);
                            strategy.push( pos3.0 ); strategy.push( pos3.1 );
                            strategy.push( pos4.0 ); strategy.push( pos4.1 );
                            break;
                        }
                    }
                    if strategy.len()==6 {
                        return strategy;
                    }
                }
            }
        }
        Vec::new()
    }

    // Find two cells from rows/cols/blks in which only two alternative values fit.
    // Both of the possible solutions will be tried out.
    fn dualism(&mut self) -> Vec<usize> {
        let vec1 = self.general_dualism( &self.indexing.rows, 0 );
        if vec1.len()==6 { return vec1; }

        let vec2 = self.general_dualism( &self.indexing.cols, 1 );
        if vec2.len()==6 { return vec2; }

        let vec3 = self.general_dualism( &self.indexing.blks, 2 );
        if vec3.len()==6 { return vec3; }

        Vec::new()
    }

    // Calculate the value formed by the three digits in the left corner
    fn corner_val(&self) -> u32 {
        let mut val = self.data[0][2].value as u32;
        val += (self.data[0][1].value as u32)*10;
        val += (self.data[0][0].value as u32)*100;
        val
    }

    // A generic printing functiion
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
            let mut probe_sudoku1 : SudokuGrid = SudokuGrid::new(indices);
            sudoku_copy( sudoku, &mut probe_sudoku1 );
            probe_sudoku1.set_cell( dual[2], dual[3], dual[0] as u8 + 1);
            probe_sudoku1.set_cell( dual[4], dual[5], dual[1] as u8 + 1);
            if sudoku_loop( &mut probe_sudoku1, depth-1, indices ) {
                sudoku_copy( &mut probe_sudoku1, sudoku );
            } else {
                let mut probe_sudoku2 : SudokuGrid = SudokuGrid::new(indices);
                sudoku_copy( sudoku, &mut probe_sudoku2 );
                probe_sudoku2.set_cell( dual[2], dual[3], dual[1] as u8 + 1);
                probe_sudoku2.set_cell( dual[4], dual[5], dual[0] as u8 + 1);
                if sudoku_loop( &mut probe_sudoku2, depth-1, indices ) {
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

fn sudoku_copy(sudoku1 : &mut SudokuGrid, sudoku2 : &mut SudokuGrid) {
    for row in 0..9 {
    for col in 0..9 {
        if sudoku2.data[row][col].value == 0 && sudoku1.data[row][col].value != 0 {
            sudoku2.set_cell( row, col, sudoku1.data[row][col].value);
        }
    }
    }
}
