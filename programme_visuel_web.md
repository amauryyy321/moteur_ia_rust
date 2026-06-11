# Programme visuel web — interface propre pour le moteur d'échecs Rust

Objectif : transformer le moteur d'échecs Rust actuel en application web locale propre, lisible et qualitative, sans mélanger la logique métier du moteur avec l'interface graphique.

Le moteur doit rester dans Rust. L'interface web doit seulement afficher l'état de la partie, envoyer les coups joués par l'utilisateur et recevoir la nouvelle position.

Architecture conseillée :

```txt
moteur_ia/
├── src/
│   ├── lib.rs
│   ├── main.rs                  # terminal / CLI
│   ├── web_server.rs             # serveur HTTP local
│   ├── board.rs
│   ├── partie.rs
│   ├── legal_move.rs
│   ├── make_move.rs
│   ├── fen.rs
│   └── ...
│
├── web/
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
│       ├── App.tsx
│       ├── main.tsx
│       ├── api.ts
│       ├── types.ts
│       ├── components/
│       │   ├── ChessBoard.tsx
│       │   ├── Square.tsx
│       │   ├── MoveList.tsx
│       │   ├── GameStatus.tsx
│       │   └── Controls.tsx
│       └── styles.css
│
└── Cargo.toml
```

## Principe général

Le design correct est le suivant :

```txt
Navigateur web
   ↓ coup demandé : e2e4
React / TypeScript
   ↓ POST /api/move
Serveur Rust Axum
   ↓ appelle Partie::jouer_coup(...)
Moteur Rust existant
   ↓ retourne nouvel état
Serveur Rust
   ↓ JSON
React
   ↓ affichage propre de l'échiquier
Utilisateur
```

Il ne faut pas refaire les règles d'échecs en JavaScript. Le front ne doit pas décider si un coup est légal. Il peut seulement aider visuellement l'utilisateur, par exemple en colorant les coups possibles envoyés par Rust.

## Phase 1 — Nettoyer la séparation moteur / interface

Objectif : vérifier que toute la logique du jeu est disponible depuis `Partie`, et pas dans `main.rs`.

Dans ton projet actuel, `main.rs` doit rester léger. Il doit seulement :

- créer une `Partie` ;
- afficher la position si tu veux garder le mode terminal ;
- lire une entrée utilisateur ;
- demander à `Partie` de jouer le coup ;
- afficher l'état de la partie.

La logique suivante doit être dans `partie.rs` :

- création d'une partie depuis une FEN ;
- génération des coups légaux ;
- tentative de jouer un coup ;
- détection de mat, pat, règle des 50 coups, répétition ;
- historique des coups ;
- état courant de la partie.

À avoir dans `partie.rs` :

```rust
impl Partie {
    pub fn depuis_fen(fen: &str) -> Result<Self, String> {
        // déjà en place chez toi normalement
    }

    pub fn etat(&self) -> EtatPartie {
        // EnCours, Mat, Pat, Nulle50Coups, NulleRepetition
    }

    pub fn coups_legaux(&mut self) -> Vec<Move> {
        // appelle generate_legal_move
    }

    pub fn jouer_coup(&mut self, mv: Move) -> Result<(), String> {
        // vérifie que le coup est légal
        // appelle make_move
        // met à jour historique, répétitions, etc.
    }
}
```

Point important : si `Partie::jouer_coup` fonctionne bien, alors le web sera simple. Le serveur web n'aura qu'à appeler cette méthode.

## Phase 2 — Créer une représentation JSON simple de la position

Objectif : envoyer au front une structure facile à afficher.

Le front n'a pas besoin de recevoir les bitboards. Les bitboards sont excellents pour le moteur, mais pas pour l'affichage.

Créer une structure dédiée à l'API, par exemple dans `web_server.rs` ou dans un fichier `api_types.rs` :

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct GameStateDto {
    pub board: Vec<SquareDto>,
    pub side_to_move: String,
    pub legal_moves: Vec<MoveDto>,
    pub status: String,
    pub move_history: Vec<String>,
}

#[derive(Serialize)]
pub struct SquareDto {
    pub square: String,      // exemple : "e4"
    pub index: u8,           // exemple : 28
    pub piece: Option<String> // exemple : "P", "k", "Q"
}

