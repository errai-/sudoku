
pub struct SudokuCell {
    pub value: u8,
    pub candidates: Vec<bool>,
    pub candidate_amnt: u8,

    pub blk_id: usize,
}

impl Clone for SudokuCell {
    fn clone(&self) -> SudokuCell {
        let mut cands = Vec::new();
        for i in self.candidates.iter() {
            cands.push( *i );
        }

        SudokuCell {
            value: self.value,
            candidates: cands,
            candidate_amnt: self.candidate_amnt,

            blk_id: self.blk_id,
        }
    }
}


impl SudokuCell {
    pub fn new(row: usize, col: usize) -> SudokuCell {
        SudokuCell {
            value: 0,
            candidates: vec![true;9],
            candidate_amnt: 9,

            blk_id: 3*row+col,
        }
    }
}

