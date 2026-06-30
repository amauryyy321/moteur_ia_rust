use crate::board::{CBoard, Color, Pieces};
const PAWN_MG: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 5, 10, 15, 15, 10, 5, 5,
    5, 10, 15, 25, 25, 15, 10, 5,
    0, 5, 15, 30, 30, 15, 5, 0,
    0, 5, 15, 35, 35, 15, 5, 0,
    5, 10, 20, 30, 30, 20, 10, 5,
    40, 40, 40, 40, 40, 40, 40, 40,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const PAWN_EG: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 5, 5, 10, 10, 5, 5, 5,
    10, 10, 15, 20, 20, 15, 10, 10,
    20, 20, 25, 35, 35, 25, 20, 20,
    35, 35, 40, 55, 55, 40, 35, 35,
    60, 60, 65, 75, 75, 65, 60, 60,
    100, 100, 100, 100, 100, 100, 100, 100,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_MG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 10, 20, 30, 30, 20, 10, -30,
    -30, 5, 20, 30, 30, 20, 5, -30,
    -30, 10, 15, 20, 20, 15, 10, -30,
    -40, -20, 0, 10, 10, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const KNIGHT_EG: [i32; 64] = [
    -40, -30, -20, -20, -20, -20, -30, -40,
    -30, -10, 0, 0, 0, 0, -10, -30,
    -20, 0, 10, 15, 15, 10, 0, -20,
    -20, 5, 15, 20, 20, 15, 5, -20,
    -20, 5, 15, 20, 20, 15, 5, -20,
    -20, 0, 10, 15, 15, 10, 0, -20,
    -30, -10, 0, 0, 0, 0, -10, -30,
    -40, -30, -20, -20, -20, -20, -30, -40,
];

const BISHOP_MG: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 0, 10, 15, 15, 10, 0, -10,
    -10, 5, 10, 15, 15, 10, 5, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const BISHOP_EG: [i32; 64] = [
    -10, -5, -5, -5, -5, -5, -5, -10,
    -5, 5, 5, 5, 5, 5, 5, -5,
    -5, 5, 10, 10, 10, 10, 5, -5,
    -5, 5, 10, 15, 15, 10, 5, -5,
    -5, 5, 10, 15, 15, 10, 5, -5,
    -5, 5, 10, 10, 10, 10, 5, -5,
    -5, 5, 5, 5, 5, 5, 5, -5,
    -10, -5, -5, -5, -5, -5, -5, -10,
];

const ROOK_MG: [i32; 64] = [
    0, 0, 5, 10, 10, 5, 0, 0,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    5, 10, 10, 10, 10, 10, 10, 5,
    0, 0, 5, 10, 10, 5, 0, 0,
];

const ROOK_EG: [i32; 64] = [
    0, 0, 5, 5, 5, 5, 0, 0,
    0, 0, 5, 5, 5, 5, 0, 0,
    0, 0, 5, 5, 5, 5, 0, 0,
    0, 0, 5, 5, 5, 5, 0, 0,
    0, 0, 5, 5, 5, 5, 0, 0,
    0, 0, 5, 5, 5, 5, 0, 0,
    5, 10, 10, 10, 10, 10, 10, 5,
    0, 0, 5, 5, 5, 5, 0, 0,
];

const QUEEN_MG: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    0, 0, 5, 5, 5, 5, 0, -5,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

const QUEEN_EG: [i32; 64] = [
    -10, -5, -5, -5, -5, -5, -5, -10,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 5, 5, 5, 5, 0, -5,
    -5, 0, 5, 10, 10, 5, 0, -5,
    -5, 0, 5, 10, 10, 5, 0, -5,
    -5, 0, 5, 5, 5, 5, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -10, -5, -5, -5, -5, -5, -5, -10,
];

const KING_MG: [i32; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20,
    10, 20, 0, 0, 0, 0, 20, 10,
    -10, -20, -20, -20, -20, -20, -20, -10,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -40, -50, -50, -60, -60, -50, -50, -40,
    -50, -60, -60, -70, -70, -60, -60, -50,
    -60, -70, -70, -80, -80, -70, -70, -60,
];