#[derive(Serialize)]
pub struct MoveDto {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
    pub notation: String,
}
```

Pourquoi faire comme ça :

- le moteur garde ses `u64` et ses bitboards ;
- l'interface reçoit des chaînes simples ;
- le débogage est beaucoup plus facile ;
- React peut afficher directement les pièces.

Exemple de JSON attendu :

```json
{
  "board": [
    { "square": "a1", "index": 0, "piece": "R" },
    { "square": "b1", "index": 1, "piece": "N" },
    { "square": "c1", "index": 2, "piece": "B" }
  ],
  "side_to_move": "white",
  "legal_moves": [
    { "from": "e2", "to": "e4", "promotion": null, "notation": "e2e4" }
  ],
  "status": "in_progress",
  "move_history": ["e2e4", "e7e5"]
}
```

## Phase 3 — Ajouter les dépendances Rust pour le serveur web

Objectif : créer une API HTTP locale autour du moteur.

Dans `Cargo.toml`, ajouter :

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.5", features = ["cors"] }
```

Puis créer un fichier :

```txt
src/web_server.rs
```

Le serveur aura au minimum ces routes :

```txt
GET  /api/state       retourne l'état actuel de la partie
POST /api/move        joue un coup
POST /api/new         recommence une partie
POST /api/from-fen    charge une position FEN
POST /api/undo        annule le dernier coup, plus tard
```

Pour commencer, ne fais que :

```txt
GET /api/state
POST /api/move
POST /api/new
```

Le reste viendra après.

## Phase 4 — Créer le serveur Rust minimal

Objectif : lancer le moteur comme serveur local.

Dans `src/web_server.rs` :

```rust
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

use crate::partie::Partie;

#[derive(Clone)]
pub struct AppState {
    pub partie: Arc<Mutex<Partie>>,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

pub async fn run_web_server() {
    let partie = Partie::depuis_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    let state = AppState {
        partie: Arc::new(Mutex::new(partie)),
    };

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/move", post(play_move))
        .route("/api/new", post(new_game))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("Serveur web lancé sur http://127.0.0.1:3001");

    axum::serve(listener, app).await.unwrap();
}

async fn get_state(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut partie = state.partie.lock().unwrap();

    // À remplacer par une vraie fonction dto depuis ton board.
    Json(serde_json::json!({
        "status": format!("{:?}", partie.etat()),
        "side_to_move": format!("{:?}", partie.board.side_to_move),
        "board": [],
        "legal_moves": []
    }))
}

async fn play_move(
    State(state): State<AppState>,
    Json(req): Json<MoveRequest>,
) -> Json<serde_json::Value> {
    let mut partie = state.partie.lock().unwrap();

    // Étape suivante : convertir req.from + req.to en Move légal.
    // Pour l'instant, on renvoie juste ce qui est reçu.
    Json(serde_json::json!({
        "received": {
            "from": req.from,
            "to": req.to,
            "promotion": req.promotion
        },
        "status": format!("{:?}", partie.etat())
    }))
}

async fn new_game(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut partie = state.partie.lock().unwrap();

    *partie = Partie::depuis_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    Json(serde_json::json!({
        "ok": true,
        "status": format!("{:?}", partie.etat())
    }))
}
```

Dans `src/lib.rs`, exposer le module :

```rust
pub mod web_server;
```

Dans `src/main.rs`, temporairement :

```rust
use moteur_ia::web_server::run_web_server;

#[tokio::main]
async fn main() {
    run_web_server().await;
}
```

Commande de test :

```bash
cargo run
```

Puis dans un autre terminal :

```bash
curl http://127.0.0.1:3001/api/state
```

Si tu reçois du JSON, le socle web fonctionne.

## Phase 5 — Convertir le board Rust en board affichable

Objectif : remplir correctement `board` dans `/api/state`.

Tu dois créer une fonction qui parcourt les 64 cases et cherche quelle pièce est présente.

Idée :

