
pub struct Indices {
    pub rows: Vec< Vec<(usize,usize)> >,
    pub cols: Vec< Vec<(usize,usize)> >,
    pub blks: Vec< Vec<(usize,usize)> >,
}

// Initialize containers
impl Indices {
    pub fn new()->Indices {
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
