mod main_test;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{format, Formatter};
use std::io::Read;
use std::iter::zip;
use std::num::NonZeroU8;
use std::ops::DerefMut;
use std::rc::Rc;

/*#[derive(Debug)]
struct Cell {
    value: Option<u8>,
    possible: Vec<u8>
}

impl Cell {
    pub fn new() -> Self {
        Self {
            value: None,
            possible: vec![1,2,3,4,5,6,7,8,9],
        }
    }
    pub fn set_value(&mut self, v: u8) {
        if let Some(v_old) = self.value {
            self.possible.push(v_old);
        }
        self.value = Some(v);
    }
}*/

#[derive(Debug)]
enum Cell {
    Value(u8),
    Possibilities(HashSet<u8>)
}

#[derive(Debug)]
struct Board {
    board: [Cell;81],
    /*pub row_board: [&'a RefCell<Cell>; 81],
    pub col_board: [&'a RefCell<Cell>; 81],
    pub field_board: [&'a RefCell<Cell>; 81]*/
}

fn calculate_field_index(i: usize) -> usize {
    let block_nr = i / 9;
    let col = block_nr % 3;
    let row = block_nr / 3;
    let block_idx = i % 9;
    (block_idx % 3) + 3 * col + block_idx / 3 * 9 + 27 * row
}


impl Board {
    pub fn new() -> Self {
        let mut board: Vec<Cell> = vec![];
        for i in 0..81 {
            board.push(Cell::Possibilities(HashSet::from_iter(1..10)));
        }
        Self {
            board: board.try_into().unwrap(),
        }
    }
    pub fn load_sudoku(&mut self, sudoku: &str) {
        let s = sudoku.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .map(|c| if c == 0 {None} else {Some(c)})
            .collect::<Vec<Option<u8>>>();
        for (cell,value) in zip(&mut self.board, s) {
            if let Some(v) = value {
                *cell = Cell::Value(v);
            }
        }
    }


    fn get_row_indizi(&self, i:usize) -> impl Iterator<Item=usize> {
        let row_offset = (i/9)*9;
        (0..9).map(move |u| row_offset + u)
    }

    fn get_col_indizi(&self, i:usize) -> impl Iterator<Item=usize> {
        let col_offset = (i%9);
        (0..9).map(move |u| col_offset + u*9)
    }

    fn get_field_indizi_by_start_index(&self, field_start_idx: usize) -> impl Iterator<Item=usize> {
        (0..9).map(move |u| {
            let field_column = u % 3;
            let field_row = u / 3;
            field_start_idx + field_column + field_row*9
        })
    }

    fn get_field_indizi(&self, i: usize) -> impl Iterator<Item=usize> {
        let field_start_idx = (i / 27)*27+((i%9)/3)*3;
        self.get_field_indizi_by_start_index(field_start_idx)
    }

    fn get_all_inidzi_to_check(&self, idx: usize) -> impl Iterator<Item=usize> {
        self.get_row_indizi(idx)
            .chain(self.get_col_indizi(idx))
            .chain(self.get_field_indizi(idx))
    }

    pub fn check_solved_cells(&mut self) -> bool {
        let mut found = false;
        for i in 0..81 {
            if let Cell::Value(value) = self.board[i] {
                for idx in  self.get_all_inidzi_to_check(i) {
                    if i != idx {
                        if let Cell::Possibilities(ref mut p) = self.board[idx] {
                            p.retain(|&x| x != value);
                            if p.len() == 1 {
                                //only one value left is fixed
                                self.board[idx] = Cell::Value(*p.iter().next().unwrap());
                                found = true;
                            }
                        }
                    }
                }
            }
        }
        found
    }

    pub fn hidden_single(&mut self) -> bool {
        let mut found = false;
        for field in 0..9 {
            let field_start_idx = field / 3 * 27 + (field % 3) * 3;
            let indizi = self.get_field_indizi_by_start_index(field_start_idx).collect::<Vec<_>>();
            for idx1 in &indizi {
                if let Cell::Possibilities(ref poss1) = self.board[*idx1] {
                    let mut other_possibilities = HashSet::new();
                    for idx2 in &indizi {
                        if idx1 != idx2 {
                            if let Cell::Possibilities(ref poss2) = self.board[*idx2] {
                                other_possibilities.extend(poss2);
                            }
                        }
                    }
                    let diff = poss1.difference(&other_possibilities).collect::<Vec<_>>();
                    if diff.len() == 1 {
                        self.board[*idx1] = Cell::Value(*diff[0]);
                        found = true;
                    }
                }
            }
        }
        found
    }

    fn handle_naked_pairs(&mut self,idx: usize, idx_to_check:usize,poss1: HashSet<u8>, to_change_iter: impl Iterator<Item=usize>) -> Option<(HashSet<u8>, Vec<usize>)> {
        if let Cell::Possibilities(ref poss_check) = self.board[idx_to_check] {
            if &poss1 == poss_check {
                let to_change_idx = to_change_iter
                    .filter(|i| *i != idx && *i != idx_to_check)
                    .collect::<Vec<_>>();
                return Some((poss1, to_change_idx));
            }
        }
        return None;
    }

    pub fn naked_pairs(&mut self) {

        let mut to_change: Vec<(HashSet<u8>, Vec<usize>)> = vec![];
        'outer: for idx in 0..81 {
            if let Cell::Possibilities(ref poss1) = self.board[idx] {

                if poss1.len() == 2 {
                    for idx_to_check in self.get_row_indizi(idx) {
                        if let Some(c) = self.handle_naked_pairs(idx, idx_to_check,poss1.clone(), self.get_row_indizi(idx)) {
                            to_change.push( c);
                        }
                    }

                    for idx_to_check in self.get_col_indizi(idx) {
                        if let Some(c) = self.handle_naked_pairs(idx, idx_to_check,poss1.clone(), self.get_col_indizi(idx)) {
                            to_change.push(c);
                        }
                    }

                    for idx_to_check in self.get_field_indizi(idx) {
                        if let Some(c) = self.handle_naked_pairs(idx, idx_to_check, poss1.clone(), self.get_field_indizi(idx)) {
                            to_change.push(c);
                        }
                    }


                }
            }
        }

        for (values, indizi) in &to_change {
            for i in indizi {
                if let Cell::Possibilities(ref mut p) = &mut self.board[*i] {
                    for v in values {
                        p.remove(v);
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for row in 0..9 {
            s.push_str("===================\n");
            for col in 0..9 {
                s.push_str("|");
                let v = match self.board[row*9+col] {
                    Cell::Value(v) => v.to_string(),
                    Cell::Possibilities(ref p) => format!("{:?}", p)
                };
                s.push_str(&format!("{}", v));
            }
            s.push_str("|");
            s.push_str("\n");
        }
        s.push_str("===================");
        write!(f, "{}", s)
    }
}



fn main() {
    let mut board = Board::new();
    board.load_sudoku("309000400200709000087000000750060230600904008028050041000000590000106007006000104");
    dbg!(&board.board[4]);

    loop {
        let mut res = false;
        res |= board.check_solved_cells();
        if !res {
            res |= board.hidden_single();
        }

        if !res {
            board.naked_pairs();
            res |= board.check_solved_cells();
        }

        if !res {
            break;
        }
    }
    println!("{}", board);
}