```rust
pub fn piece_at(board: &CBoard, square: u8) -> Option<Pieces> {
    let bb = 1u64 << square;

    for i in 0..12 {
        if board.piece_bb[i] & bb != 0 {
            return Some(piece_from_index(i));
        }
    }

    None
}
```

Attention : les index 12 et 13 correspondent à `PiecesBlanches` et `PiecesNoires`, donc il ne faut pas les afficher comme des pièces.

Il faudra aussi convertir `Pieces` en symbole :

```rust
pub fn piece_to_symbol(piece: Pieces) -> &'static str {
    match piece {
        Pieces::PionBlanc => "P",
        Pieces::CavalierBlanc => "N",
        Pieces::FouBlanc => "B",
        Pieces::TourBlanche => "R",
        Pieces::DameBlanche => "Q",
        Pieces::RoiBlanc => "K",

        Pieces::PionNoir => "p",
        Pieces::CavalierNoir => "n",
        Pieces::FouNoir => "b",
        Pieces::TourNoire => "r",
        Pieces::DameNoire => "q",
        Pieces::RoiNoir => "k",

        _ => "?",
    }
}
```

Ensuite, `/api/state` doit renvoyer les 64 cases, pas seulement les cases occupées. C'est plus simple pour le front.

Ordre conseillé pour l'affichage :

```txt
a8 b8 c8 d8 e8 f8 g8 h8
...
a1 b1 c1 d1 e1 f1 g1 h1
```

Mais attention : dans ton moteur, `a1 = 0` et `h8 = 63`. Pour afficher côté blanc, il faut parcourir les rangs de 7 à 0 :

```rust
for rank in (0..8).rev() {
    for file in 0..8 {
        let index = rank * 8 + file;
    }
}
```

## Phase 6 — Convertir une demande front en coup légal Rust

Objectif : quand le front envoie `{ from: "e2", to: "e4" }`, Rust doit trouver le `Move` correspondant dans les coups légaux.

Ne reconstruis pas un `Move` à la main depuis le front. C'est une erreur classique. Le bon design est :

1. Rust génère tous les coups légaux.
2. Rust cherche celui qui correspond à `from`, `to`, et éventuellement `promotion`.
3. Rust joue exactement ce `Move`.

Pseudo-code :

```rust
let from_index = coord_to_square_index(&req.from)?;
let to_index = coord_to_square_index(&req.to)?;
let legal_moves = partie.coups_legaux();

let selected = legal_moves
    .into_iter()
    .find(|m| {
        m.from == from_index
            && m.to == to_index
            && promotion_matches(m.promotion, &req.promotion)
    })
    .ok_or("Coup illégal")?;

partie.jouer_coup(selected)?;
```

Pourquoi c'est important :

- les coups spéciaux restent gérés par le moteur ;
- le roque est reconnu ;
- la prise en passant est reconnue ;
- la promotion garde le bon flag ;
- le front ne peut pas tricher avec un coup invalide.

## Phase 7 — Créer le front React / TypeScript

Objectif : faire une interface moderne et maintenable.

Dans le dossier racine :

```bash
npm create vite@latest web -- --template react-ts
cd web
npm install
npm install lucide-react
```

Structure minimale :

```txt
web/src/
├── App.tsx
├── api.ts
├── types.ts
├── styles.css
└── components/
    ├── ChessBoard.tsx
    ├── Square.tsx
    ├── MoveList.tsx
    ├── GameStatus.tsx
    └── Controls.tsx
```

Dans `web/src/types.ts` :

```ts
export type SquareDto = {
  square: string;
  index: number;
  piece: string | null;
};

export type MoveDto = {
  from: string;
  to: string;
  promotion: string | null;
  notation: string;
};

export type GameStateDto = {
  board: SquareDto[];
  side_to_move: string;
  legal_moves: MoveDto[];
  status: string;
  move_history: string[];
};
```

Dans `web/src/api.ts` :

