use crate::board::{CBoard, Color};
use std::collections::HashMap;
use crate::chess_move::Move;
pub type TranspositionTable = HashMap <ClePosition, TTEntry>;


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
    pub depth: u32,
    pub score: i32,
    pub flag : TTFlag,
    pub best_move: Option<Move>,
}