use crate::attack_tables::AttackTables;
use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::{Move,MoveFlag};
use crate::legal_move::generate_legal_move;
use crate::legality::is_king_in_check;
use crate::make_move::make_move;


pub fn evaluation_materielle(board: &CBoard)-> i32{
    let valeurs = [(Pieces::PionBlanc, 100),(Pieces::CavalierBlanc, 320),(Pieces::FouBlanc, 330),(Pieces::TourBlanche, 500),(Pieces::DameBlanche, 900),(Pieces::PionNoir, -100),(Pieces::CavalierNoir, -320),(Pieces::FouNoir, -330),(Pieces::TourNoire, -500),(Pieces::DameNoire, -900)];
    let mut score = 0;
    for (pieces,valeur) in valeurs {
        let nombre = board.piece_bb[pieces as usize].count_ones() as i32;
        score += nombre * valeur;
    }
    score
}
pub fn evaluation_negamax(board : &CBoard)->i32{
    let score = evaluation_materielle(board);
    if board.side_to_move == Color::Blanc {
        return score;
    }
    else {
        return -score;
    }
}

pub fn score_ordre_coup(mv: &Move) -> i32{
    match mv.flag {
        MoveFlag::Promotion | MoveFlag::PromotionCapture => 1000,
        MoveFlag::Capture | MoveFlag::EnPassant => 500,
        MoveFlag::Castling => 100,
        _=>0,
    }
}

pub fn minimax(board: &mut CBoard, tables: &AttackTables, depth:u32 ,mut alpha : i32 ,beta : i32)->i32{

    if depth == 0{
        return evaluation_negamax(board);
    }

    let mut moves = generate_legal_move(board, tables);
    moves.sort_by_key(|mv| -score_ordre_coup(mv));


    if moves.is_empty(){
        if is_king_in_check(board,tables,board.side_to_move){
            return -10000;
        }
        return 0;
    }
    let mut meilleure = - 10000;


    for coups in moves{
        let old_board = board.clone();
        make_move(board,coups );
        let score = -minimax(board,tables,depth - 1,-beta,-alpha);
        *board = old_board;
        meilleure = meilleure.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
 
        

    }
    meilleure
}



pub fn meilleur_coup(board : &mut CBoard, tables : &AttackTables, depth:u32)-> Option<Move>{
    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| -score_ordre_coup(mv));
    let mut meilleur_mv = None;
    let mut meilleur_score = -100000;
    let mut alpha = - 100000;
    let mut beta = 100000;

    for mv in coups {
        let old_board = board.clone();
        make_move(board,mv);
        let score = -minimax(board,tables,depth -1,-beta,-alpha);
        *board = old_board;

        if score > meilleur_score{
            meilleur_mv = Some(mv);
            meilleur_score = score;
        }



    }
    meilleur_mv
}