```ts
import type { GameStateDto } from "./types";

const API_URL = "http://127.0.0.1:3001";

export async function fetchGameState(): Promise<GameStateDto> {
  const res = await fetch(`${API_URL}/api/state`);
  if (!res.ok) throw new Error("Impossible de charger la partie");
  return res.json();
}

export async function playMove(from: string, to: string, promotion?: string | null): Promise<GameStateDto> {
  const res = await fetch(`${API_URL}/api/move`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from, to, promotion: promotion ?? null }),
  });

  if (!res.ok) throw new Error("Coup refusé par le moteur");
  return res.json();
}

export async function newGame(): Promise<GameStateDto> {
  const res = await fetch(`${API_URL}/api/new`, { method: "POST" });
  if (!res.ok) throw new Error("Impossible de recommencer la partie");
  return res.json();
}
```

## Phase 8 — Afficher l'échiquier

Objectif : afficher un plateau propre, lisible, responsive.

Dans `ChessBoard.tsx`, le principe est :

- recevoir `board` ;
- afficher 64 cases en grille CSS ;
- mémoriser la case sélectionnée ;
- au clic sur une pièce de la couleur au trait, sélectionner ;
- au clic sur une destination, envoyer le coup au backend ;
- colorer les coups possibles.

Pseudo-code React :

```tsx
import { useState } from "react";
import type { GameStateDto } from "../types";

type Props = {
  game: GameStateDto;
  onMove: (from: string, to: string) => void;
};

export function ChessBoard({ game, onMove }: Props) {
  const [selected, setSelected] = useState<string | null>(null);

  function handleClick(square: string) {
    if (selected === null) {
      setSelected(square);
      return;
    }

    if (selected === square) {
      setSelected(null);
      return;
    }

    onMove(selected, square);
    setSelected(null);
  }

  const possibleTargets = selected
    ? game.legal_moves.filter(m => m.from === selected).map(m => m.to)
    : [];

  return (
    <div className="board">
      {game.board.map((sq) => (
        <button
          key={sq.square}
          className={[
            "square",
            isLightSquare(sq.index) ? "light" : "dark",
            selected === sq.square ? "selected" : "",
            possibleTargets.includes(sq.square) ? "target" : "",
          ].join(" ")}
          onClick={() => handleClick(sq.square)}
        >
          <span className="piece">{pieceToUnicode(sq.piece)}</span>
        </button>
      ))}
    </div>
  );
}

function isLightSquare(index: number): boolean {
  const rank = Math.floor(index / 8);
  const file = index % 8;
  return (rank + file) % 2 === 0;
}

function pieceToUnicode(piece: string | null): string {
  switch (piece) {
    case "K": return "♔";
    case "Q": return "♕";
    case "R": return "♖";
    case "B": return "♗";
    case "N": return "♘";
    case "P": return "♙";
    case "k": return "♚";
    case "q": return "♛";
    case "r": return "♜";
    case "b": return "♝";
    case "n": return "♞";
    case "p": return "♟";
    default: return "";
  }
}
```

Remarque : pour une interface plus professionnelle, tu pourras ensuite remplacer les caractères Unicode par des fichiers SVG de pièces.

## Phase 9 — Créer le layout visuel propre

Objectif : ne pas avoir juste un échiquier brut, mais une vraie interface.

Disposition conseillée :

```txt
┌─────────────────────────────────────────────┐
│ Moteur d'échecs Rust                        │
│ Partie locale — règles complètes            │
├───────────────────────┬─────────────────────┤
│                       │ Trait : Blanc        │
│      Échiquier        │ État : En cours      │
│                       │ Coups joués          │
│                       │ Boutons              │
└───────────────────────┴─────────────────────┘
```

À afficher à droite :

- couleur au trait ;
- état de la partie ;
- nombre de coups légaux ;
- historique des coups ;
- bouton nouvelle partie ;
- bouton charger FEN, plus tard ;
- bouton inverser le plateau, plus tard.

Dans `styles.css`, viser une interface sobre :

