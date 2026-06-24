use moteur_ia::attack_tables::init_attack_tables;
use moteur_ia::board::CBoard;
use moteur_ia::fen::board_from_fen;
use moteur_ia::legal_move::generate_legal_move;
use moteur_ia::make_move::{make_move, unmake_move};

fn assert_make_unmake_restores_all_legal_moves(mut board: CBoard) {
    let tables = init_attack_tables();
    let moves = generate_legal_move(&mut board, &tables);

    for mv in moves {
        let before = board;
        let undo = make_move(&mut board, mv);
        unmake_move(&mut board, mv, undo);

        assert_eq!(board, before, "make/unmake ne restaure pas {:?}", mv);
    }
}

fn assert_make_unmake_restores_two_plies(mut board: CBoard) {
    let tables = init_attack_tables();
    let first_moves = generate_legal_move(&mut board, &tables);

    for first_mv in first_moves {
        let before_first = board;
        let first_undo = make_move(&mut board, first_mv);
        let second_moves = generate_legal_move(&mut board, &tables);

        for second_mv in second_moves {
            let before_second = board;
            let second_undo = make_move(&mut board, second_mv);
            unmake_move(&mut board, second_mv, second_undo);

            assert_eq!(
                board, before_second,
                "make/unmake ne restaure pas le deuxieme coup {:?} apres {:?}",
                second_mv, first_mv
            );
        }

        unmake_move(&mut board, first_mv, first_undo);

        assert_eq!(
            board, before_first,
            "make/unmake ne restaure pas le premier coup {:?}",
            first_mv
        );
    }
}

#[test]
fn make_unmake_restores_start_position() {
    assert_make_unmake_restores_all_legal_moves(CBoard::init_position_depart());
}

#[test]
fn make_unmake_restores_special_moves() {
    let positions = [
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "7k/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/p7/4K3 b - - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ];

    for fen in positions {
        let board = board_from_fen(fen).unwrap();
        assert_make_unmake_restores_all_legal_moves(board);
    }
}

#[test]
fn make_unmake_restores_nested_moves() {
    let board =
        board_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

    assert_make_unmake_restores_two_plies(board);
}
