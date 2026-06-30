use crate::board::{CBoard,Color};
#[derive(Clone)]
pub struct ZobristKeys {
    pub pieces : [[u64;64]; 12],
    pub side_to_move: u64,
    pub castling: [u64;16],
    pub en_passant_file: [u64; 8],
}

fn splitmix64(state : &mut u64) -> u64{
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

impl ZobristKeys {
    pub fn new() -> Self {
        let mut seed = 0x1234_5678_9ABC_DEF0;
        let mut pieces = [[0u64; 64]; 12];

        for piece in 0..12 {
            for square in 0..64 {
                pieces[piece][square] = splitmix64(&mut seed);
            }
        }
        let mut side_to_move = splitmix64(&mut seed);

        let mut castling = [0u64;16];
        for value in castling.iter_mut() {
            *value = splitmix64(&mut seed);
        }

        let mut en_passant_file = [0u64; 8];
        for value in en_passant_file.iter_mut(){
            *value = splitmix64(&mut seed);
        }

        Self{
            pieces,
            side_to_move,
            castling,
            en_passant_file,
        }
    }
}
impl Default for ZobristKeys{
    fn default() -> Self{
        Self::new()
    }
}
pub fn zobrist_hash (board: &CBoard, keys: &ZobristKeys) -> u64{
    let mut hash = 0u64;
    for piece_index in 0..12 {
        let mut bb = board.piece_bb[piece_index];

        while bb != 0 {
            let square = bb.trailing_zeros() as usize;
            hash ^= keys.pieces[piece_index][square];
            bb &= bb -1;
        }
    }
    if matches!(board.side_to_move, Color::Noir){
        hash ^= keys.side_to_move;
    }

    hash ^= keys.castling[board.castling_rights as usize];

    if let Some(square) = board.en_passant_square {
        let file = (square % 8) as usize;
        hash ^= keys.en_passant_file[file];
    }
    hash
}