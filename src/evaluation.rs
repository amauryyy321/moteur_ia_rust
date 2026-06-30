use crate::board::{CBoard, Color, Pieces};
const BONUS_CAVALIER: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BONUS_PION: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 10, 15, 15, 10, 5, 5, 5, 10, 15, 20, 20, 15, 10, 5, 0, 5, 15, 30,
    30, 15, 5, 0, 0, 5, 15, 35, 35, 15, 5, 0, 5, 10, 20, 30, 30, 20, 10, 5, 40, 40, 40, 40, 40, 40,
    40, 40, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub fn evaluation_materielle(board: &CBoard) -> i32 {
    let valeurs = [
        (Pieces::PionBlanc, 100),
        (Pieces::CavalierBlanc, 320),
        (Pieces::FouBlanc, 330),
        (Pieces::TourBlanche, 500),
        (Pieces::DameBlanche, 900),
        (Pieces::PionNoir, -100),
        (Pieces::CavalierNoir, -320),
        (Pieces::FouNoir, -330),
        (Pieces::TourNoire, -500),
        (Pieces::DameNoire, -900),
    ];
    let mut score = 0;
    for (pieces, valeur) in valeurs {
        let nombre = board.piece_bb[pieces as usize].count_ones() as i32;
        score += nombre * valeur;
    }
    score
}
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_blanc(board);
    match board.side_to_move {
        Color::Blanc => score,
        Color::Noir => -score,
    }
}
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);
    score += evaluation_pions(board);
    score
}

fn pop_lsb(bb: &mut u64) -> Option<u8> {
    if *bb == 0 {
        return None;
    }

    let square = bb.trailing_zeros() as u8;
    *bb &= *bb - 1;

    Some(square)
}
// je ne comprend pas cette fonction
fn mirror_square(square: u8) -> u8 {
    square ^ 56
}

fn evaluation_pions(board: &CBoard) -> i32 {
    let mut score = 0;
    let mut pion_blanc = board.piece_bb[Pieces::PionBlanc as usize];

    while let Some(square) = pop_lsb(&mut pion_blanc) {
        score += BONUS_PION[square as usize];
    }
    let mut pion_noir = board.piece_bb[Pieces::PionNoir as usize];

    while let Some(square) = pop_lsb(&mut pion_noir) {
        let mirrored = mirror_square(square);
        score -= BONUS_PION[mirrored as usize];
    }

    score
}

fn evaluation_cavaliers(board: &CBoard) -> i32 {
    let mut score = 0;
    let mut cavaliers_blancs = board.piece_bb[Pieces::CavalierBlanc as usize];

    while let Some(square) = pop_lsb(&mut cavaliers_blancs) {
        score += BONUS_CAVALIER[square as usize];
    }
    let mut cavaliers_noirs = board.piece_bb[Pieces::CavalierNoir as usize];

    while let Some(square) = pop_lsb(&mut cavaliers_noirs) {
        let mirrored = mirror_square(square);
        score -= BONUS_CAVALIER[mirrored as usize];
    }

    score
}

fn evaluation_paire_de_fous(board: &CBoard) -> i32 {
    let mut score = 0;
    let fous_blancs = board.piece_bb[Pieces::FouBlanc as usize].count_ones();
    let fous_noirs = board.piece_bb[Pieces::FouNoir as usize].count_ones();

    if fous_blancs >= 2 {
        score += 30;
    }
    if fous_noirs >= 2 {
        score -= 30;
    }
    score
}

fn evaluation_roque(board: &CBoard) -> i32 {
    let mut score = 0;

    if board.white_king_square == 6 || board.white_king_square == 2 {
        score += 40;
    }
    if board.black_king_square == 62 || board.black_king_square == 58 {
        score -= 40;
    }
    score
}


