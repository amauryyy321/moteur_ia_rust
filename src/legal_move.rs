use crate::attack_tables::AttackTables;
use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::Move;
use crate::legality::is_king_in_check;
use crate::make_move::make_move;
use crate::notation::coord_to_square_index;
use crate::pseudo_legal_move::generate_pseudo_legal_move;

pub fn generate_legal_move(board: &mut CBoard, tables: &AttackTables) -> Vec<Move> {
    let gen_pseudo_move = generate_pseudo_legal_move(board, tables);
    let color = board.side_to_move;
    let mut legal_move = Vec::new();

    for mv in gen_pseudo_move {
        let old_board = board.clone();
        make_move(board, mv);
        if !is_king_in_check(board, tables, color) {
            legal_move.push(mv);
        }
        *board = old_board;
    }
    legal_move
}
pub fn trouver_coup_legal(board: &mut CBoard, tables: &AttackTables, texte: &str) -> Option<Move> {
    if texte.len() < 4 {
        return None;
    }

    let from = coord_to_square_index(&texte[0..2]).ok()?;
    let to = coord_to_square_index(&texte[2..4]).ok()?;

    let promotion = if texte.len() == 5 {
        let lettre = texte.chars().nth(4)?;
        match (lettre, board.side_to_move) {
            ('q' | 'Q', Color::Blanc) => Some(Pieces::DameBlanche),
            ('r' | 'R', Color::Blanc) => Some(Pieces::TourBlanche),
            ('b' | 'B', Color::Blanc) => Some(Pieces::FouBlanc),
            ('n' | 'N', Color::Blanc) => Some(Pieces::CavalierBlanc),
            ('q' | 'Q', Color::Noir) => Some(Pieces::DameNoire),
            ('r' | 'R', Color::Noir) => Some(Pieces::TourNoire),
            ('b' | 'B', Color::Noir) => Some(Pieces::FouNoir),
            ('n' | 'N', Color::Noir) => Some(Pieces::CavalierNoir),
            _ => return None,
        }
    } else {
        None
    };
    generate_legal_move(board, tables)
        .into_iter()
        .find(|mv| mv.from == from && mv.to == to && promotion == mv.promotion)
}
