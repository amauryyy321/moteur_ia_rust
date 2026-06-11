pub const WHITE_KINGSIDE: u8 = 0b0001;
pub const WHITE_QUEENSIDE: u8 = 0b0010;
pub const BLACK_KINGSIDE: u8 = 0b0100;
pub const BLACK_QUEENSIDE: u8 = 0b1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Blanc,
    Noir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pieces {
    PionBlanc,
    PionNoir,
    CavalierBlanc,
    CavalierNoir,
    FouBlanc,
    FouNoir,
    TourBlanche,
    TourNoire,
    DameBlanche,
    DameNoire,
    RoiBlanc,
    RoiNoir,
    PiecesBlanches,
    PiecesNoires,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CBoard {
    pub piece_bb: [u64; 14],
    pub vide_bb: u64,
    pub occupe_bb: u64,
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant_square: Option<u8>,

    pub halfmove_clock: u32,
    pub fullmove_number: u32,

    pub white_king_square: u8,
    pub black_king_square: u8,
}
impl CBoard {
    pub fn init_position_depart() -> CBoard {
        let piece_bb = Self::init_piece_tableau();

        let occupe_bb =
            piece_bb[Pieces::PiecesBlanches as usize] | piece_bb[Pieces::PiecesNoires as usize];

        let vide_bb = !occupe_bb;
        let side_to_move = Color::Blanc;
        let halfmove_clock = 0;
        let fullmove_number = 0;
        let white_king_square = 4;
        let black_king_square = 60;
        let castling_rights = 15;
        let en_passant_square = None;

        CBoard {
            piece_bb,
            vide_bb,
            occupe_bb,
            side_to_move,
            castling_rights,
            en_passant_square,
            halfmove_clock,
            fullmove_number,
            white_king_square,
            black_king_square,
        }
    }

    pub fn init_piece_tableau() -> [u64; 14] {
        let mut piece_bb = [0u64; 14];

        piece_bb[Pieces::DameBlanche as usize] = 1u64 << 3;
        piece_bb[Pieces::RoiBlanc as usize] = 1u64 << 4;
        piece_bb[Pieces::FouBlanc as usize] = (1u64 << 2) | (1u64 << 5);
        piece_bb[Pieces::CavalierBlanc as usize] = (1u64 << 1) | (1u64 << 6);
        piece_bb[Pieces::TourBlanche as usize] = (1u64 << 0) | (1u64 << 7);
        piece_bb[Pieces::PionBlanc as usize] = ((1u64 << 8) - 1) << 8;

        piece_bb[Pieces::PionNoir as usize] = ((1u64 << 8) - 1) << 48;
        piece_bb[Pieces::TourNoire as usize] = (1u64 << 56) | (1u64 << 63);
        piece_bb[Pieces::CavalierNoir as usize] = (1u64 << 57) | (1u64 << 62);
        piece_bb[Pieces::FouNoir as usize] = (1u64 << 58) | (1u64 << 61);
        piece_bb[Pieces::DameNoire as usize] = 1u64 << 59;
        piece_bb[Pieces::RoiNoir as usize] = 1u64 << 60;

        piece_bb[Pieces::PiecesBlanches as usize] = piece_bb[Pieces::PionBlanc as usize]
            | piece_bb[Pieces::CavalierBlanc as usize]
            | piece_bb[Pieces::FouBlanc as usize]
            | piece_bb[Pieces::TourBlanche as usize]
            | piece_bb[Pieces::DameBlanche as usize]
            | piece_bb[Pieces::RoiBlanc as usize];

        piece_bb[Pieces::PiecesNoires as usize] = piece_bb[Pieces::PionNoir as usize]
            | piece_bb[Pieces::CavalierNoir as usize]
            | piece_bb[Pieces::FouNoir as usize]
            | piece_bb[Pieces::TourNoire as usize]
            | piece_bb[Pieces::DameNoire as usize]
            | piece_bb[Pieces::RoiNoir as usize];

        piece_bb
    }
    pub fn update_occupancies(&mut self) {
        self.piece_bb[Pieces::PiecesBlanches as usize] = self.piece_bb[Pieces::PionBlanc as usize]
            | self.piece_bb[Pieces::CavalierBlanc as usize]
            | self.piece_bb[Pieces::FouBlanc as usize]
            | self.piece_bb[Pieces::TourBlanche as usize]
            | self.piece_bb[Pieces::DameBlanche as usize]
            | self.piece_bb[Pieces::RoiBlanc as usize];

        self.piece_bb[Pieces::PiecesNoires as usize] = self.piece_bb[Pieces::PionNoir as usize]
            | self.piece_bb[Pieces::CavalierNoir as usize]
            | self.piece_bb[Pieces::FouNoir as usize]
            | self.piece_bb[Pieces::TourNoire as usize]
            | self.piece_bb[Pieces::DameNoire as usize]
            | self.piece_bb[Pieces::RoiNoir as usize];

        self.occupe_bb = self.piece_bb[Pieces::PiecesBlanches as usize]
            | self.piece_bb[Pieces::PiecesNoires as usize];
        self.vide_bb = !self.occupe_bb;
    }
}