```css
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #111827;
  color: #f9fafb;
}

.app {
  min-height: 100vh;
  padding: 32px;
}

.header {
  max-width: 1100px;
  margin: 0 auto 24px auto;
}

.header h1 {
  margin: 0;
  font-size: 32px;
}

.header p {
  margin: 8px 0 0 0;
  color: #9ca3af;
}

.layout {
  max-width: 1100px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: minmax(320px, 640px) 320px;
  gap: 24px;
  align-items: start;
}

.board {
  width: min(80vw, 640px);
  aspect-ratio: 1 / 1;
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  grid-template-rows: repeat(8, 1fr);
  border-radius: 18px;
  overflow: hidden;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.45);
}

.square {
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
}

.square.light {
  background: #e5d5b5;
}

.square.dark {
  background: #7a4f2a;
}

.square.selected {
  outline: 4px solid rgba(250, 204, 21, 0.9);
  outline-offset: -4px;
}

.square.target::after {
  content: "";
  width: 28%;
  height: 28%;
  border-radius: 999px;
  background: rgba(17, 24, 39, 0.35);
  position: absolute;
}

.piece {
  font-size: clamp(32px, 7vw, 70px);
  line-height: 1;
  z-index: 1;
  user-select: none;
}

.panel {
  background: rgba(31, 41, 55, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  padding: 20px;
  box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
}

.panel h2 {
  margin-top: 0;
}

button.primary {
  width: 100%;
  padding: 12px 16px;
  border: none;
  border-radius: 10px;
  background: #f9fafb;
  color: #111827;
  font-weight: 700;
  cursor: pointer;
}

.move-list {
  margin-top: 16px;
  max-height: 300px;
  overflow: auto;
  color: #d1d5db;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

@media (max-width: 900px) {
  .layout {
    grid-template-columns: 1fr;
  }

  .board {
    width: 100%;
  }
}
```

## Phase 10 — App.tsx minimal

Objectif : connecter le front à l'API.

```tsx
import { useEffect, useState } from "react";
import type { GameStateDto } from "./types";
import { fetchGameState, newGame, playMove } from "./api";
import { ChessBoard } from "./components/ChessBoard";
import "./styles.css";

export default function App() {
  const [game, setGame] = useState<GameStateDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    try {
      setGame(await fetchGameState());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erreur inconnue");
    }
  }

  async function handleMove(from: string, to: string) {
    try {
      const next = await playMove(from, to, null);
      setGame(next);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Coup refusé");
    }
  }

  async function handleNewGame() {
    setGame(await newGame());
  }

  useEffect(() => {
    refresh();
  }, []);

  if (!game) {
    return <div className="app">Chargement...</div>;
  }

  return (
    <main className="app">
      <header className="header">
        <h1>Moteur d'échecs Rust</h1>
        <p>Interface web locale connectée à ton moteur bitboards.</p>
      </header>

      <section className="layout">
        <ChessBoard game={game} onMove={handleMove} />

        <aside className="panel">
          <h2>Partie</h2>
          <p><strong>Trait :</strong> {game.side_to_move}</p>
          <p><strong>État :</strong> {game.status}</p>
          <p><strong>Coups légaux :</strong> {game.legal_moves.length}</p>

          {error && <p className="error">{error}</p>}

          <button className="primary" onClick={handleNewGame}>
            Nouvelle partie
          </button>

          <div className="move-list">
            {game.move_history.map((move, index) => (
              <div key={`${move}-${index}`}>{index + 1}. {move}</div>
            ))}
          </div>
        </aside>
      </section>
    </main>
  );
}
```

## Phase 11 — Gestion des promotions

Objectif : ne pas bloquer quand un pion arrive sur la dernière rangée.

Dans un premier temps, tu peux faire simple : promotion automatique en dame.

Côté front :

```ts
playMove(from, to, "q")
```

Côté Rust : si plusieurs coups légaux ont le même `from` et `to`, choisir la dame par défaut si aucune promotion n'est précisée.

Ensuite, ajouter une petite fenêtre de choix :

```txt
Promotion
[ Dame ] [ Tour ] [ Fou ] [ Cavalier ]
```

Ne commence pas par cette fenêtre. Fais d'abord marcher les coups normaux.

## Phase 12 — Améliorations visuelles importantes

Une fois la base fonctionnelle, ajouter dans cet ordre :

1. Coordonnées autour du plateau : `a b c d e f g h` et `1 2 3 4 5 6 7 8`.
2. Dernier coup joué en surbrillance.
3. Cases possibles au clic sur une pièce.
4. Coups de capture affichés différemment des coups calmes.
5. Affichage clair du mat, pat, nulle 50 coups, nulle par répétition.
6. Bouton pour retourner l'échiquier.
7. Entrée FEN.
8. Mode analyse avec liste des coups légaux.
9. Plus tard : évaluation matérielle.
10. Plus tard : minimax et choix automatique du moteur.

