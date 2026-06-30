use crate::board::{CBoard, Color};
use std::collections::HashMap;
use crate::chess_move::Move;


pub struct TranspositionTable{
    entries: Vec<Option<TTEntry>>,
    mask: usize,
}


impl TranspositionTable{
    pub fn new(size_mb: usize) -> Self{
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let raw_len = (size_mb * 1024 * 1024) / entry_size;
        let len = raw_len.next_power_of_two().max(1);

        Self{
            entries: vec![None; len],
            mask : len - 1,
        }
    }

    fn index(&self, key: u64)-> usize{
        key as usize & self.mask
    }
    pub fn get(&self, key : u64) -> Option<TTEntry> {
        let entry = self.entries[self.index(key)]?;

        if entry.key == key {
            Some(entry)
        }
        else{
            None
        }
    }

    pub fn insert(&mut self, entry : TTEntry){
        let index = self.index(entry.key);

        let replace = match self.entries[index] {
            None =>  true,
            Some(old) => entry.depth >= old.depth,
        };

        if replace{
            self.entries[index] = Some(entry);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClePosition {
    pub piece_bb: [u64; 14],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant_square: Option<u8>,
}

pub fn cle_position(board: &CBoard) -> ClePosition {
    ClePosition {
        piece_bb: board.piece_bb,
        side_to_move: board.side_to_move,
        castling_rights: board.castling_rights,
        en_passant_square: board.en_passant_square,
    }
}



#[derive(Clone, Copy, Debug,PartialEq,Eq)]
pub enum TTFlag{
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry{
    pub key :u64,
    pub depth: u32,
    pub score: i32,
    pub flag : TTFlag,
    pub best_move: Option<Move>,
}

