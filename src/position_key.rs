use crate::board::{CBoard, Color};

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
