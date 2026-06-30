# Roadmap d'amelioration de l'evaluation

Ce document est un guide d'integration progressif pour rendre la fonction d'evaluation plus intelligente sans toucher aux optimisations de recherche. Il part du code actuel du dossier `src/` et donne des emplacements precis pour chaque ajout.

Hors perimetre volontaire: alpha-beta, table de transposition, Zobrist, killer moves, history heuristic, null move, LMR et toute autre optimisation de recherche. Ici, le sujet est uniquement la qualite echiqueenne du score retourne par l'evaluation.

# Analyse de l'etat actuel de l'evaluation

L'evaluation est actuellement dans:

```text
src/eval.rs
```

Le fichier `src/eval.rs` contient aussi la recherche: `evaluation_min_max`, `evaluation_negamax_alpha_beta`, `quiescence`, `meilleur_coup`, `meilleur_coup_iterative`, l'ordre des coups et les heuristiques de recherche. C'est pratique au debut, mais cela rend l'evaluation difficile a faire evoluer proprement.

Les elements d'evaluation existants sont:

| Element | Present | Emplacement actuel |
|---|---:|---|
| Materiel | Oui | `evaluation_materielle`, `src/eval.rs`, bloc autour de la ligne 122 |
| Bonus de cavaliers | Oui | `BONUS_CAVALIER` puis `evaluation_cavaliers`, autour des lignes 19 et 478 |
| Bonus de pions | Oui | `BONUS_PION` puis `evaluation_pions`, autour des lignes 25 et 461 |
| Paire de fous | Oui | `evaluation_paire_de_fous`, autour de la ligne 495 |
| Roque | Oui | `evaluation_roque`, autour de la ligne 509 |
| Conversion blanc vers negamax | Oui | `evaluation_negamax`, autour de la ligne 142 |
| Tables de cases | Oui, partielles | Pions et cavaliers uniquement |
| `mirror_square` | Oui | autour de la ligne 458 |

La representation est bien adaptee a une evaluation moderne:

```text
src/board.rs
```

- `CBoard::piece_bb` contient les bitboards par piece.
- `Pieces::PiecesBlanches` et `Pieces::PiecesNoires` contiennent les occupations par couleur.
- `white_king_square` et `black_king_square` donnent la case des rois.
- Les cases utilisent l'index `0..63`, avec `a1 = 0`, `h1 = 7`, `a8 = 56`, `h8 = 63`.

La fonction:

```rust
fn mirror_square(square: u8) -> u8 {
    square ^ 56
}
```

retourne verticalement une case. Exemple: `a1` devient `a8`, `e2` devient `e7`. C'est utile parce que les tables de cases sont ecrites du point de vue blanc. Pour evaluer une piece noire, on lit la case miroir puis on soustrait le bonus.

Le point a ameliorer en priorite: `evaluation_blanc` additionne aujourd'hui un score unique:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);
    score += evaluation_pions(board);
    score
}
```

Cette base est saine pour apprendre, mais elle ne distingue pas encore:

- ouverture, milieu de jeu, finale;
- roi abrite en milieu de jeu et actif en finale;
- structure de pions;
- mobilite;
- tours sur colonnes ouvertes;
- developpement.

# 1. Nettoyer et structurer l'evaluation existante

## Objectif

Separer l'evaluation de la recherche pour pouvoir ajouter des fonctions sans allonger encore `src/eval.rs`.

Recommendation: creer un fichier dedie:

```text
src/evaluation.rs
```

Garder dans `src/eval.rs` tout ce qui concerne la recherche et l'ordre des coups. Deplacer dans `src/evaluation.rs` tout ce qui calcule un score statique de position.

## Fichiers concernes

```text
src/eval.rs
src/evaluation.rs
src/lib.rs
```

## Ce qu'il faut deplacer

Depuis `src/eval.rs`, chercher et deplacer dans `src/evaluation.rs`:

- `BONUS_CAVALIER`;
- `BONUS_PION`;
- `evaluation_materielle`;
- `evaluation_negamax`;
- `evaluation_blanc`;
- `pop_lsb`;
- `mirror_square`;
- `evaluation_pions`;
- `evaluation_cavaliers`;
- `evaluation_paire_de_fous`;
- `evaluation_roque`;
- les tests unitaires directement lies a ces fonctions.

Ne pas deplacer:

- `SCORE_MAT`, `INF`, `MAX_PLY`;
- `SearchStats`;
- `SearchHeuristics`;
- `SearchLimits`;
- `valeur_piece_abs`;
- `score_capture_mvv_lva`;
- `score_ordre_coup`;
- `evaluation_min_max`;
- `evaluation_negamax_alpha_beta`;
- `quiescence`;
- `meilleur_coup`;
- `meilleur_coup_iterative`.

## Code a mettre dans `src/evaluation.rs`

Placement: creer le fichier `src/evaluation.rs`, puis mettre ces imports tout en haut du fichier.

```rust
use crate::board::{CBoard, Color, Pieces};
```

Ensuite, coller les constantes et fonctions d'evaluation deplacees. Les fonctions qui doivent etre appelees depuis `src/eval.rs` doivent etre publiques:

```rust
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
    for (piece, valeur) in valeurs {
        let nombre = board.piece_bb[piece as usize].count_ones() as i32;
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
```

`evaluation_blanc` doit aussi etre publique si vous voulez l'utiliser dans les tests d'integration:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);
    score += evaluation_pions(board);
    score
}
```

Les fonctions internes restent privees:

```rust
fn pop_lsb(bb: &mut u64) -> Option<u8> {
    if *bb == 0 {
        return None;
    }

    let square = bb.trailing_zeros() as u8;
    *bb &= *bb - 1;
    Some(square)
}

fn mirror_square(square: u8) -> u8 {
    square ^ 56
}
```

## Modification dans `src/lib.rs`

Placement: dans `src/lib.rs`, juste apres:

```rust
pub mod eval;
```

Ajouter:

```rust
pub mod evaluation;
```

## Modification dans `src/eval.rs`

Placement: en haut de `src/eval.rs`, dans le bloc d'imports.

Chercher:

```rust
use crate::board::{CBoard, Color, Pieces};
```

Remplacer par:

```rust
use crate::board::{CBoard, Pieces};
use crate::evaluation::evaluation_negamax;
```

Si `board_from_fen` n'est plus utilise que par les tests deplaces, supprimer aussi:

```rust
use crate::fen::board_from_fen;
```

## Tests a garder ou deplacer

Les tests actuels en bas de `src/eval.rs` peuvent etre deplaces dans `src/evaluation.rs`, sous les fonctions d'evaluation.

Placement dans `src/evaluation.rs`: tout en bas du fichier, ajouter:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::board_from_fen;

    #[test]
    fn cavalier_centre_meilleur_que_cavalier_bord() {
        let board_bord = board_from_fen("7k/8/8/8/8/8/N7/K7 w - - 0 1").unwrap();
        let board_centre = board_from_fen("7k/8/8/8/3N4/8/8/K7 w - - 0 1").unwrap();

        assert!(evaluation_blanc(&board_centre) > evaluation_blanc(&board_bord));
    }
}
```

## Verification

- `cargo test cavalier_centre_meilleur_que_cavalier_bord`
- `cargo test paire_de_fous_donne_bonus_aux_blancs`
- `cargo test roque_blanc_donne_bonus`

Risque: faible.  
Difficulte: faible.  
Gain attendu: code plus lisible, base plus propre pour les etapes suivantes.

# 2. Creer une evaluation par phase de jeu

## Pourquoi ajouter une phase

Une meme caracteristique ne vaut pas pareil selon la partie:

- En ouverture, developper les pieces et proteger le roi compte beaucoup.
- En milieu de jeu, la securite du roi, la mobilite et les cases actives comptent fortement.
- En finale, le roi devient une piece active et les pions passes prennent beaucoup de valeur.

La methode simple consiste a calculer une phase entre `0` et `24`:

- `24` = milieu de jeu complet, beaucoup de pieces.
- `0` = finale, peu de pieces.

Ensuite, on calcule deux scores:

- `mg`: score milieu de jeu;
- `eg`: score finale.

Puis on interpole:

```text
score = (mg * phase + eg * (24 - phase)) / 24
```

## Code a ajouter dans `src/evaluation.rs`

Placement: dans `src/evaluation.rs`, apres les imports et avant les tables de cases.

Ajouter:

```rust
const MAX_PHASE: i32 = 24;

