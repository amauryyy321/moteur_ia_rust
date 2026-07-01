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
    score.add(evaluation_structure_pions(board));
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

/* evaluation des structures de pions */

const DOUBLED_PAWN_PENALTY: i32 = 12;
const ISOLATED_PAWN_PENALTY: i32 = 15;
const BACKWARD_PAWN_PENALTY: i32 = 16;
const PASSED_PAWN_PENALTY: [i32;8] = [0,8,15,30,55,90,140,0];

fn file_mask(file: u8) -> u64 {
    0x0101_0101_0101_0101u64 << file
}

fn adjacent_files_mask(file : u8) -> u64{
    let mut mask = 0u64;
    if file > 0 {
        mask |= file_mask(file - 1);
    }
    if file < 7 { 
        mask |= file_mask(file + 1);
    }
    mask
}


fn passed_pawn_mask(square: u8, color: Color) -> u64 {
    let file = square % 8;
    let rank = square / 8;
    let files = file_mask(file) | adjacent_files_mask(file);

    let foward_ranks = match(color){
        Color::Blanc => {
            if rank == 7 {
                0
            }else {
                !0u64 << ((rank + 1)*8)
            }
        }
        Color::Noir => {
            if rank == 0 {
                0
            }else {
                (1u64 << (rank*8))-1
            }
        }
    
        
    };
    files & foward_ranks
}

fn rank_from_color(square : u8, color: Color) -> usize {
    let rank = square / 8;
    match color {
        Color::Blanc => rank as usize,
        Color::Noir => (7-rank) as usize ,
    }
}

fn square_from_file_rank(file : i32,rank : i32) -> Option<u8> {
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Some((rank * 8 + file) as u8)
    }else {
        None
    }
}

fn pawn_attacks_square(enemy_pawn: u64, target: u8, enemy_color : Color) -> bool{
    let file = (target % 8) as i32;
    let rank = ( target / 8) as i32;

    let attackers = match enemy_color {
        Color::Blanc => [(file-1,rank-1),(file+1,rank-1)],
        Color::Noir => [(file-1,rank+1),(file+1,rank+1)],
    };

    attackers.into_iter().filter_map(|(f,r)| square_from_file_rank(f,r)).any(|sq| enemy_pawn & (1u64 << sq) != 0)
}

fn has_friendly_pawn_behind_on_adjacent_file(friendly_pawns: u64, square: u8, color : Color)-> bool {
    let file = (square % 8) as i32;
    let rank = (square / 8) as i32;

    for df in [-1,1] {
        let f = file + df;
        for r in 0..8 {
            let behind = match color {
                Color::Blanc => r <= rank,
                Color::Noir => r >= rank,
            };
            if behind {
                if let Some(sq) = square_from_file_rank(f,r){
                    if friendly_pawns& (1u64 << sq ) != 0 {
                        return true;
                    }
                }
            }
        }
    }
    false
}


fn evaluation_structure_pions_couleur(board: &CBoard, color: Color) -> i32{
    let (friendly_piece,enemy_piece) = match color {
        Color::Blanc => (Pieces::PionBlanc,Pieces::PionNoir),
        Color::Noir => (Pieces::PionNoir,Pieces::PionBlanc),
    };

    let friendly_pawns = board.piece_bb[friendly_piece as usize];
    let enemy_pawns = board.piece_bb[enemy_piece as usize];

    let mut score = 0;

    for file in 0..8{
        let count = (friendly_pawns & file_mask(file)).count_ones() as i32;
        if count > 1 {
            score -= (count - 1) * DOUBLED_PAWN_PENALTY;
        }
    }

    let mut pawns = friendly_pawns;

    while let Some(square) = pop_lsb(&mut pawns) {
        let file = square % 8;

        if friendly_pawns & adjacent_files_mask(file) == 0 {
            score -= ISOLATED_PAWN_PENALTY;
        }

        if enemy_pawns & passed_pawn_mask(square, color) == 0{
            score += PASSED_PAWN_PENALTY[rank_from_color(square,color) as usize];
        }

        let front_square = match color {
            Color::Blanc  if square <= 55 => Some(square + 8),
            Color::Noir  if square >= 8 => Some(square - 8),
            _ => None,
        };

        if let Some(front) = front_square {
            let enemy_color = match color {
                Color::Blanc => Color::Noir,
                Color::Noir => Color::Blanc,
            };

            if !has_friendly_pawn_behind_on_adjacent_file(friendly_pawns,square,color) && pawn_attacks_square(enemy_pawns, front , enemy_color){
                score -= BACKWARD_PAWN_PENALTY;
            }
        }
    }
    score
}

fn evaluation_structure_pions(board: &CBoard) -> EvalScore {
    let white = evaluation_structure_pions_couleur(board, Color::Blanc);
    let black = evaluation_structure_pions_couleur(board, Color::Noir);

    let score = white - black ;

    EvalScore{
        mg : score ,
        eg : score
    }
}