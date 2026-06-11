use crate::attack_tables::AttackTables;

use crate::attack_tables::{masques_mouvements_fou, masques_mouvements_tour};
use crate::board::{CBoard, Color, Pieces};

pub fn is_square_attacked(
    board: &CBoard,
    tables: &AttackTables,
    square: u8,
    by_color: Color,
) -> bool {
    let square_bb = 1u64 << square;
    match by_color {
        Color::Blanc => {
            //pion
            if tables.pawn_attacks[Color::Noir as usize][square as usize]
                & board.piece_bb[Pieces::PionBlanc as usize]
                != 0
            {
                return true;
            }
            //fou + dame
            if masques_mouvements_fou(square as usize, board.occupe_bb)
                & (board.piece_bb[Pieces::FouBlanc as usize]
                    | board.piece_bb[Pieces::DameBlanche as usize])
                != 0
            {
                return true;
            }

            //tour + dame
            if masques_mouvements_tour(square as usize, board.occupe_bb)
                & (board.piece_bb[Pieces::TourBlanche as usize]
                    | board.piece_bb[Pieces::DameBlanche as usize])
                != 0
            {
                return true;
            }
            // cavalier
            if tables.knight_attacks[square as usize]
                & board.piece_bb[Pieces::CavalierBlanc as usize]
                != 0
            {
                return true;
            }

            if tables.king_attacks[square as usize] & board.piece_bb[Pieces::RoiBlanc as usize] != 0
            {
                return true;
            }
            false
        }
        Color::Noir => {
            //pion
            if tables.pawn_attacks[Color::Blanc as usize][square as usize]
                & board.piece_bb[Pieces::PionNoir as usize]
                != 0
            {
                return true;
            }
            //fou + dame
            if masques_mouvements_fou(square as usize, board.occupe_bb)
                & (board.piece_bb[Pieces::FouNoir as usize]
                    | board.piece_bb[Pieces::DameNoire as usize])
                != 0
            {
                return true;
            }

            //tour + dame
            if masques_mouvements_tour(square as usize, board.occupe_bb)
                & (board.piece_bb[Pieces::TourNoire as usize]
                    | board.piece_bb[Pieces::DameNoire as usize])
                != 0
            {
                return true;
            }
            // cavalier
            if tables.knight_attacks[square as usize]
                & board.piece_bb[Pieces::CavalierNoir as usize]
                != 0
            {
                return true;
            }
            //roi
            if tables.king_attacks[square as usize] & board.piece_bb[Pieces::RoiNoir as usize] != 0
            {
                return true;
            }
            false
        }
    }
}

pub fn is_king_in_check(board: &CBoard, tables: &AttackTables, color: Color) -> bool {
    let king = match color {
        Color::Blanc => board.white_king_square,
        Color::Noir => board.black_king_square,
    };

    let enemy_color = match color {
        Color::Blanc => Color::Noir,
        Color::Noir => Color::Blanc,
    };
    is_square_attacked(board, tables, king, enemy_color)
}