#[derive(Default, Clone, Copy)]
struct EvalScore {
    mg: i32,
    eg: i32,
}

impl EvalScore {
    fn add(&mut self, other: EvalScore) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

fn blend_score(score: EvalScore, phase: i32) -> i32 {
    (score.mg * phase + score.eg * (MAX_PHASE - phase)) / MAX_PHASE
}

fn game_phase(board: &CBoard) -> i32 {
    let phase = board.piece_bb[Pieces::CavalierBlanc as usize].count_ones() as i32
        + board.piece_bb[Pieces::CavalierNoir as usize].count_ones() as i32
        + board.piece_bb[Pieces::FouBlanc as usize].count_ones() as i32
        + board.piece_bb[Pieces::FouNoir as usize].count_ones() as i32
        + 2 * board.piece_bb[Pieces::TourBlanche as usize].count_ones() as i32
        + 2 * board.piece_bb[Pieces::TourNoire as usize].count_ones() as i32
        + 4 * board.piece_bb[Pieces::DameBlanche as usize].count_ones() as i32
        + 4 * board.piece_bb[Pieces::DameNoire as usize].count_ones() as i32;

    phase.clamp(0, MAX_PHASE)
}
```

## Remplacement de `evaluation_blanc`

Placement: dans `src/evaluation.rs`, chercher la fonction `pub fn evaluation_blanc(board: &CBoard) -> i32`.

Remplacer tout le bloc par:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let phase = game_phase(board);
    let materiel = evaluation_materielle(board);

    let mut score = EvalScore {
        mg: materiel,
        eg: materiel,
    };

    score.add(evaluation_tables_de_cases(board));
    score.add(EvalScore {
        mg: evaluation_paire_de_fous(board),
        eg: evaluation_paire_de_fous(board),
    });

    // Les anciennes fonctions peuvent rester branchees au debut.
    // Elles seront remplacees progressivement par les sections suivantes.
    score.mg += evaluation_roque(board);

    blend_score(score, phase)
}
```

## Modification de `evaluation_negamax`

Placement: dans `src/evaluation.rs`, laisser `evaluation_negamax` appeler `evaluation_blanc`. Il n'a pas besoin de connaitre la phase.

```rust
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_blanc(board);
    match board.side_to_move {
        Color::Blanc => score,
        Color::Noir => -score,
    }
}
```

## Verification

Ajouter temporairement ce test dans `src/evaluation.rs`, dans le module `tests`:

```rust
#[test]
fn phase_position_initiale_superieure_a_phase_finale() {
    let initiale =
        board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let finale = board_from_fen("7k/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();

    assert!(game_phase(&initiale) > game_phase(&finale));
}
```

Risque: moyen si les scores `mg` et `eg` ne sont pas alimentes de facon coherente.  
Difficulte: moyenne.  
Gain attendu: base indispensable pour roi en finale, pions passes et developpement.

# 3. Ameliorer les tables de cases

## Idee

Une table de cases donne un bonus ou malus selon la case d'une piece.

Exemples:

- Un cavalier aime le centre car il controle plus de cases.
- Un pion avance peut etre utile, mais trop avance sans soutien peut devenir faible.
- Le roi doit etre protege en milieu de jeu, mais actif en finale.

Votre moteur a deja deux tables:

- `BONUS_CAVALIER`;
- `BONUS_PION`.

L'etape suivante est de remplacer ces deux tables par des tables milieu de jeu et finale pour toutes les pieces.

## Code a ajouter dans `src/evaluation.rs`

Placement: apres `game_phase` et avant `evaluation_blanc`.

Ajouter ces constantes:

```rust
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
```

## Fonction generique d'application des tables

Placement: dans `src/evaluation.rs`, juste apres les constantes de tables.

Ajouter:

```rust
fn score_piece_square(
    board: &CBoard,
    white_piece: Pieces,
    black_piece: Pieces,
    mg_table: &[i32; 64],
    eg_table: &[i32; 64],
) -> EvalScore {
    let mut score = EvalScore::default();

    let mut white = board.piece_bb[white_piece as usize];
    while let Some(square) = pop_lsb(&mut white) {
        score.mg += mg_table[square as usize];
        score.eg += eg_table[square as usize];
    }

    let mut black = board.piece_bb[black_piece as usize];
    while let Some(square) = pop_lsb(&mut black) {
        let mirrored = mirror_square(square);
        score.mg -= mg_table[mirrored as usize];
        score.eg -= eg_table[mirrored as usize];
    }

    score
}

fn evaluation_tables_de_cases(board: &CBoard) -> EvalScore {
    let mut score = EvalScore::default();

    score.add(score_piece_square(board, Pieces::PionBlanc, Pieces::PionNoir, &PAWN_MG, &PAWN_EG));
    score.add(score_piece_square(board, Pieces::CavalierBlanc, Pieces::CavalierNoir, &KNIGHT_MG, &KNIGHT_EG));
    score.add(score_piece_square(board, Pieces::FouBlanc, Pieces::FouNoir, &BISHOP_MG, &BISHOP_EG));
    score.add(score_piece_square(board, Pieces::TourBlanche, Pieces::TourNoire, &ROOK_MG, &ROOK_EG));
    score.add(score_piece_square(board, Pieces::DameBlanche, Pieces::DameNoire, &QUEEN_MG, &QUEEN_EG));
    score.add(score_piece_square(board, Pieces::RoiBlanc, Pieces::RoiNoir, &KING_MG, &KING_EG));

    score
}
```

## Code a remplacer

Placement: dans `src/evaluation.rs`.

Quand `evaluation_tables_de_cases` est branchee dans `evaluation_blanc`, les anciennes fonctions suivantes deviennent redondantes:

- `evaluation_pions`;
- `evaluation_cavaliers`;
- `BONUS_PION`;
- `BONUS_CAVALIER`.

Vous pouvez les supprimer apres avoir valide les tests.

## Verification

- Un cavalier au centre doit rester meilleur qu'un cavalier au bord.
- En finale, un roi central doit devenir meilleur qu'un roi passif.

Risque: moyen, car une table trop forte peut dominer le materiel.  
Difficulte: moyenne.  
Gain attendu: evaluation beaucoup plus sensible aux bonnes cases.

# 4. Structure de pions

## Pourquoi evaluer les pions

Les pions structurent toute la position:

- Un pion double est souvent faible parce qu'il ne peut plus etre soutenu naturellement par un pion de la meme colonne.
- Un pion isole n'a aucun pion ami sur les colonnes voisines.
- Un pion passe n'a plus de pion adverse devant lui sur sa colonne ou les colonnes voisines.
- Un pion arriere est difficile a pousser et peut devenir une cible.

Cette evaluation doit rester peu couteuse: elle parcourt seulement les bitboards de pions.

## Code a ajouter dans `src/evaluation.rs`

Placement: apres `evaluation_tables_de_cases`.

Ajouter:

```rust
const DOUBLED_PAWN_PENALTY: i32 = 12;
const ISOLATED_PAWN_PENALTY: i32 = 15;
const BACKWARD_PAWN_PENALTY: i32 = 10;
const PASSED_PAWN_BONUS: [i32; 8] = [0, 8, 15, 30, 55, 90, 140, 0];

fn file_mask(file: u8) -> u64 {
    0x0101_0101_0101_0101u64 << file
}

fn adjacent_files_mask(file: u8) -> u64 {
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

    let forward_ranks = match color {
        Color::Blanc => {
            if rank == 7 {
                0
            } else {
                !0u64 << ((rank + 1) * 8)
            }
        }
        Color::Noir => {
            if rank == 0 {
                0
            } else {
                (1u64 << (rank * 8)) - 1
            }
        }
    };

    files & forward_ranks
}

fn rank_from_color(square: u8, color: Color) -> usize {
    let rank = square / 8;
    match color {
        Color::Blanc => rank as usize,
        Color::Noir => (7 - rank) as usize,
    }
}

fn square_from_file_rank(file: i32, rank: i32) -> Option<u8> {
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Some((rank * 8 + file) as u8)
    } else {
        None
    }
}

fn pawn_attacks_square(enemy_pawns: u64, target: u8, enemy_color: Color) -> bool {
    let file = (target % 8) as i32;
    let rank = (target / 8) as i32;

    let attackers = match enemy_color {
        Color::Blanc => [(file - 1, rank - 1), (file + 1, rank - 1)],
        Color::Noir => [(file - 1, rank + 1), (file + 1, rank + 1)],
    };

    attackers
        .into_iter()
        .filter_map(|(f, r)| square_from_file_rank(f, r))
        .any(|sq| enemy_pawns & (1u64 << sq) != 0)
}

fn has_friendly_pawn_behind_on_adjacent_file(friendly_pawns: u64, square: u8, color: Color) -> bool {
    let file = (square % 8) as i32;
    let rank = (square / 8) as i32;

    for df in [-1, 1] {
        let f = file + df;
        for r in 0..8 {
            let behind = match color {
                Color::Blanc => r <= rank,
                Color::Noir => r >= rank,
            };
            if behind {
                if let Some(sq) = square_from_file_rank(f, r) {
                    if friendly_pawns & (1u64 << sq) != 0 {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn evaluation_structure_pions_couleur(board: &CBoard, color: Color) -> i32 {
    let (friendly_piece, enemy_piece) = match color {
        Color::Blanc => (Pieces::PionBlanc, Pieces::PionNoir),
        Color::Noir => (Pieces::PionNoir, Pieces::PionBlanc),
    };

    let friendly_pawns = board.piece_bb[friendly_piece as usize];
    let enemy_pawns = board.piece_bb[enemy_piece as usize];
    let mut score = 0;

    for file in 0..8 {
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

        if enemy_pawns & passed_pawn_mask(square, color) == 0 {
            score += PASSED_PAWN_BONUS[rank_from_color(square, color)];
        }

        let front_square = match color {
            Color::Blanc if square <= 55 => Some(square + 8),
            Color::Noir if square >= 8 => Some(square - 8),
            _ => None,
        };

        if let Some(front) = front_square {
            let enemy_color = match color {
                Color::Blanc => Color::Noir,
                Color::Noir => Color::Blanc,
            };

            if !has_friendly_pawn_behind_on_adjacent_file(friendly_pawns, square, color)
                && pawn_attacks_square(enemy_pawns, front, enemy_color)
            {
                score -= BACKWARD_PAWN_PENALTY;
            }
        }
    }

    score
}

fn evaluation_structure_pions(board: &CBoard) -> EvalScore {
    let white = evaluation_structure_pions_couleur(board, Color::Blanc);
    let black = evaluation_structure_pions_couleur(board, Color::Noir);
    let score = white - black;

    EvalScore {
        mg: score,
        eg: score,
    }
}
```

## Brancher dans `evaluation_blanc`

Placement: dans `src/evaluation.rs`, dans `evaluation_blanc`, apres:

```rust
score.add(evaluation_tables_de_cases(board));
```

Ajouter:

```rust
score.add(evaluation_structure_pions(board));
```

## Verification

- Un pion passe doit augmenter le score.
- Un pion isole doit baisser le score.
- Deux pions doubles sur une colonne doivent etre penalises.

Risque: moyen, car les pions passes peuvent devenir trop forts si le bonus est trop haut.  
Difficulte: moyenne.  
Gain attendu: le moteur comprend mieux les plans de pions.

# 5. Mobilite des pieces

## Idee

La mobilite mesure le nombre de cases accessibles par les pieces. Une piece active vaut souvent plus qu'une piece enfermee.

Ne pas utiliser `generate_legal_move` dans l'evaluation pour commencer:

- cela cree des allocations;
- cela modifie temporairement le plateau via `make_move`;
- cela coute cher a chaque feuille de recherche.

Utiliser plutot les masques existants dans:

```text
src/attack_tables.rs
```

Fonctions utiles:

- `masques_mouvements_cavalier`;
- `masques_mouvements_fou`;
- `masques_mouvements_tour`;
- `masques_mouvements_dame`.

## Imports a ajouter dans `src/evaluation.rs`

Placement: tout en haut de `src/evaluation.rs`, avec les autres imports.

Ajouter:

```rust
use crate::attack_tables::{
    masques_mouvements_cavalier, masques_mouvements_dame, masques_mouvements_fou,
    masques_mouvements_tour,
};
```

## Code a ajouter

Placement: apres `evaluation_structure_pions`.

```rust
const KNIGHT_MOBILITY: i32 = 4;
const BISHOP_MOBILITY: i32 = 4;
const ROOK_MOBILITY: i32 = 2;
const QUEEN_MOBILITY: i32 = 1;

fn mobility_side(board: &CBoard, color: Color) -> i32 {
    let (own, knight, bishop, rook, queen) = match color {
        Color::Blanc => (
            board.piece_bb[Pieces::PiecesBlanches as usize],
            Pieces::CavalierBlanc,
            Pieces::FouBlanc,
            Pieces::TourBlanche,
            Pieces::DameBlanche,
        ),
        Color::Noir => (
            board.piece_bb[Pieces::PiecesNoires as usize],
            Pieces::CavalierNoir,
            Pieces::FouNoir,
            Pieces::TourNoire,
            Pieces::DameNoire,
        ),
    };

    let mut score = 0;

    let mut pieces = board.piece_bb[knight as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        score += ((masques_mouvements_cavalier(square as usize) & !own).count_ones() as i32)
            * KNIGHT_MOBILITY;
    }

    let mut pieces = board.piece_bb[bishop as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        score += ((masques_mouvements_fou(square as usize, board.occupe_bb) & !own).count_ones() as i32)
            * BISHOP_MOBILITY;
    }

    let mut pieces = board.piece_bb[rook as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        score += ((masques_mouvements_tour(square as usize, board.occupe_bb) & !own).count_ones() as i32)
            * ROOK_MOBILITY;
    }

    let mut pieces = board.piece_bb[queen as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        score += ((masques_mouvements_dame(square as usize, board.occupe_bb) & !own).count_ones() as i32)
            * QUEEN_MOBILITY;
    }

    score
}

fn evaluation_mobilite(board: &CBoard) -> EvalScore {
    let score = mobility_side(board, Color::Blanc) - mobility_side(board, Color::Noir);

    EvalScore {
        mg: score,
        eg: score / 2,
    }
}
```

## Brancher dans `evaluation_blanc`

Placement: dans `evaluation_blanc`, apres la structure de pions.

```rust
score.add(evaluation_mobilite(board));
```

## Verification

Comparer deux positions avec le meme materiel:

- fou bloque par ses propres pions;
- fou avec diagonale libre.

Le score de la position active doit etre meilleur.

Risque: moyen si la mobilite domine les tables de cases.  
Difficulte: moyenne.  
Gain attendu: le moteur prefere les pieces actives.

# 6. Securite du roi

## Idee

En milieu de jeu, le roi doit etre protege. En finale, le roi doit devenir actif. C'est pour cette raison que la securite du roi doit surtout entrer dans `mg`, pas dans `eg`.

Elements simples a evaluer:

- roi roque;
- pions devant le roi;
- colonnes ouvertes pres du roi;
- attaques adverses dans la zone du roi.

## Import a ajouter

Placement: en haut de `src/evaluation.rs`, completer l'import `attack_tables` en ajoutant aussi:

```rust
masques_mouvements_roi,
```

L'import complet devient:

```rust
use crate::attack_tables::{
    masques_mouvements_cavalier, masques_mouvements_dame, masques_mouvements_fou,
    masques_mouvements_roi, masques_mouvements_tour,
};
```

## Code a ajouter

Placement: apres `evaluation_mobilite`.

```rust
const CASTLED_KING_BONUS: i32 = 25;
const KING_SHIELD_BONUS: i32 = 8;
const OPEN_FILE_NEAR_KING_PENALTY: i32 = 18;
const SEMI_OPEN_FILE_NEAR_KING_PENALTY: i32 = 10;
const KING_ZONE_ATTACK_PENALTY: i32 = 6;

fn king_square(board: &CBoard, color: Color) -> u8 {
    match color {
        Color::Blanc => board.white_king_square,
        Color::Noir => board.black_king_square,
    }
}

fn king_has_castled(board: &CBoard, color: Color) -> bool {
    match color {
        Color::Blanc => board.white_king_square == 6 || board.white_king_square == 2,
        Color::Noir => board.black_king_square == 62 || board.black_king_square == 58,
    }
}

fn pawn_shield_score(board: &CBoard, color: Color) -> i32 {
    let king = king_square(board, color);
    let king_file = (king % 8) as i32;
    let king_rank = (king / 8) as i32;
    let pawn_piece = match color {
        Color::Blanc => Pieces::PionBlanc,
        Color::Noir => Pieces::PionNoir,
    };

    let shield_rank = match color {
        Color::Blanc => king_rank + 1,
        Color::Noir => king_rank - 1,
    };

    let mut score = 0;
    for df in -1..=1 {
        if let Some(square) = square_from_file_rank(king_file + df, shield_rank) {
            if board.piece_bb[pawn_piece as usize] & (1u64 << square) != 0 {
                score += KING_SHIELD_BONUS;
            } else {
                score -= KING_SHIELD_BONUS;
            }
        }
    }

    score
}

fn file_pressure_near_king(board: &CBoard, color: Color) -> i32 {
    let king = king_square(board, color);
    let king_file = (king % 8) as i32;
    let friendly_pawn = match color {
        Color::Blanc => Pieces::PionBlanc,
        Color::Noir => Pieces::PionNoir,
    };
    let all_pawns =
        board.piece_bb[Pieces::PionBlanc as usize] | board.piece_bb[Pieces::PionNoir as usize];

    let mut penalty = 0;
    for df in -1..=1 {
        let file = king_file + df;
        if !(0..8).contains(&file) {
            continue;
        }

        let mask = file_mask(file as u8);
        let has_any_pawn = all_pawns & mask != 0;
        let has_friendly_pawn = board.piece_bb[friendly_pawn as usize] & mask != 0;

        if !has_any_pawn {
            penalty += OPEN_FILE_NEAR_KING_PENALTY;
        } else if !has_friendly_pawn {
            penalty += SEMI_OPEN_FILE_NEAR_KING_PENALTY;
        }
    }

    penalty
}

fn attacks_on_king_zone(board: &CBoard, attacker: Color, zone: u64) -> i32 {
    let (knight, bishop, rook, queen) = match attacker {
        Color::Blanc => (
            Pieces::CavalierBlanc,
            Pieces::FouBlanc,
            Pieces::TourBlanche,
            Pieces::DameBlanche,
        ),
        Color::Noir => (
            Pieces::CavalierNoir,
            Pieces::FouNoir,
            Pieces::TourNoire,
            Pieces::DameNoire,
        ),
    };

    let mut attacks = 0;

    let mut pieces = board.piece_bb[knight as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        attacks += (masques_mouvements_cavalier(square as usize) & zone).count_ones() as i32;
    }

    let mut pieces = board.piece_bb[bishop as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        attacks += (masques_mouvements_fou(square as usize, board.occupe_bb) & zone).count_ones() as i32;
    }

    let mut pieces = board.piece_bb[rook as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        attacks += (masques_mouvements_tour(square as usize, board.occupe_bb) & zone).count_ones() as i32;
    }

    let mut pieces = board.piece_bb[queen as usize];
    while let Some(square) = pop_lsb(&mut pieces) {
        attacks += 2 * (masques_mouvements_dame(square as usize, board.occupe_bb) & zone).count_ones() as i32;
    }

    attacks
}

fn king_safety_side(board: &CBoard, color: Color) -> i32 {
    let king = king_square(board, color);
    let zone = masques_mouvements_roi(king as usize) | (1u64 << king);
    let enemy = match color {
        Color::Blanc => Color::Noir,
        Color::Noir => Color::Blanc,
    };

    let mut score = 0;

    if king_has_castled(board, color) {
        score += CASTLED_KING_BONUS;
    }

    score += pawn_shield_score(board, color);
    score -= file_pressure_near_king(board, color);
    score -= attacks_on_king_zone(board, enemy, zone) * KING_ZONE_ATTACK_PENALTY;

    score
}

fn evaluation_securite_roi(board: &CBoard) -> EvalScore {
    let mg = king_safety_side(board, Color::Blanc) - king_safety_side(board, Color::Noir);

    EvalScore { mg, eg: 0 }
}
```

## Remplacer l'ancien bonus de roque

Placement: dans `evaluation_blanc`, supprimer ou commenter:

```rust
score.mg += evaluation_roque(board);
```

Puis ajouter:

```rust
score.add(evaluation_securite_roi(board));
```

Apres validation, supprimer l'ancienne fonction `evaluation_roque`, car la securite du roi couvre deja le roque.

## Verification

- Roi roque avec bouclier de pions: meilleur.
- Roi sur colonne ouverte: moins bon.
- En finale, le malus doit disparaitre naturellement car `evaluation_securite_roi` met `eg: 0`.

Risque: moyen, car les attaques autour du roi peuvent etre bruitees.  
Difficulte: moyenne.  
Gain attendu: le moteur evite mieux de laisser son roi expose.

# 7. Tours sur colonnes ouvertes et semi-ouvertes

## Definitions

Une colonne ouverte ne contient aucun pion, blanc ou noir.  
Une colonne semi-ouverte pour les blancs ne contient aucun pion blanc, mais contient au moins un pion noir. Pour les noirs, c'est l'inverse.

Une tour aime les colonnes ouvertes car elle controle toute la colonne. La 7e rangee est aussi forte: une tour blanche sur la 7e attaque souvent les pions noirs et limite le roi noir.

## Code a ajouter

Placement: apres `evaluation_securite_roi`.

```rust
const ROOK_OPEN_FILE_BONUS: i32 = 25;
const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 15;
const ROOK_SEVENTH_RANK_BONUS: i32 = 20;

fn is_open_file(board: &CBoard, file: u8) -> bool {
    let pawns =
        board.piece_bb[Pieces::PionBlanc as usize] | board.piece_bb[Pieces::PionNoir as usize];
    pawns & file_mask(file) == 0
}

fn is_semi_open_file_for_color(board: &CBoard, file: u8, color: Color) -> bool {
    let friendly_pawn = match color {
        Color::Blanc => Pieces::PionBlanc,
        Color::Noir => Pieces::PionNoir,
    };
    let enemy_pawn = match color {
        Color::Blanc => Pieces::PionNoir,
        Color::Noir => Pieces::PionBlanc,
    };
    let mask = file_mask(file);

    board.piece_bb[friendly_pawn as usize] & mask == 0
        && board.piece_bb[enemy_pawn as usize] & mask != 0
}

fn rook_file_score_side(board: &CBoard, color: Color) -> i32 {
    let rook = match color {
        Color::Blanc => Pieces::TourBlanche,
        Color::Noir => Pieces::TourNoire,
    };
    let seventh_rank = match color {
        Color::Blanc => 6,
        Color::Noir => 1,
    };

    let mut score = 0;
    let mut rooks = board.piece_bb[rook as usize];
    while let Some(square) = pop_lsb(&mut rooks) {
        let file = square % 8;
        let rank = square / 8;

        if is_open_file(board, file) {
            score += ROOK_OPEN_FILE_BONUS;
        } else if is_semi_open_file_for_color(board, file, color) {
            score += ROOK_SEMI_OPEN_FILE_BONUS;
        }

        if rank == seventh_rank {
            score += ROOK_SEVENTH_RANK_BONUS;
        }
    }

    score
}

fn evaluation_tours_colonnes(board: &CBoard) -> EvalScore {
    let score = rook_file_score_side(board, Color::Blanc) - rook_file_score_side(board, Color::Noir);

    EvalScore {
        mg: score,
        eg: score,
    }
}
```

## Brancher dans `evaluation_blanc`

Placement: apres `score.add(evaluation_mobilite(board));`.

Ajouter:

```rust
score.add(evaluation_tours_colonnes(board));
```

## Verification

- Tour sur colonne ouverte > tour bloquee par son propre pion.
- Tour sur 7e rangee > tour passive.

Risque: faible.  
Difficulte: faible.  
Gain attendu: meilleur placement des tours.

# 8. Developpement en ouverture

## Idee

En ouverture, il faut sortir les cavaliers et les fous, roquer, et eviter de sortir la dame trop tot.

Cette evaluation ne doit pas peser en finale. Si vous utilisez deja `EvalScore`, mettez ce bonus uniquement dans `mg`. L'interpolation le reduira automatiquement quand la phase baisse.

## Code a ajouter

Placement: apres `evaluation_tours_colonnes`.

```rust
const DEVELOPED_MINOR_BONUS: i32 = 10;
const UNCASTLED_OPENING_PENALTY: i32 = 20;
const EARLY_QUEEN_PENALTY: i32 = 15;

fn undeveloped_minors(board: &CBoard, color: Color) -> i32 {
    let initial_squares: &[(Pieces, u8)] = match color {
        Color::Blanc => &[
            (Pieces::CavalierBlanc, 1),
            (Pieces::CavalierBlanc, 6),
            (Pieces::FouBlanc, 2),
            (Pieces::FouBlanc, 5),
        ],
        Color::Noir => &[
            (Pieces::CavalierNoir, 57),
            (Pieces::CavalierNoir, 62),
            (Pieces::FouNoir, 58),
            (Pieces::FouNoir, 61),
        ],
    };

    initial_squares
        .iter()
        .filter(|(piece, square)| board.piece_bb[*piece as usize] & (1u64 << *square) != 0)
        .count() as i32
}

fn development_side(board: &CBoard, color: Color) -> i32 {
    let undeveloped = undeveloped_minors(board, color);
    let developed = 4 - undeveloped;
    let queen_start = match color {
        Color::Blanc => (Pieces::DameBlanche, 3),
        Color::Noir => (Pieces::DameNoire, 59),
    };

    let mut score = developed * DEVELOPED_MINOR_BONUS;

    if !king_has_castled(board, color) {
        score -= UNCASTLED_OPENING_PENALTY;
    }

    let queen_left_start =
        board.piece_bb[queen_start.0 as usize] & (1u64 << queen_start.1) == 0;
    if queen_left_start && undeveloped > 0 {
        score -= EARLY_QUEEN_PENALTY * undeveloped;
    }

    score
}

fn evaluation_developpement(board: &CBoard) -> EvalScore {
    let mg = development_side(board, Color::Blanc) - development_side(board, Color::Noir);

    EvalScore { mg, eg: 0 }
}
```

## Brancher dans `evaluation_blanc`

Placement: apres la securite du roi.

Ajouter:

```rust
score.add(evaluation_developpement(board));
```

## Verification

- En position d'ouverture, les pieces mineures developpees doivent augmenter le score.
- En finale, l'effet doit etre faible ou nul grace a `eg: 0`.

Risque: moyen, parce qu'une mauvaise ponderation peut pousser le moteur a developper sans calculer les menaces.  
Difficulte: faible.  
Gain attendu: meilleur jeu d'ouverture sans livre d'ouvertures.

# 9. Tests d'evaluation

## Emplacement recommande

Apres creation de `src/evaluation.rs` et declaration dans `src/lib.rs`, creer:

```text
tests/evaluation_tests.rs
```

Ces tests sont des tests d'integration. Ils n'ont acces qu'aux fonctions publiques. Il faut donc que `evaluation_blanc` soit `pub`.

## Code exact du fichier

Placement: creer `tests/evaluation_tests.rs`, puis coller:

```rust
use moteur_ia::evaluation::evaluation_blanc;
use moteur_ia::fen::board_from_fen;

fn score(fen: &str) -> i32 {
    let board = board_from_fen(fen).unwrap();
    evaluation_blanc(&board)
}

#[test]
fn cavalier_au_centre_meilleur_qu_au_bord() {
    let bord = score("7k/8/8/8/8/8/N7/K7 w - - 0 1");
    let centre = score("7k/8/8/8/3N4/8/8/K7 w - - 0 1");

    assert!(centre > bord);
}

#[test]
fn paire_de_fous_meilleure_qu_un_seul_fou() {
    let un_fou = score("7k/8/8/8/8/8/8/K2B4 w - - 0 1");
    let deux_fous = score("7k/8/8/8/8/8/8/K2BB3 w - - 0 1");

    assert!(deux_fous > un_fou);
}

#[test]
fn roi_roque_meilleur_qu_un_roi_non_roque_en_milieu_de_jeu() {
    let non_roque = score("6k1/8/8/8/8/8/5PPP/4K3 w K - 0 1");
    let roque = score("6k1/8/8/8/8/8/5PPP/6K1 w - - 0 1");

    assert!(roque > non_roque);
}

#[test]
fn pion_passe_meilleur_qu_un_pion_bloque_par_pion_adverse() {
    let non_passe = score("7k/4p3/8/4P3/8/8/8/4K3 w - - 0 1");
    let passe = score("7k/p7/8/4P3/8/8/8/4K3 w - - 0 1");

    assert!(passe > non_passe);
}

#[test]
fn pion_isole_moins_bon_qu_un_pion_soutenable() {
    let soutenable = score("7k/8/8/8/4P3/3P4/8/4K3 w - - 0 1");
    let isole = score("7k/8/8/8/4P3/P7/8/4K3 w - - 0 1");

    assert!(soutenable > isole);
}

#[test]
fn tour_sur_colonne_ouverte_meilleure_qu_avec_pion_devant() {
    let colonne_bloquee = score("7k/7p/8/8/8/8/4P3/K3R3 w - - 0 1");
    let colonne_ouverte = score("7k/7p/8/8/8/8/P7/K3R3 w - - 0 1");

    assert!(colonne_ouverte > colonne_bloquee);
}

#[test]
fn en_finale_roi_actif_meilleur_que_roi_passif() {
    let passif = score("7k/8/8/8/8/8/8/K7 w - - 0 1");
    let actif = score("7k/8/8/8/4K3/8/8/8 w - - 0 1");

    assert!(actif > passif);
}
```

## Ce que garantit chaque test

- `cavalier_au_centre_meilleur_qu_au_bord`: les tables de cavaliers fonctionnent.
- `paire_de_fous_meilleure_qu_un_seul_fou`: le bonus de paire de fous reste actif.
- `roi_roque_meilleur_qu_un_roi_non_roque_en_milieu_de_jeu`: la securite du roi valorise un roi abrite.
- `pion_passe_meilleur_qu_un_pion_bloque_par_pion_adverse`: la detection de pion passe fonctionne.
- `pion_isole_moins_bon_qu_un_pion_soutenable`: les pions isoles sont penalises.
- `tour_sur_colonne_ouverte_meilleure_qu_avec_pion_devant`: les colonnes ouvertes sont detectees.
- `en_finale_roi_actif_meilleur_que_roi_passif`: la table de roi finale est branchee.

## Commande de verification

```text
cargo test evaluation_tests
```

Risque: faible.  
Difficulte: faible.  
Gain attendu: chaque nouvelle feature d'evaluation devient verifiable.

# 10. Ponderation des scores

## Unite: centipawns

Le moteur utilise deja:

```text
Pion = 100
Cavalier = 320
Fou = 330
Tour = 500
Dame = 900
```

Les bonus doivent rester inferieurs au materiel, sauf cas tactiques evidents que l'evaluation statique ne doit pas essayer de resoudre seule.

## Valeurs de depart recommandees

| Feature | Valeur initiale |
|---|---:|
| Paire de fous | 30 |
| Cavalier central vs bord | 40 a 70 d'ecart |
| Pion isole | -15 |
| Pion double | -12 par pion supplementaire |
| Pion arriere simple | -10 |
| Pion passe rang 4 | +30 |
| Pion passe rang 5 | +55 |
| Pion passe rang 6 | +90 |
| Pion passe rang 7 | +140 |
| Mobilite cavalier | +4 par case |
| Mobilite fou | +4 par case |
| Mobilite tour | +2 par case |
| Mobilite dame | +1 par case |
| Tour colonne ouverte | +25 |
| Tour colonne semi-ouverte | +15 |
| Tour 7e rangee | +20 |
| Roi roque | +25 |
| Pion de bouclier | +8 |
| Colonne ouverte pres du roi | -18 |
| Developpement piece mineure | +10 |
| Dame sortie trop tot | -15 par piece mineure non developpee |

## Comment detecter une feature trop forte

Un bonus est trop fort si:

- le moteur sacrifie un pion sans compensation tactique claire pour obtenir seulement ce bonus;
- le moteur refuse un gain materiel parce qu'une table de cases donne trop;
- en analyse de plusieurs positions, le score change violemment pour un petit deplacement non critique;
- deux tests contradictoires deviennent difficiles a satisfaire.

Regle pratique:

- bonus faible: `5..15`;
- bonus moyen: `20..40`;
- bonus fort: `50..100`;
- au-dessus de `100`, attention: c'est au moins la valeur d'un pion.

## Methode de reglage

1. Ajouter une feature.
2. Ajouter au moins un test avec FEN.
3. Tester quelques positions a la main avec `evaluation_blanc`.
4. Lancer une partie contre une version precedente.
5. Ajuster par petits pas: `5`, `10`, rarement `20`.

# 11. Plan d'integration recommande

```text
Etape 1 : creer src/evaluation.rs et y deplacer l'evaluation statique
Etape 2 : declarer pub mod evaluation dans src/lib.rs
Etape 3 : faire appeler crate::evaluation::evaluation_negamax depuis src/eval.rs
Etape 4 : ajouter EvalScore, game_phase et blend_score
Etape 5 : remplacer les tables pion/cavalier par les tables completes MG/EG
Etape 6 : ajouter les tests d'evaluation existants puis les nouveaux tests FEN
Etape 7 : ajouter la structure de pions
Etape 8 : ajouter la mobilite legere par masques
Etape 9 : remplacer evaluation_roque par evaluation_securite_roi
Etape 10 : ajouter tours sur colonnes ouvertes/semi-ouvertes
Etape 11 : ajouter developpement en ouverture
Etape 12 : regler les poids progressivement
```

## Detail par etape

| Etape | Risque | Difficulte | Gain attendu | Verification |
|---|---|---:|---|---|
| Deplacer l'evaluation | Faible | 1/5 | Code plus propre | `cargo test` |
| Ajouter phase | Moyen | 3/5 | Score adapte a la partie | test phase initiale/finale |
| Tables completes | Moyen | 3/5 | Meilleures cases pour toutes les pieces | tests cavalier et roi actif |
| Structure de pions | Moyen | 3/5 | Meilleurs plans de pions | tests pion passe/isole |
| Mobilite | Moyen | 3/5 | Pieces plus actives | positions avec pieces bloquees |
| Securite du roi | Moyen | 3/5 | Roi moins expose | test roi roque |
| Tours colonnes | Faible | 2/5 | Meilleures tours | test colonne ouverte |
| Developpement | Moyen | 2/5 | Ouvertures plus naturelles | positions d'ouverture |
| Reglage poids | Moyen | 4/5 | Evaluation plus stable | parties comparatives |

# Resultat attendu apres cette roadmap

Apres cette roadmap, le moteur n'evaluera plus seulement le materiel, quelques cases de cavaliers/pions, la paire de fous et le roque. Il saura mieux distinguer les positions selon la phase de jeu.

Concretement, il devrait:

- preferer des pieces developpees et actives en ouverture;
- mieux valoriser les cavaliers au centre, les fous actifs, les tours ouvertes et les dames bien placees;
- proteger davantage son roi en milieu de jeu;
- reduire naturellement l'importance de la securite du roi en finale;
- activer son roi en finale;
- comprendre les pions isoles, doubles, passes et arrieres simples;
- valoriser les tours sur colonnes ouvertes ou semi-ouvertes;
- produire des scores plus explicables et plus faciles a tester.

Le plus important: chaque ajout reste incremental. Vous pouvez integrer une section, lancer les tests, ajuster les poids, puis passer a la suivante sans devoir reecrire tout le moteur.
