use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, CBoard, Color, Pieces, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};
use crate::chess_move::{Move, MoveFlag};

pub fn make_move(board: &mut CBoard, mv: Move) {
    let from_bb = 1u64 << mv.from;
    let to_bb = 1u64 << mv.to;
    let couleur_avant_coup = board.side_to_move;

    //enlever piece de depart
    board.piece_bb[mv.piece as usize] &= !(from_bb);

    //enlever la piece capturer
    if let Some(captured_piece) = mv.captured {
        if mv.flag != MoveFlag::EnPassant {
            board.piece_bb[captured_piece as usize] &= !(to_bb);
        }
        match (captured_piece, mv.to) {
            (Pieces::TourBlanche, 7) => {
                board.castling_rights &= !(WHITE_KINGSIDE);
            }
            (Pieces::TourBlanche, 0) => {
                board.castling_rights &= !(WHITE_QUEENSIDE);
            }
            (Pieces::TourNoire, 63) => {
                board.castling_rights &= !(BLACK_KINGSIDE);
            }
            (Pieces::TourNoire, 56) => {
                board.castling_rights &= !(BLACK_QUEENSIDE);
            }
            _ => {}
        }
    }

    //gerer la promotion et le changement de position de la piece

    if let Some(promotion) = mv.promotion {
        board.piece_bb[promotion as usize] |= to_bb;
    } else {
        board.piece_bb[mv.piece as usize] |= to_bb;
    }

    //gerer la prise en passant
    if mv.flag == MoveFlag::EnPassant {
        match mv.piece {
            Pieces::PionBlanc => {
                let captured_piece = mv.to - 8;
                board.piece_bb[Pieces::PionNoir as usize] &= !(1u64 << captured_piece);
            }
            Pieces::PionNoir => {
                let captured_piece = mv.to + 8;
                board.piece_bb[Pieces::PionBlanc as usize] &= !(1u64 << captured_piece);
            }
            _ => {}
        }
    }
    //castling

    if mv.flag == MoveFlag::Castling {
        match (mv.piece, mv.from, mv.to) {
            (Pieces::RoiBlanc, 4, 6) => {
                board.piece_bb[Pieces::TourBlanche as usize] &= !(1u64 << 7);
                board.piece_bb[Pieces::TourBlanche as usize] |= (1u64 << 5);
            }
            (Pieces::RoiBlanc, 4, 2) => {
                board.piece_bb[Pieces::TourBlanche as usize] &= !(1u64 << 0);
                board.piece_bb[Pieces::TourBlanche as usize] |= (1u64 << 3);
            }
            (Pieces::RoiNoir, 60, 62) => {
                board.piece_bb[Pieces::TourNoire as usize] &= !(1u64 << 63);
                board.piece_bb[Pieces::TourNoire as usize] |= (1u64 << 61);
            }
            (Pieces::RoiNoir, 60, 58) => {
                board.piece_bb[Pieces::TourNoire as usize] &= !(1u64 << 56);
                board.piece_bb[Pieces::TourNoire as usize] |= (1u64 << 59);
            }
            _ => {}
        }
    }

    //mise a jour position du roi
    match mv.piece {
        Pieces::RoiNoir => {
            board.castling_rights &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
            board.black_king_square = mv.to;
        }
        Pieces::RoiBlanc => {
            board.castling_rights &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
            board.white_king_square = mv.to;
        }
        Pieces::TourBlanche => {
            if mv.from == 7 {
                board.castling_rights &= !(WHITE_KINGSIDE);
            } else if mv.from == 0 {
                board.castling_rights &= !(WHITE_QUEENSIDE);
            }
        }
        Pieces::TourNoire => {
            if mv.from == 63 {
                board.castling_rights &= !(BLACK_KINGSIDE);
            } else if mv.from == 56 {
                board.castling_rights &= !(BLACK_QUEENSIDE);
            }
        }
        _ => {}
    }

    // mis a jour de en passant

    board.en_passant_square = None;

    if mv.flag == MoveFlag::DoublePawnPush {
        match mv.piece {
            Pieces::PionBlanc => {
                board.en_passant_square = Some(mv.from + 8);
            }
            Pieces::PionNoir => {
                board.en_passant_square = Some(mv.from - 8);
            }
            _ => {}
        }
    }
    //si la couleur du dernier coups est noir alors le nombre de fullmove augmente
    if matches!(couleur_avant_coup, Color::Noir) {
        board.fullmove_number += 1;
    }
    let reset_halfmove = matches!(mv.piece, Pieces::PionBlanc | Pieces::PionNoir)
        || mv.captured.is_some()
        || matches!(mv.flag, MoveFlag::EnPassant);
    if reset_halfmove {
        board.halfmove_clock = 0
    } else {
        board.halfmove_clock += 1;
    }

    board.side_to_move = match board.side_to_move {
        Color::Blanc => Color::Noir,
        Color::Noir => Color::Blanc,
    };
    board.update_occupancies();
}
