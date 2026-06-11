use crate::attack_tables::{AttackTables, init_attack_tables};
use crate::board::CBoard;
use crate::chess_move::Move;
use crate::fen::board_from_fen;
use crate::legal_move::generate_legal_move;
use crate::legality::is_king_in_check;
use crate::make_move::make_move;
use crate::position_key::{ClePosition, cle_position};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatPartie {
    EnCours,
    Mat,
    Pat,
    Nulle50Coups,
    NulleRepetition,
}

//structure de la partie

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partie {
    pub board: CBoard,
    pub tables: AttackTables,
    pub coups_joues: Vec<Move>,
    pub repetitions: HashMap<ClePosition, u32>,
}
//permet de creer une partie depuis une fentre et d init les attack_tables
impl Partie {
    pub fn etat_avec_coups(&mut self) -> (EtatPartie, Vec<Move>) {
        if self.board.halfmove_clock >= 100 {
            return (EtatPartie::Nulle50Coups, Vec::new());
        }
        if self
            .repetitions
            .get(&cle_position(&self.board))
            .copied()
            .unwrap_or(0)
            >= 3
        {
            return (EtatPartie::NulleRepetition, Vec::new());
        }

        let coups = generate_legal_move(&mut self.board, &self.tables);
        if !coups.is_empty() {
            return (EtatPartie::EnCours, coups);
        }

        if is_king_in_check(&self.board, &self.tables, self.board.side_to_move) {
            return (EtatPartie::Mat, coups);
        }

        (EtatPartie::Pat, coups)
    }

    pub fn etat(&mut self) -> EtatPartie {
        self.etat_avec_coups().0
    }
    pub fn jouer_coup(&mut self, mv: Move) {
        make_move(&mut self.board, mv);
        self.coups_joues.push(mv);
        let cle = cle_position(&self.board);
        let compteur = self.repetitions.entry(cle).or_insert(0);
        *compteur += 1;
    }
    pub fn depuis_fen(fen: &str) -> Result<Self, String> {
        let board = board_from_fen(fen)?;
        let mut repetitions = HashMap::new();
        let cle = cle_position(&board);
        repetitions.insert(cle, 1);
        Ok(Self {
            board,
            tables: init_attack_tables(),
            coups_joues: Vec::new(),
            repetitions,
        })
    }
}
// permet de jouer un coup et de l ajouet au vecteur de mouvement
