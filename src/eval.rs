    use crate::attack_tables::AttackTables;
    use crate::board::{CBoard, Color, Pieces};
    use crate::chess_move::{Move,MoveFlag};
    use crate::legal_move::{generate_legal_move,generate_tactical_legal_move};
    use crate::legality::is_king_in_check;
    use crate::make_move::make_move;
    use crate::fen::board_from_fen;
    use crate::position_key::{TranspositionTable,cle_position,TTFlag,TTEntry};
    use std::cmp::Reverse;
    use std::time::Instant;
    const SCORE_MAT : i32 = 100_000;
    const INF : i32 = 1_000_000;
    const BONUS_CAVALIER: [i32; 64]= [
        -50, -40, -30, -30, -30, -30, -40, -50,
        -40, -20,   0,   0,   0,   0, -20, -40,
        -30,   0,  10,  15,  15,  10,   0, -30,
        -30,   5,  15,  20,  20,  15,   5, -30,
        -30,   0,  15,  20,  20,  15,   0, -30,
        -30,   5,  10,  15,  15,  10,   5, -30,
        -40, -20,   0,   5,   5,   0, -20, -40,
        -50, -40, -30, -30, -30, -30, -40, -50,
    ];


     const BONUS_PION: [i32; 64]= [
        100, 100, 100, 100, 100, 100, 100, 100,
        80, 80,   80,   80,   80,   80, 80, 80,
        40,   05,  45,  45,  45,  45,   40, 40,
        15,   15,  15,  20,  20,  20,   15, 15,
        10,   10,  20,  20,  20,  20,   10, 10,
        5,   5,  10,  15,  15,  10,   5, 5,
        0,0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[derive(Default, Debug, Clone)]
    pub struct SearchStats{
        pub nodes : u64,
        pub qnodes: u64,
        pub cutoffs: u64,
        pub qcutoffs: u64,
    }
    fn valeur_piece_abs(piece: Pieces) ->i32{
        match piece{
            Pieces::PionBlanc| Pieces::PionNoir => 100,
            Pieces::CavalierBlanc | Pieces::CavalierNoir => 320,
            Pieces::FouBlanc | Pieces::FouNoir => 330,
            Pieces::TourBlanche | Pieces::TourNoire => 500,
            Pieces::DameBlanche |Pieces::DameNoire => 900,
            Pieces::RoiBlanc | Pieces::RoiNoir => 20000,
            _=> 0,
        }
    }
    fn valeur_piece(piece: Pieces)-> i32{
        let valeurs = [(Pieces::PionBlanc, 100),(Pieces::CavalierBlanc, 320),(Pieces::FouBlanc, 330),(Pieces::TourBlanche, 500),(Pieces::DameBlanche, 900),(Pieces::PionNoir, -100),(Pieces::CavalierNoir, -320),(Pieces::FouNoir, -330),(Pieces::TourNoire, -500),(Pieces::DameNoire, -900)];
        for (piece_ref,valeur_piece) in valeurs{
            if piece == piece_ref {
                return valeur_piece as i32;
            }
        }
        return 0;
    }
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
        let score = evaluation_blanc(board);
        match board.side_to_move{
            Color::Blanc => score,
            Color::Noir => -score,
        }
    }
    fn score_ordre_coup_avec_tt(mv: &Move, tt_best: Option<Move>)->i32{
        if Some(*mv) == tt_best {
            return 1_000_000;
        }
        score_ordre_coup(mv)
    }
    pub fn score_ordre_coup(mv: &Move) -> i32{
        
        match mv.flag {
            MoveFlag::Promotion | MoveFlag::PromotionCapture => {
                let mut score = 8000;
                if let Some(promotion) = mv.promotion{
                    score += valeur_piece_abs(promotion);
                }
                if let Some (piece_capturee) = mv.captured  {
                    score += 10 * valeur_piece_abs(piece_capturee)-valeur_piece_abs(mv.piece);
                }
                score
            },
            MoveFlag::Capture | MoveFlag::EnPassant => {


                let valeur_capture = match mv.captured{
                    Some(piece)=>valeur_piece_abs(piece),
                    None => 100,
                };
                let valeur_attaquante = valeur_piece_abs(mv.piece);
                1000+10*valeur_capture-valeur_attaquante
            }
            MoveFlag::Castling => 100,
            _=>0,
        }
    }
    pub fn evaluation_min_max(board: &mut CBoard, tables: &AttackTables, depth:u32 )->i32{

        if depth == 0{
            return evaluation_negamax(board);
        }

        let mut moves = generate_legal_move(board, tables);

        if moves.is_empty(){
            if is_king_in_check(board,tables,board.side_to_move){
                return -SCORE_MAT + depth as i32;
            }
            return 0;
        }
        let mut meilleure = - INF;


        for coups in moves{
            let old_board = board.clone();
            make_move(board,coups );
            let score = -evaluation_min_max(board,tables,depth - 1);
            *board = old_board;
            meilleure = meilleure.max(score);    

        }
        meilleure
    }

    pub fn meilleur_coup_iterative(board: &mut CBoard,tables : &AttackTables,max_depth: u32)-> Option<Move>{
        let mut best_move = None;
        let mut tt = TranspositionTable::new();

        for depth in 1..=max_depth{
            let mv = meilleur_coup(board,tables,depth,&mut tt);
            if mv.is_some(){
                best_move = mv;
            }
            println!("deph {} -> {:?}",depth,best_move);
        }
        best_move
    }


    pub fn evaluation_negamax_alpha_beta(board: &mut CBoard, tables: &AttackTables, depth:u32 ,mut alpha : i32 ,beta : i32,stats: &mut SearchStats,tt : &mut TranspositionTable)->i32{
        stats.nodes +=1;
        let original_alpha = alpha;
        let key = cle_position(board);
        let mut meilleure = -INF;
        let mut meilleur_mv = None;


        if let Some(entry) = tt.get(&key) {
            if entry.depth >= depth{
                match entry.flag {
                    TTFlag::Exact => return entry.score,
                    TTFlag::LowerBound => alpha = alpha.max(entry.score),
                    TTFlag::UpperBound =>{
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



        if depth == 0{
            return quiescence(board,tables,alpha,beta,0, stats);
        }

        let mut moves = generate_legal_move(board, tables);
        let tt_best = tt.get(&key).and_then(|entry| entry.best_move);
        moves.sort_by_key(|mv| Reverse(score_ordre_coup_avec_tt(mv, tt_best)));



        if moves.is_empty(){
            if is_king_in_check(board,tables,board.side_to_move){
                return -SCORE_MAT + depth as i32;
            }
            return 0;
        }


        for coups in moves{
            let old_board = board.clone();
            make_move(board,coups );
            let score = -evaluation_negamax_alpha_beta(board,tables,depth - 1,-beta,-alpha,stats,tt);
            *board = old_board;
            if score >meilleure{
                meilleure = meilleure.max(score);
                meilleur_mv = Some(coups);
            }
            
            alpha = alpha.max(score);

            if alpha >= beta {
                stats.cutoffs+=1;
                break;
            }

    
            

        }
        let flag = if  meilleure <= original_alpha{TTFlag::UpperBound}else if meilleure >= beta {TTFlag::LowerBound}else {TTFlag::Exact};
        tt.insert(key,TTEntry{depth,score : meilleure,flag,best_move:meilleur_mv});
        meilleure
    }



    pub fn meilleur_coup(board : &mut CBoard, tables : &AttackTables, depth:u32 ,tt :&mut TranspositionTable)-> Option<Move>{

        let mut stats = SearchStats::default();
        let start = Instant::now();


        let mut coups = generate_legal_move(board, tables);

        let key = cle_position(board);
        let tt_best = tt.get(&key).and_then(|entry| entry.best_move);
        coups.sort_by_key(|mv| Reverse(score_ordre_coup_avec_tt(mv, tt_best)));
        let mut meilleur_mv = None;
        let mut meilleur_score = -INF;
        let mut alpha = - INF;
        let beta = INF;
        if depth == 0 {
            return None;
        }

        for mv in coups {
            let old_board = board.clone();
            make_move(board,mv);
            let score = -evaluation_negamax_alpha_beta(board,tables,depth -1,-beta,-alpha,&mut stats,tt);
            *board = old_board;

            if score > meilleur_score{
                meilleur_mv = Some(mv);
                meilleur_score = score;
                
            }
            alpha = alpha.max(score);



        }
        
        let elapsed = start.elapsed();
        println!("Temps : {}",elapsed.as_millis());
        println!("Nodes : {}",stats.nodes);
        println!("QNodes : {}",stats.qnodes);
        println!("Cutoffs : {}",stats.cutoffs);
        println!("QCutoffs : {}",stats.qcutoffs);
        meilleur_mv
    }


    pub fn evaluation_blanc(board: &CBoard)->i32{
        let mut score = 0;

        score += evaluation_materielle(board);
        score += evaluation_cavaliers(board);
        score += evaluation_paire_de_fous(board);
        score += evaluation_roque(board);
        score += evaluation_pions(board);
        score
    }



    fn pop_lsb(bb: &mut u64)-> Option<u8>{
        if *bb == 0 {
            return None;
        }

        let square = bb.trailing_zeros() as u8;
        *bb &= *bb -1;

        Some(square)
    }

    // je ne comprend pas cette fonction 
    fn mirror_square(square: u8)->u8{
        square ^ 56
    }
    fn evaluation_pions(board: &CBoard) ->  i32 {
        let mut score = 0;
        let mut pion_blanc = board.piece_bb[Pieces::PionBlanc as usize];

        while let Some(square) = pop_lsb(&mut pion_blanc){
            score += BONUS_PION[square as usize];
        }
        let mut pion_noir = board.piece_bb[Pieces::PionNoir as usize];

        while let Some(square) = pop_lsb(&mut pion_noir){
            let mirrored = mirror_square(square);
            score -= BONUS_PION[mirrored as usize];
        }

        score
    }

    fn evaluation_cavaliers(board: &CBoard) ->  i32 {
        let mut score = 0;
        let mut cavaliers_blancs = board.piece_bb[Pieces::CavalierBlanc as usize];

        while let Some(square) = pop_lsb(&mut cavaliers_blancs){
            score += BONUS_CAVALIER[square as usize];
        }
        let mut cavaliers_noirs = board.piece_bb[Pieces::CavalierNoir as usize];

        while let Some(square) = pop_lsb(&mut cavaliers_noirs){
            let mirrored = mirror_square(square);
            score -= BONUS_CAVALIER[mirrored as usize];
        }

        score
    }

    fn evaluation_paire_de_fous(board: &CBoard) -> i32 {
        let mut score = 0;
        let fous_blancs = board.piece_bb[Pieces::FouBlanc as usize].count_ones();
        let fous_noirs = board.piece_bb[Pieces::FouNoir as usize].count_ones();

        if fous_blancs >= 2{
            score +=30;
        }
        if fous_noirs >= 2{
            score -=30;
        }
        score
    }

    fn evaluation_roque(board: &CBoard)-> i32{
        let mut score = 0;

        if board.white_king_square == 6 || board.white_king_square == 2 {
            score += 40;
        }
        if board.black_king_square == 62 || board.black_king_square == 58 {
            score -=40;
        }
        score
    }
    #[test]
    fn cavalier_centre_meilleur_que_cavalier_bord() {
        let board_bord = board_from_fen(
            "7k/8/8/8/8/8/N7/K7 w - - 0 1"
        ).unwrap();

        let board_centre = board_from_fen(
            "7k/8/8/8/3N4/8/8/K7 w - - 0 1"
        ).unwrap();

        assert!(evaluation_blanc(&board_centre) > evaluation_blanc(&board_bord));
    }
    pub fn quiescence(board: &mut CBoard,tables: &AttackTables,mut alpha: i32,beta: i32 ,qdepth : u32, stats : &mut SearchStats)->i32{
        stats.qnodes +=1;
        let in_check = is_king_in_check(board,tables,board.side_to_move);

        
        if qdepth == 0{
            return evaluation_negamax(board);
        }
        if !in_check{
            let stand_pat = evaluation_negamax(board);

            if stand_pat >= beta {
                return beta;
            }

            if stand_pat > alpha{
                alpha = stand_pat;
            }
        }

        let mut moves = generate_tactical_legal_move(board,tables);
        

        if moves.is_empty(){
            if in_check{
                return -SCORE_MAT;
            }
        }

        moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

        for mv in moves {
            let old_board = board.clone();
            make_move(board,mv);
            let score = -quiescence(board,tables,-beta,-alpha,qdepth-1,stats);

            *board = old_board;

            if score >= beta{
                stats.qcutoffs +=1;
                return beta;
            }

            if score > alpha{
                alpha = score;
            }
        }
        alpha
    }
    

    #[test]
    fn paire_de_fous_donne_bonus_aux_blancs() {
        let board_un_fou = board_from_fen(
            "7k/8/8/8/8/8/8/K2B4 w - - 0 1"
        ).unwrap();

        let board_deux_fous = board_from_fen(
            "7k/8/8/8/8/8/8/K2BB3 w - - 0 1"
        ).unwrap();

        assert!(evaluation_blanc(&board_deux_fous) > evaluation_blanc(&board_un_fou));
    }

    #[test]
    fn roque_blanc_donne_bonus() {
        let board_non_roque = board_from_fen(
            "7k/8/8/8/8/8/8/R3K2R w KQ - 0 1"
        ).unwrap();

        let board_roque = board_from_fen(
            "7k/8/8/8/8/8/8/R4RK1 w - - 0 1"
        ).unwrap();

        assert!(evaluation_blanc(&board_roque) > evaluation_blanc(&board_non_roque));
    }