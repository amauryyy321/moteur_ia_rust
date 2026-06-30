use crate::attack_tables::AttackTables;
use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::{Move, MoveFlag};
use crate::legal_move::{generate_legal_move, generate_tactical_legal_move};
use crate::legality::is_king_in_check;
use crate::make_move::{make_move, unmake_move};
use crate::position_key::{TTEntry, TTFlag, TranspositionTable, cle_position};
use crate::zobrist::{zobrist_hash,ZobristKeys};
use crate::evaluation::evaluation_negamax;
use std::cmp::Reverse;
use std::time::{Duration, Instant};


const SCORE_MAT: i32 = 100_000;
const INF: i32 = 1_000_000;
const MAX_PLY: usize = 128;
const ASPIRATION_WINDOW: i32 = 50;





#[derive(Default, Debug, Clone)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,
    pub cutoffs: u64,
    pub qcutoffs: u64,
    pub tt_hits: u64,
    pub beta_cutoffs : u64,
    pub beta_cutoffs_first_move: u64,
    pub null_cutoffs: u64,
    pub lmr_researches: u64,
}
#[derive(Clone)]
pub struct SearchHeuristics{
    pub killer_moves: [[Option<Move>; 2]; MAX_PLY],
    pub history : [[i32;64];64],
}
#[derive(Debug, Copy, Clone)]
pub struct SearchResult{
    pub best_move : Option<Move>,
    pub score: i32,
}
impl Default for SearchHeuristics{
    fn default() -> Self{
        Self{
            killer_moves : [[None;2];MAX_PLY],
            history: [[0;64];64]
        }
    }
}

fn update_history(heuristics : &mut SearchHeuristics,mv : Move,depth: u32){
    if !is_quiet_move(&mv){
        return;
    }
    let bonus = (depth * depth) as i32;
    heuristics.history[mv.from as usize][mv.to as usize] += bonus;
}
fn maybe_decay_history(heuristics: &mut SearchHeuristics){
    let max_value = heuristics.history.iter().flatten().copied().max().unwrap_or(0);
    if max_value < 100_000 {
        return;
    }
    for row in heuristics.history.iter_mut(){
        for value in row.iter_mut(){
            *value /=2
        }
    }
}
pub struct SearchLimits {
    pub start: Instant,
    pub max_time: Duration,
}
impl SearchLimits {
    pub fn should_stop(&self) -> bool {
        self.start.elapsed() >= self.max_time
    }
}
fn valeur_piece_abs(piece: Pieces) -> i32 {
    match piece {
        Pieces::PionBlanc | Pieces::PionNoir => 100,
        Pieces::CavalierBlanc | Pieces::CavalierNoir => 320,
        Pieces::FouBlanc | Pieces::FouNoir => 330,
        Pieces::TourBlanche | Pieces::TourNoire => 500,
        Pieces::DameBlanche | Pieces::DameNoire => 900,
        Pieces::RoiBlanc | Pieces::RoiNoir => 20000,
        _ => 0,
    }
}
fn score_capture_mvv_lva(mv : &Move) -> i32{
    let victim = mv.captured.map(valeur_piece_abs).unwrap_or(100);
    let attacker = valeur_piece_abs(mv.piece);
    10 * victim - attacker
}
fn valeur_piece(piece: Pieces) -> i32 {
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
    for (piece_ref, valeur_piece) in valeurs {
        if piece == piece_ref {
            return valeur_piece as i32;
        }
    }
    return 0;
}


fn score_ordre_coup_avec_tt(mv: &Move, tt_best: Option<Move>) -> i32 {
    if Some(*mv) == tt_best {
        return 1_000_000;
    }
    score_ordre_coup(mv)
}
pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::PromotionCapture =>{
            let promotion = mv.promotion.map(valeur_piece_abs).unwrap_or(0);
            20_000 + promotion + score_capture_mvv_lva(mv)
        }

        MoveFlag::Promotion => {
            let promotion = mv.promotion.map(valeur_piece_abs).unwrap_or(0);
            15_000 + promotion
        }
        MoveFlag::Capture | MoveFlag::EnPassant => {
            10_000 + score_capture_mvv_lva(mv)
        }
        MoveFlag::Castling => 100,
        _ => 0,
    }
}
pub fn evaluation_min_max(board: &mut CBoard, tables: &AttackTables, depth: u32) -> i32 {
    if depth == 0 {
        return evaluation_negamax(board);
    }

    let mut moves = generate_legal_move(board, tables);

    if moves.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -SCORE_MAT - depth as i32;
        }
        return 0;
    }
    let mut meilleure = -INF;

    for coups in moves {
        let undo = make_move(board, coups);
        let score = -evaluation_min_max(board, tables, depth - 1);
        unmake_move(board, coups, undo);
        meilleure = meilleure.max(score);
    }
    meilleure
}


