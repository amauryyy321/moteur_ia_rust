use crate::attack_tables::AttackTables;
use crate::attack_tables::init_attack_tables;
use crate::board::CBoard;
use crate::fen::board_from_fen;
use crate::legal_move::generate_legal_move;
use crate::make_move::make_move;

pub fn perft(board: &mut CBoard, tables: &AttackTables, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = generate_legal_move(board, tables);
    let mut nodes = 0;

    for mv in moves {
        let old_board = board.clone();
        make_move(board, mv);
        nodes += perft(board, tables, depth - 1);
        *board = old_board;
    }
    nodes
}

#[test]
pub fn test_perft_start_pos_from_fen() {
    let tables = init_attack_tables();
    let positions = [(1, 20), (2, 400), (3, 8902), (4, 197281), (5, 4865609)];
    for (depth, expected) in positions {
        let mut board =
            board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let result = perft(&mut board, &tables, depth);
        assert_eq!(
            result, expected,
            "il y a une erreur a la depth numero : {}",
            depth
        );
    }
}
#[test]
pub fn test_perft_kiwipete() {
    let tables = init_attack_tables();
    let positions = [(1, 48), (2, 2039), (3, 97862), (4, 4085603)];
    for (depth, expected) in positions {
        let mut board =
            board_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        let result = perft(&mut board, &tables, depth);
        assert_eq!(
            result, expected,
            "il y a une erreur a la depth numero : {}",
            depth
        );
    }
}
