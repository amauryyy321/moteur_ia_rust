use crate::attack_tables::init_attack_tables;
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, CBoard, Color, Pieces, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};
use crate::chess_move::MoveFlag;
use crate::legal_move::generate_legal_move;
use crate::notation::coord_to_square_index;
pub fn board_from_fen(fen: &str) -> Result<CBoard, String> {
    let mut piece_bb = [0u64; 14];
    let mut rank = 7;
    let mut file = 0;
    let mut parts = fen.split_whitespace();
    let placement = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;
    let side = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;
    let castling = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;
    let en_passant = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;
    let half_move = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;
    let full_move = parts
        .next()
        .ok_or("Fen invalide : placement manquant".to_string())?;

    for (lettre) in placement.chars() {
        let square = (rank * 8 + file) as u8;

        match (lettre) {
            'r' => piece_bb[Pieces::TourNoire as usize] |= 1u64 << square,
            'n' => piece_bb[Pieces::CavalierNoir as usize] |= 1u64 << square,
            'b' => piece_bb[Pieces::FouNoir as usize] |= 1u64 << square,
            'k' => piece_bb[Pieces::RoiNoir as usize] |= 1u64 << square,
            'q' => piece_bb[Pieces::DameNoire as usize] |= 1u64 << square,
            'p' => piece_bb[Pieces::PionNoir as usize] |= 1u64 << square,
            'P' => piece_bb[Pieces::PionBlanc as usize] |= 1u64 << square,

            'R' => piece_bb[Pieces::TourBlanche as usize] |= 1u64 << square,
            'N' => piece_bb[Pieces::CavalierBlanc as usize] |= 1u64 << square,
            'B' => piece_bb[Pieces::FouBlanc as usize] |= 1u64 << square,
            'K' => piece_bb[Pieces::RoiBlanc as usize] |= 1u64 << square,
            'Q' => piece_bb[Pieces::DameBlanche as usize] |= 1u64 << square,
            n @ '1'..='8' => file += n.to_digit(10).unwrap() as i32,
            '/' => {
                if file != 8 {
                    return Err("la ligne est incomplete".to_string());
                }

                rank -= 1;
                file = 0;
            }
            _ => {
                return Err(format!("Caractere invalide : {}", lettre));
            }
        }

        if lettre != '/' && !('1'..='8').contains(&lettre) {
            file += 1;
        }
        if file > 8 {
            return Err("trop de file !".to_string());
        }
        if rank < 0 {
            return Err("trop de rank !".to_string());
        }
    }
    if rank != 0 || file != 8 {
        return Err("l echiquier est incomplet !".to_string());
    }
    let side_to_move = match (side) {
        "w" => Color::Blanc,
        "b" => Color::Noir,
        _ => return Err("Fen invalide : side_to_move doit etre w ou b".to_string()),
    };

    let mut castling_rights = 0u8;

    if castling != "-" {
        for c in castling.chars() {
            match (c) {
                'K' => {
                    castling_rights |= WHITE_KINGSIDE;
                }
                'Q' => {
                    castling_rights |= WHITE_QUEENSIDE;
                }
                'k' => {
                    castling_rights |= BLACK_KINGSIDE;
                }
                'q' => {
                    castling_rights |= BLACK_QUEENSIDE;
                }
                _ => return Err("Fen invalide : les droits de rock sont KQkq".to_string()),
            }
        }
    }

    let en_passant_square = match en_passant {
        "-" => None,
        coord => Some(coord_to_square_index(coord)?),
    };

    let halfmove_clock = half_move
        .parse::<u32>()
        .map_err(|_| "Fen invalide : false half move".to_string())?;
    let fullmove_number = full_move
        .parse::<u32>()
        .map_err(|_| "Fen invalide : false full move".to_string())?;

    if piece_bb[Pieces::RoiBlanc as usize].count_ones() != 1 {
        return Err("Fen invalide : pas de roi blanc sur l echiquier".to_string());
    }
    if piece_bb[Pieces::RoiNoir as usize].count_ones() != 1 {
        return Err("Fen invalide : pas de roi noir sur l echiquier".to_string());
    }

    let mut board = CBoard {
        piece_bb,
        vide_bb: 0,
        occupe_bb: 0,
        side_to_move,
        castling_rights,
        en_passant_square,
        halfmove_clock,
        fullmove_number,
        white_king_square: piece_bb[Pieces::RoiBlanc as usize].trailing_zeros() as u8,
        black_king_square: piece_bb[Pieces::RoiNoir as usize].trailing_zeros() as u8,
    };
    board.update_occupancies();
    Ok(board)
}

#[test]
pub fn test_compare_init_table_to_fen_init() {
    let board_1 =
        board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let board_2 = CBoard::init_position_depart();

    assert_eq!(board_1.piece_bb, board_2.piece_bb);
    assert_eq!(board_1.vide_bb, board_2.vide_bb);
    assert_eq!(board_1.occupe_bb, board_2.occupe_bb);
    assert_eq!(board_1.castling_rights, board_2.castling_rights);
    assert_eq!(board_1.en_passant_square, board_2.en_passant_square);
    assert_eq!(board_1.white_king_square, board_2.white_king_square);
    assert_eq!(board_1.black_king_square, board_2.black_king_square);
}
#[test]
pub fn test_fen_castling_moves_exist() {
    let tables = init_attack_tables();
    let mut board = board_from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let moves = generate_legal_move(&mut board, &tables);
    let white_king_side = moves
        .iter()
        .any(|m| m.from == 4 && m.to == 6 && m.flag == MoveFlag::Castling);
    let white_queen_side = moves
        .iter()
        .any(|m| m.from == 4 && m.to == 2 && m.flag == MoveFlag::Castling);
    assert!(white_king_side, "petit roque blanc absent");
    assert!(white_queen_side, "grand roque blanc absent");
}

#[test]
pub fn test_fen_en_passant_move_exist() {
    let tables = init_attack_tables();
    let mut board = board_from_fen("7k/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
    assert_eq!(board.en_passant_square, Some(43));
    let moves = generate_legal_move(&mut board, &tables);
    let ep_exists = moves
        .iter()
        .any(|m| m.from == 36 && m.to == 43 && m.flag == MoveFlag::EnPassant);

    assert!(ep_exists, "en passant mouvement absent");
}

#[test]
pub fn test_fen_promotion_exist() {
    let tables = init_attack_tables();
    let mut board = board_from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let moves = generate_legal_move(&mut board, &tables);
    let promotion: Vec<_> = moves
        .iter()
        .filter(|m| m.from == 48 && m.to == 56 && m.flag == MoveFlag::Promotion)
        .collect();

    assert_eq!(
        promotion.len(),
        4,
        "Toutes les promotions possibles ne sont pas gerer"
    );
    assert!(
        promotion
            .iter()
            .any(|m| m.promotion == Some(Pieces::DameBlanche))
    );
    assert!(
        promotion
            .iter()
            .any(|m| m.promotion == Some(Pieces::TourBlanche))
    );
    assert!(
        promotion
            .iter()
            .any(|m| m.promotion == Some(Pieces::FouBlanc))
    );
    assert!(
        promotion
            .iter()
            .any(|m| m.promotion == Some(Pieces::CavalierBlanc))
    );
}