pub fn meilleur_coup_iterative(
    board: &mut CBoard,
    tables: &AttackTables,
    max_depth: u32,
) -> Option<Move> {
    let mut best_move = None;
    let mut tt = TranspositionTable::new(64);
    let keys = ZobristKeys::new();

    let mut heuristics = SearchHeuristics::default();

    let limits = SearchLimits {
        start: Instant::now(),
        max_time: Duration::from_millis(2000),
    };

    let mut previous_score = 0;
    let mut windows = 50;

    for depth in 1..=max_depth {

        if limits.should_stop() {
            break;
        }
        let use_aspiration = depth >= 2 && best_move.is_some();
        let mut window = ASPIRATION_WINDOW;
        let mut alpha = if use_aspiration{
            (previous_score-window).max(-INF)
        }else {INF};
        let mut beta = if use_aspiration {
        (previous_score + window).min(INF)
    } else {
        INF
    };

       
        
        loop{
            if limits.should_stop() {
                break;
            }
            let result = meilleur_coup(board,tables,depth, &mut tt, &keys, &limits, best_move, &mut heuristics, alpha,beta,);
            if limits.should_stop(){
                break;
            }
            if result.best_move.is_none(){
                break;
            }
            if use_aspiration && result.score <= alpha{
                alpha = (alpha - window).max(-INF);
                window = (window * 2).min(INF);
                continue;
            }
            if use_aspiration && result.score >= beta{
                beta = (beta + window).min(INF);
                window = (window * 2).min(INF);
                continue;
            }
            previous_score = result.score;
            best_move = result.best_move;
            break;
        }

        println!("deph {} -> {:?}", depth, best_move);
    }
    best_move
}
pub fn is_quiet_move(mv : &Move) -> bool {
    matches!(mv.flag, MoveFlag::Quiet | MoveFlag::DoublePawnPush | MoveFlag::Castling)
}
pub fn score_killer_move(heuristics: &mut SearchHeuristics, ply: usize, mv: Move){
    if ply >= MAX_PLY{
        return;
    }
    if heuristics.killer_moves[ply][0] == Some(mv){
        return;
    }
    heuristics.killer_moves[ply][1] = heuristics.killer_moves[ply][0];
    heuristics.killer_moves[ply][0] = Some(mv);
}
pub fn evaluation_negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    tt: &mut TranspositionTable,
    keys : &ZobristKeys,
    limits: &SearchLimits,
    ply : usize,
    heuristics : &mut SearchHeuristics
) -> i32 {
    stats.nodes += 1;
    if limits.should_stop() {
        return evaluation_negamax(board);
    }
    let original_alpha = alpha;
    let key = zobrist_hash(board,keys);
    let mut meilleure = -INF;
    let mut meilleur_mv = None;

    if let Some(entry) = tt.get(key) {
        if entry.depth >= depth {
            match entry.flag {
                TTFlag::Exact => return entry.score,
                TTFlag::LowerBound => alpha = alpha.max(entry.score),
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        return entry.score;
                    }
                }
            }
        }
        if alpha >= beta {
            return entry.score;
        }
    }

    if depth == 0 {
        return quiescence(board, tables, alpha, beta, 4, stats, limits);
    }

    let mut moves = generate_legal_move(board, tables);
    let tt_best = tt.get(key).and_then(|entry| entry.best_move);
    moves.sort_by_key(|mv| Reverse(score_search_move(mv,tt_best,heuristics,ply)));

    if moves.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -SCORE_MAT - depth as i32;
        }
        return 0;
    }

    for coups in moves {
        let undo = make_move(board, coups);
        let score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
            stats,
            tt,
            keys,
            limits,
            ply +1,
            heuristics,
        );
        unmake_move(board, coups, undo);
        if score > meilleure {
            meilleure = score;
            meilleur_mv = Some(coups);
        }

        alpha = alpha.max(score);

        if alpha >= beta {
            if is_quiet_move(&coups){
            score_killer_move(heuristics, ply, coups);
            }

            stats.cutoffs += 1;
            break;
        }
    }
    let flag = if meilleure <= original_alpha {
        TTFlag::UpperBound
    } else if meilleure >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };
    tt.insert(
        TTEntry {
            key,
            depth,
            score: meilleure,
            flag,
            best_move: meilleur_mv,
        },
    );
    meilleure
}
fn score_root_move(mv: &Move, previous_best : Option<Move>,tt_best : Option<Move>) ->i32{
    if Some(*mv) == previous_best{
        return 2_000_000;
    }
    if Some(*mv) == tt_best{
        return 1_000_000;
    }
    score_ordre_coup(mv)
}
fn score_search_move(mv : &Move,tt_best: Option<Move>,heuristics: &SearchHeuristics,ply: usize)->i32{
    if Some (*mv) == tt_best {
        return 2_000_000;
    }
    if is_quiet_move(mv){
        let history_score = heuristics.history[mv.from as usize][mv.to as usize];
        return history_score.min(700_000);
    }
    if ply < MAX_PLY{
        if heuristics.killer_moves[ply][0] == Some (*mv){
            return 900_000;
        }
        if heuristics.killer_moves[ply][1] == Some (*mv){
            return 800_000;
        }

    }
    score_ordre_coup(mv)
}
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    tt: &mut TranspositionTable,
    keys : &ZobristKeys,
    limits: &SearchLimits,
    previous_best : Option<Move>,
    heuristics : &mut SearchHeuristics,
    root_alpha : i32,
    root_beta : i32,
) -> SearchResult {
    let mut stats = SearchStats::default();
    let start = Instant::now();

    let mut coups = generate_legal_move(board, tables);

    let key = zobrist_hash(board,keys);
    let tt_best = tt.get(key).and_then(|entry| entry.best_move);
    coups.sort_by_key(|mv| Reverse(score_root_move(mv,previous_best, tt_best)));
    let mut meilleur_mv = None;
    let mut meilleur_score = -INF;
    let mut alpha = root_alpha;
    let beta = root_beta;
    if depth == 0 {
        return SearchResult{
            best_move : None,
            score : evaluation_negamax(board),
        };
    }

    if coups.is_empty(){
        let score = if is_king_in_check(board,tables,board.side_to_move){
            -SCORE_MAT
        }
        else{
            0
        };
        return return SearchResult{
            best_move : None,
            score,
        };
    }

    for mv in coups {
        if limits.should_stop() {
            break;
        }
        let undo = make_move(board, mv);
        let score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
            &mut stats,
            tt,
            keys,
            limits,
            1,
            heuristics,
        );
        unmake_move(board, mv, undo);

        if score > meilleur_score {
            meilleur_mv = Some(mv);
            meilleur_score = score;
        }
        alpha = alpha.max(score);
        if alpha >= beta{
            break;
        }
    }

    let elapsed = start.elapsed();
    
    println!("Nodes : {}", stats.nodes);
    println!("QNodes : {}", stats.qnodes);
    println!("Cutoffs : {}", stats.cutoffs);
    println!("QCutoffs : {}", stats.qcutoffs);
    SearchResult{
        best_move : meilleur_mv,
        score: meilleur_score,
    }
}
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
    qdepth: u32,
    stats: &mut SearchStats,
    limits: &SearchLimits,
) -> i32 {
    stats.qnodes += 1;
    if limits.should_stop() {
        return evaluation_negamax(board);
    }
    let in_check = is_king_in_check(board, tables, board.side_to_move);

    if qdepth == 0 {
        return evaluation_negamax(board);
    }
    if !in_check {
        let stand_pat = evaluation_negamax(board);

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }
    }

    let mut moves = generate_tactical_legal_move(board, tables);

    if moves.is_empty() {
        if in_check {
            return -SCORE_MAT;
        }
    }

    moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    for mv in moves {
        let undo = make_move(board, mv);
        let score = -quiescence(board, tables, -beta, -alpha, qdepth - 1, stats, limits);

        unmake_move(board, mv, undo);

        if score >= beta {
            stats.qcutoffs += 1;
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

