use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use std::time::Instant;

use crate::api_type::{ApiResponse, MoveRequest, NewGameRequest, game_state_from_partie};
use crate::board::{Color, Pieces};
use crate::eval::{meilleur_coup_iterative};
use crate::notation::coord_to_square_index;
use crate::partie::Partie;

const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const AI_DEPTH: u32 = 7;

#[derive(Clone)]
pub struct AppState {
    pub partie: Arc<Mutex<Partie>>,
}

pub async fn run_web_server() {
    let partie = Partie::depuis_fen(INITIAL_FEN).unwrap();

    let state = AppState {
        partie: Arc::new(Mutex::new(partie)),
    };

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/move", post(play_move))
        .route("/api/ai-move", post(play_ai_move))
        .route("/api/new", post(new_game))
        .fallback_service(
            ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html")),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("Serveur web lance sur http://127.0.0.1:3001");

    axum::serve(listener, app).await.unwrap();
}

async fn get_state(State(state): State<AppState>) -> Json<ApiResponse> {
    let mut partie = state.partie.lock().unwrap();

    Json(success_response(&mut partie))
}

async fn play_move(
    State(state): State<AppState>,
    Json(req): Json<MoveRequest>,
) -> Json<ApiResponse> {
    let mut partie = state.partie.lock().unwrap();
    let (etat, legal_moves) = partie.etat_avec_coups();

    if !matches!(etat, crate::partie::EtatPartie::EnCours) {
        let state = game_state_from_partie(&partie, etat, &legal_moves);
        return Json(ApiResponse::error("La partie est deja terminee", state));
    }

    let from = match coord_to_square_index(req.from.trim()) {
        Ok(square) => square,
        Err(err) => {
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            return Json(ApiResponse::error(err, state));
        }
    };

    let to = match coord_to_square_index(req.to.trim()) {
        Ok(square) => square,
        Err(err) => {
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            return Json(ApiResponse::error(err, state));
        }
    };

    let promotion =
        match promotion_from_request(req.promotion.as_deref(), partie.board.side_to_move) {
            Ok(promotion) => promotion,
            Err(err) => {
                let state = game_state_from_partie(&partie, etat, &legal_moves);
                return Json(ApiResponse::error(err, state));
            }
        };

    let selected_move = legal_moves
        .iter()
        .copied()
        .find(|mv| mv.from == from && mv.to == to && mv.promotion == promotion);

    match selected_move {
        Some(mv) => {
            partie.jouer_coup(mv);
            Json(success_response(&mut partie))
        }
        None => {
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            Json(ApiResponse::error(
                "Coup illegal dans cette position",
                state,
            ))
        }
    }
}

async fn play_ai_move(State(state): State<AppState>) -> Json<ApiResponse> {
    let mut partie = state.partie.lock().unwrap();
    let (etat, legal_moves) = partie.etat_avec_coups();

    if !matches!(etat, crate::partie::EtatPartie::EnCours) {
        let state = game_state_from_partie(&partie, etat, &legal_moves);
        return Json(ApiResponse::error("La partie est deja terminee", state));
    }

    match jouer_coup_ia(&mut partie) {
        Ok(()) => Json(success_response(&mut partie)),
        Err(err) => {
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            Json(ApiResponse::error(err, state))
        }
    }
}

async fn new_game(
    State(state): State<AppState>,
    req: Option<Json<NewGameRequest>>,
) -> Json<ApiResponse> {
    let mut partie = state.partie.lock().unwrap();
    let player_color = match couleur_joueur_depuis_requete(
        req.as_ref().and_then(|req| req.player_color.as_deref()),
    ) {
        Ok(color) => color,
        Err(err) => {
            let (etat, legal_moves) = partie.etat_avec_coups();
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            return Json(ApiResponse::error(err, state));
        }
    };

    *partie = Partie::depuis_fen(INITIAL_FEN).unwrap();

    if player_color == Color::Noir {
        if let Err(err) = jouer_coup_ia(&mut partie) {
            let (etat, legal_moves) = partie.etat_avec_coups();
            let state = game_state_from_partie(&partie, etat, &legal_moves);
            return Json(ApiResponse::error(err, state));
        }
    }

    Json(success_response(&mut partie))
}

fn jouer_coup_ia(partie: &mut Partie) -> Result<(), &'static str> {
    let mut board = partie.board;
    let start = Instant::now();
    let Some(mv) = meilleur_coup_iterative(&mut board, &partie.tables, AI_DEPTH) else {
        return Err("Aucun coup trouve pour l'IA");
    };
    let duree = start.elapsed();
    println!("Temps de calcul: {} ms", duree.as_millis());

    partie.jouer_coup(mv);
    Ok(())
}

fn couleur_joueur_depuis_requete(color: Option<&str>) -> Result<Color, &'static str> {
    match color.unwrap_or("Blanc") {
        "Blanc" | "blanc" | "white" | "White" => Ok(Color::Blanc),
        "Noir" | "noir" | "black" | "Black" => Ok(Color::Noir),
        _ => Err("Couleur du joueur invalide"),
    }
}

fn success_response(partie: &mut Partie) -> ApiResponse {
    let (etat, legal_moves) = partie.etat_avec_coups();
    ApiResponse::ok(game_state_from_partie(partie, etat, &legal_moves))
}

fn promotion_from_request(
    promotion: Option<&str>,
    side_to_move: Color,
) -> Result<Option<Pieces>, String> {
    let Some(promotion) = promotion else {
        return Ok(None);
    };

    let promotion = promotion.trim();
    if promotion.is_empty() {
        return Ok(None);
    }

    let piece = match (promotion.to_ascii_lowercase().as_str(), side_to_move) {
        ("q", Color::Blanc) => Pieces::DameBlanche,
        ("r", Color::Blanc) => Pieces::TourBlanche,
        ("b", Color::Blanc) => Pieces::FouBlanc,
        ("n", Color::Blanc) => Pieces::CavalierBlanc,
        ("q", Color::Noir) => Pieces::DameNoire,
        ("r", Color::Noir) => Pieces::TourNoire,
        ("b", Color::Noir) => Pieces::FouNoir,
        ("n", Color::Noir) => Pieces::CavalierNoir,
        _ => return Err("Promotion invalide".to_string()),
    };

    Ok(Some(piece))
}
