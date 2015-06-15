
pub use indices::Indices;
pub use sudokucell::SudokuCell;

#[derive(Clone)]
pub struct SudokuGrid<'a> {
    data: Vec< Vec<SudokuCell> >,

    row_counters: Vec< Vec<u8> >,
    col_counters: Vec< Vec<u8> >,
    blk_counters: Vec< Vec<u8> >,

    indexing: &'a Indices,
}

// Initialize containers
impl<'a> SudokuGrid<'a> {
    pub fn new(ind: &'a Indices)->SudokuGrid<'a> {
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

    pub fn set_cell(&mut self, row: usize, col: usize, val: u8) {
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

    pub fn update(&mut self) -> bool {
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
    pub fn is_complete(&self,sanity_check: bool) -> bool {
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

    // Find two cells from rows/cols/blks in which only two alternative values fit.
    // Both of the possible solutions will be tried out.
    pub fn dualism(&mut self) -> Vec<usize> {
        let vec1 = self.general_dualism( &self.indexing.rows, 0 );
        if vec1.len()==6 { return vec1; }

        let vec2 = self.general_dualism( &self.indexing.cols, 1 );
        if vec2.len()==6 { return vec2; }

        let vec3 = self.general_dualism( &self.indexing.blks, 2 );
        if vec3.len()==6 { return vec3; }

        Vec::new()
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

    // Calculate the value formed by the three digits in the left corner
    pub fn corner_val(&self) -> u32 {
        let mut val = self.data[0][2].value as u32;
        val += (self.data[0][1].value as u32)*10;
        val += (self.data[0][0].value as u32)*100;
        val
    }

    // A generic printing functiion
    pub fn print(&self) {
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
