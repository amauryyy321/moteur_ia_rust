use moteur_ia::board::{
    CBoard, Pieces,
    WHITE_KINGSIDE, WHITE_QUEENSIDE,
};

use moteur_ia::chess_move::{Move, MoveFlag};
use moteur_ia::fen::board_from_fen;
use moteur_ia::make_move::make_move;
use moteur_ia::notation::{coord_to_square, square_to_coord};

#[test]
fn test_square_to_coord_and_coord_to_square() {
    assert_eq!(coord_to_square("a1"), 1u64 << 0);
    assert_eq!(coord_to_square("e4"), 1u64 << 28);
    assert_eq!(coord_to_square("h8"), 1u64 << 63);

    assert_eq!(square_to_coord(1u64 << 0), "a1");
    assert_eq!(square_to_coord(1u64 << 28), "e4");
    assert_eq!(square_to_coord(1u64 << 63), "h8");
}

#[test]
fn test_fen_startpos_equals_init_position() {
    let board_fen = board_from_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    let board_init = CBoard::init_position_depart();

    assert_eq!(board_fen.piece_bb, board_init.piece_bb);
    assert_eq!(board_fen.occupe_bb, board_init.occupe_bb);
    assert_eq!(board_fen.vide_bb, board_init.vide_bb);

    assert_eq!(board_fen.castling_rights, board_init.castling_rights);
    assert_eq!(board_fen.en_passant_square, board_init.en_passant_square);

    assert_eq!(board_fen.white_king_square, board_init.white_king_square);
    assert_eq!(board_fen.black_king_square, board_init.black_king_square);
}

#[test]
fn test_e2e4_sets_en_passant_square() {
    let mut board = CBoard::init_position_depart();

    let mv = Move {
        from: 12,
        to: 28,
        piece: Pieces::PionBlanc,
        captured: None,
        promotion: None,
        flag: MoveFlag::DoublePawnPush,
    };

    make_move(&mut board, mv);

    assert_eq!(board.en_passant_square, Some(20));
}

#[test]
fn test_white_kingside_castling_moves_king_and_rook() {
    let mut board = board_from_fen(
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
    ).unwrap();

    let mv = Move {
        from: 4,
        to: 6,
        piece: Pieces::RoiBlanc,
        captured: None,
        promotion: None,
        flag: MoveFlag::Castling,
    };

    make_move(&mut board, mv);

    assert_eq!(
        board.piece_bb[Pieces::RoiBlanc as usize] & (1u64 << 6),
        1u64 << 6
    );

    assert_eq!(
        board.piece_bb[Pieces::TourBlanche as usize] & (1u64 << 5),
        1u64 << 5
    );

    assert_eq!(
        board.piece_bb[Pieces::RoiBlanc as usize] & (1u64 << 4),
        0
    );

    assert_eq!(
        board.piece_bb[Pieces::TourBlanche as usize] & (1u64 << 7),
        0
    );

    assert_eq!(board.white_king_square, 6);

    assert_eq!(
        board.castling_rights & (WHITE_KINGSIDE | WHITE_QUEENSIDE),
        0
    );
}

#[test]
fn test_promotion_removes_pawn_and_adds_promoted_piece() {
    let mut board = board_from_fen(
        "4k3/P7/8/8/8/8/8/4K3 w - - 0 1"
    ).unwrap();

    let mv = Move {
        from: 48,
        to: 56,
        piece: Pieces::PionBlanc,
        captured: None,
        promotion: Some(Pieces::DameBlanche),
        flag: MoveFlag::Promotion,
    };

    make_move(&mut board, mv);

    assert_eq!(
        board.piece_bb[Pieces::PionBlanc as usize] & (1u64 << 48),
        0
    );

    assert_eq!(
        board.piece_bb[Pieces::PionBlanc as usize] & (1u64 << 56),
        0
    );

    assert_eq!(
        board.piece_bb[Pieces::DameBlanche as usize] & (1u64 << 56),
        1u64 << 56
    );

    assert_eq!(
        board.piece_bb[Pieces::PiecesBlanches as usize] & (1u64 << 56),
        1u64 << 56
    );

    assert_eq!(
        board.occupe_bb & (1u64 << 56),
        1u64 << 56
    );
}

#[test]
fn test_en_passant_removes_correct_pawn() {
    let mut board = board_from_fen(
        "7k/8/8/3pP3/8/8/8/4K3 w - d6 0 1"
    ).unwrap();

    let mv = Move {
        from: 36,
        to: 43,
        piece: Pieces::PionBlanc,
        captured: Some(Pieces::PionNoir),
        promotion: None,
        flag: MoveFlag::EnPassant,
    };

    make_move(&mut board, mv);

    assert_eq!(
        board.piece_bb[Pieces::PionBlanc as usize] & (1u64 << 43),
        1u64 << 43
    );

    assert_eq!(
        board.piece_bb[Pieces::PionBlanc as usize] & (1u64 << 36),
        0
    );

    assert_eq!(
        board.piece_bb[Pieces::PionNoir as usize] & (1u64 << 35),
        0
    );

    assert_eq!(
        board.occupe_bb & (1u64 << 35),
        0
    );

    assert_eq!(
        board.occupe_bb & (1u64 << 43),
        1u64 << 43
    );

    assert_eq!(board.en_passant_square, None);
}