use serde::{Deserialize, Serialize};

use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::Move;
use crate::notation::move_to_coord;
use crate::partie::{EtatPartie, Partie};

#[derive(Serialize)]
pub struct ApiResponse {
    pub ok: bool,
    pub error: Option<String>,
    pub state: GameStateDto,
}

impl ApiResponse {
    pub fn ok(state: GameStateDto) -> Self {
        Self {
            ok: true,
            error: None,
            state,
        }
    }

    pub fn error(message: impl Into<String>, state: GameStateDto) -> Self {
        Self {
            ok: false,
            error: Some(message.into()),
            state,
        }
    }
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

#[derive(Deserialize)]
pub struct NewGameRequest {
    pub player_color: Option<String>,
}

#[derive(Serialize)]
pub struct GameStateDto {
    pub status: String,
    pub side_to_move: String,
    pub board: Vec<Option<PieceDto>>,
    pub legal_moves: Vec<MoveDto>,
    pub move_history: Vec<String>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[derive(Serialize)]
pub struct PieceDto {
    pub code: String,
    pub name: String,
    pub color: String,
    pub symbol: String,
}

#[derive(Serialize)]
pub struct MoveDto {
    pub from: u8,
    pub to: u8,
    pub from_coord: String,
    pub to_coord: String,
    pub promotion: Option<String>,
    pub notation: String,
}

pub fn game_state_from_partie(
    partie: &Partie,
    etat: EtatPartie,
    legal_moves: &[Move],
) -> GameStateDto {
    GameStateDto {
        status: status_to_string(etat),
        side_to_move: color_to_string(partie.board.side_to_move),
        board: board_to_dto(&partie.board),
        legal_moves: legal_moves.iter().map(move_to_dto).collect(),
        move_history: partie.coups_joues.iter().map(move_to_coord).collect(),
        halfmove_clock: partie.board.halfmove_clock,
        fullmove_number: partie.board.fullmove_number,
    }
}

fn board_to_dto(board: &CBoard) -> Vec<Option<PieceDto>> {
    (0..64)
        .map(|square| piece_on_square(board, square).map(piece_to_dto))
        .collect()
}

fn move_to_dto(mv: &Move) -> MoveDto {
    MoveDto {
        from: mv.from,
        to: mv.to,
        from_coord: square_index_to_coord(mv.from),
        to_coord: square_index_to_coord(mv.to),
        promotion: mv.promotion.map(promotion_to_string),
        notation: move_to_coord(mv),
    }
}

fn piece_on_square(board: &CBoard, square: u8) -> Option<Pieces> {
    let square_bb = 1u64 << square;
    [
        Pieces::PionBlanc,
        Pieces::PionNoir,
        Pieces::CavalierBlanc,
        Pieces::CavalierNoir,
        Pieces::FouBlanc,
        Pieces::FouNoir,
        Pieces::TourBlanche,
        Pieces::TourNoire,
        Pieces::DameBlanche,
        Pieces::DameNoire,
        Pieces::RoiBlanc,
        Pieces::RoiNoir,
    ]
    .into_iter()
    .find(|piece| board.piece_bb[*piece as usize] & square_bb != 0)
}

fn piece_to_dto(piece: Pieces) -> PieceDto {
    let (code, name, color, symbol) = match piece {
        Pieces::PionBlanc => ("P", "Pion", Color::Blanc, "P"),
        Pieces::PionNoir => ("p", "Pion", Color::Noir, "p"),
        Pieces::CavalierBlanc => ("N", "Cavalier", Color::Blanc, "N"),
        Pieces::CavalierNoir => ("n", "Cavalier", Color::Noir, "n"),
        Pieces::FouBlanc => ("B", "Fou", Color::Blanc, "B"),
        Pieces::FouNoir => ("b", "Fou", Color::Noir, "b"),
        Pieces::TourBlanche => ("R", "Tour", Color::Blanc, "R"),
        Pieces::TourNoire => ("r", "Tour", Color::Noir, "r"),
        Pieces::DameBlanche => ("Q", "Dame", Color::Blanc, "Q"),
        Pieces::DameNoire => ("q", "Dame", Color::Noir, "q"),
        Pieces::RoiBlanc => ("K", "Roi", Color::Blanc, "K"),
        Pieces::RoiNoir => ("k", "Roi", Color::Noir, "k"),
        Pieces::PiecesBlanches | Pieces::PiecesNoires => unreachable!(),
    };

    PieceDto {
        code: code.to_string(),
        name: name.to_string(),
        color: color_to_string(color),
        symbol: symbol.to_string(),
    }
}

fn status_to_string(etat: EtatPartie) -> String {
    match etat {
        EtatPartie::EnCours => "EnCours",
        EtatPartie::Mat => "Mat",
        EtatPartie::Pat => "Pat",
        EtatPartie::Nulle50Coups => "Nulle50Coups",
        EtatPartie::NulleRepetition => "NulleRepetition",
    }
    .to_string()
}

fn color_to_string(color: Color) -> String {
    match color {
        Color::Blanc => "Blanc",
        Color::Noir => "Noir",
    }
    .to_string()
}

fn promotion_to_string(piece: Pieces) -> String {
    match piece {
        Pieces::DameBlanche | Pieces::DameNoire => "q",
        Pieces::TourBlanche | Pieces::TourNoire => "r",
        Pieces::FouBlanc | Pieces::FouNoir => "b",
        Pieces::CavalierBlanc | Pieces::CavalierNoir => "n",
        _ => "",
    }
    .to_string()
}

fn square_index_to_coord(square: u8) -> String {
    let file = (b'a' + square % 8) as char;
    let rank = square / 8 + 1;
    format!("{}{}", file, rank)
}