## Phase 13 — Commandes de lancement

Terminal 1 : backend Rust

```bash
cargo run
```

Terminal 2 : frontend React

```bash
cd web
npm run dev
```

Puis ouvrir :

```txt
http://localhost:5173
```

## Phase 14 — Tests à faire avant de considérer l'interface comme fiable

Tester manuellement :

- position initiale affichée correctement ;
- e2e4 fonctionne ;
- e7e5 fonctionne ;
- un coup illégal est refusé ;
- le roque fonctionne ;
- la prise en passant fonctionne ;
- la promotion fonctionne ;
- le mat est affiché ;
- le pat est affiché ;
- la règle des 50 coups est affichée ;
- la répétition est affichée ;
- nouvelle partie remet bien tout à zéro.

Tests FEN utiles :

```txt
Position initiale :
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

Mat simple :
7k/7Q/6K1/8/8/8/8/8 b - - 0 1

Pat simple :
7k/5Q2/6K1/8/8/8/8/8 b - - 0 1

Règle des 50 coups :
7k/8/8/8/8/8/8/K7 w - - 100 80
```

## Phase 15 — Ordre de travail conseillé

Jour 1 — API Rust minimale

Durée : 3 à 4 heures.

Livrable : `cargo run` lance un serveur local et `/api/state` renvoie du JSON.

À faire :

- ajouter Axum, Tokio, Serde ;
- créer `web_server.rs` ;
- créer `AppState` avec `Arc<Mutex<Partie>>` ;
- créer `/api/state` ;
- vérifier avec `curl`.

Jour 2 — Board JSON réel

Durée : 4 à 5 heures.

Livrable : `/api/state` renvoie les 64 cases avec les pièces correctes.

À faire :

- créer `piece_at` ;
- créer `piece_to_symbol` ;
- créer `square_index_to_coord` si nécessaire ;
- remplir `GameStateDto` ;
- afficher les coups légaux dans le JSON.

Jour 3 — Front React minimal

Durée : 4 à 5 heures.

Livrable : le navigateur affiche l'échiquier.

À faire :

- créer projet Vite React TS ;
- créer `types.ts` ;
- créer `api.ts` ;
- créer `ChessBoard.tsx` ;
- afficher les pièces Unicode.

Jour 4 — Coups jouables

Durée : 5 à 6 heures.

Livrable : on peut jouer une partie depuis le navigateur.

À faire :

- clic source ;
- clic destination ;
- POST `/api/move` ;
- côté Rust, chercher le `Move` légal correspondant ;
- renvoyer le nouvel état complet.

Jour 5 — Interface propre

Durée : 4 à 5 heures.

Livrable : interface visuellement présentable.

À faire :

- CSS propre ;
- panneau latéral ;
- état de partie ;
- historique ;
- bouton nouvelle partie ;
- erreurs lisibles.

Jour 6 — Coups spéciaux et bugs

Durée : 5 à 6 heures.

Livrable : roque, prise en passant et promotion testés depuis le web.

À faire :

- tester roque ;
- tester en passant ;
- tester promotion automatique dame ;
- corriger les conversions front/backend.

Jour 7 — Qualité finale

Durée : 4 à 6 heures.

Livrable : application propre, stable, montrable à quelqu'un.

À faire :

- dernier coup en surbrillance ;
- coordonnées du plateau ;
- bouton inverser l'échiquier ;
- entrée FEN ;
- messages fin de partie ;
- README avec commandes de lancement.

## Règle de conception à garder

Le front affiche. Le backend décide. Le moteur calcule.

```txt
React = interface
Axum = passerelle HTTP
Partie = logique de jeu
Board / Move / legal_move / make_move = moteur pur
```

Si tu respectes cette séparation, tu pourras ensuite ajouter facilement :

- une IA minimax ;
- une évaluation matérielle ;
- un mode analyse ;
- une sauvegarde de parties ;
- un historique PGN ;
- une interface publique déployée en ligne.