const KING_EG: [i32; 64] = [
    -50, -30, -20, -10, -10, -20, -30, -50,
    -30, -10, 0, 10, 10, 0, -10, -30,
    -20, 0, 20, 25, 25, 20, 0, -20,
    -10, 10, 25, 35, 35, 25, 10, -10,
    -10, 10, 25, 35, 35, 25, 10, -10,
    -20, 0, 20, 25, 25, 20, 0, -20,
    -30, -10, 0, 10, 10, 0, -10, -30,
    -50, -30, -20, -10, -10, -20, -30, -50,
];

const MAX_PHASE : i32 = 24;


#[derive(Default,Clone,Copy)]
struct EvalScore {
    mg: i32,
    eg: i32,
}
impl EvalScore{
    fn add(&mut self, other: EvalScore){
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

fn blend_score(score: EvalScore, phase: i32)-> i32{
    (score.mg * phase + score.eg * (MAX_PHASE - phase)) /MAX_PHASE
}

fn score_piece_square(board : &CBoard,white_piece: Pieces,black_piece : Pieces,mg_table: &[i32; 64],eg_table : &[i32;64])->EvalScore{
    let mut score = EvalScore::default();

    let mut white = board.piece_bb[white_piece as usize];

    while let Some(square) = pop_lsb(&mut white) {
        score.mg += mg_table[square as usize];
        score.eg += eg_table[square as usize];
    }

    let mut black = board.piece_bb[black_piece as usize];

    while let Some(square) = pop_lsb(&mut black) {
        let mirrored = mirror_square(square);
        score.mg -= mg_table[square as usize];
        score.eg -= eg_table[square as usize];
    }

    score
}
fn evaluation_tables_de_cases(board: &CBoard) -> EvalScore{
    let mut score = EvalScore::default();

    score.add(score_piece_square(board, Pieces::PionBlanc, Pieces::PionNoir,&PAWN_MG, &PAWN_EG));
    score.add(score_piece_square(board, Pieces::CavalierBlanc, Pieces::CavalierNoir,&KNIGHT_MG, &KNIGHT_EG));
    score.add(score_piece_square(board, Pieces::FouBlanc, Pieces::FouNoir,&BISHOP_MG, &BISHOP_EG));
    score.add(score_piece_square(board, Pieces::TourBlanche, Pieces::TourNoire,&ROOK_MG, &ROOK_EG));
    score.add(score_piece_square(board, Pieces::DameBlanche, Pieces::DameNoire,&QUEEN_MG, &QUEEN_EG));
    score.add(score_piece_square(board, Pieces::RoiBlanc, Pieces::RoiNoir,&KING_MG, &KING_EG));

    score
}
fn game_phase(board : &CBoard) -> i32{
    let phase = board.piece_bb[Pieces::CavalierBlanc as usize].count_ones() as i32 
        + board.piece_bb[Pieces::CavalierNoir as usize].count_ones() as i32
        + board.piece_bb[Pieces::FouBlanc as usize].count_ones() as i32
        + board.piece_bb[Pieces::FouNoir as usize].count_ones() as i32
        + 2 * board.piece_bb[Pieces::TourBlanche as usize].count_ones() as i32
        + 2 * board.piece_bb[Pieces::TourNoire as usize].count_ones() as i32
        + 4 * board.piece_bb[Pieces::DameBlanche as usize].count_ones() as i32
        + 4 * board.piece_bb[Pieces::DameNoire as usize].count_ones() as i32;
    phase.clamp(0,MAX_PHASE)

}
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
    let phase = game_phase(board);
    let materiel = evaluation_materielle(board);


    let mut score = EvalScore{
        mg : materiel,
        eg : materiel,
    };

    score.add(evaluation_tables_de_cases(board));
    score.add(EvalScore{
        mg: evaluation_paire_de_fous(board),
        eg: evaluation_paire_de_fous(board),
    });



    //score += evaluation_roque(board);
    blend_score(score, phase)
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


