# Plan d'action jour par jour vers une IA d'echecs

Objectif: transformer le moteur actuel en programme d'echecs capable de jouer une partie complete, puis ajouter une IA simple avec minimax et alpha-beta.

Important: ce document donne un plan et des bouts de code pour guider le travail. Les exemples ne sont pas forcement a copier exactement. Il faut les adapter aux noms et aux types du projet.

Regle de progression: a la fin de chaque journee, lancer au minimum:

```bash
cargo check
cargo test
```

## Etat actuel

Le moteur sait deja:

- representer le plateau avec des bitboards;
- charger une position FEN;
- generer les coups pseudo-legaux;
- filtrer les coups legaux;
- verifier si un roi est en echec;
- jouer les coups avec `make_move`;
- gerer roque, promotion et prise en passant;
- faire des tests `perft`.

Ce qui manque avant une IA propre:

- detection de mat;
- detection de pat;
- regle des 50 coups;
- repetition trois fois de la meme position;
- historique de partie;
- evaluation;
- recherche IA.

## Jour 1: creer un etat de partie simple

But: avoir un type qui dit si la partie continue, si c'est mat ou si c'est pat.

Fichier conseille:

- creer plus tard un fichier comme `src/game_state.rs` ou `src/partie.rs`.

Idee de type:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatPartie {
    EnCours,
    Mat,
    Pat,
}
```

Idee de fonction:

```rust
pub fn etat_partie_simple(
    board: &mut CBoard,
    tables: &AttackTables,
) -> EtatPartie {
    let coups = generate_legal_move(board, tables);

    if !coups.is_empty() {
        return EtatPartie::EnCours;
    }

    if is_king_in_check(board, tables, board.side_to_move) {
        EtatPartie::Mat
    } else {
        EtatPartie::Pat
    }
}
```

Attention:

- `generate_legal_move` prend `&mut CBoard`;
- il faudra peut-etre ajouter des `derive` sur certains types pour les tests;
- ne pas melanger encore les nulles par repetition et 50 coups.

Objectif de fin de journee:

- une position normale retourne `EnCours`;
- une position de mat retourne `Mat`;
- une position de pat retourne `Pat`.

## Jour 2: ajouter des tests pour mat et pat

But: verifier que l'etat de partie fonctionne.

FEN de mat possible:

```text
7k/7Q/6K1/8/8/8/8/8 b - - 0 1
```

Idee de test:

```rust
#[test]
fn test_position_mat() {
    let tables = init_attack_tables();
    let mut board = board_from_fen(
        "7k/7Q/6K1/8/8/8/8/8 b - - 0 1"
    ).unwrap();

    assert_eq!(etat_partie_simple(&mut board, &tables), EtatPartie::Mat);
}
```

FEN de pat possible:

```text
7k/5K2/6Q1/8/8/8/8/8 b - - 0 1
```

Idee de test:

```rust
#[test]
fn test_position_pat() {
    let tables = init_attack_tables();
    let mut board = board_from_fen(
        "7k/5K2/6Q1/8/8/8/8/8 b - - 0 1"
    ).unwrap();

    assert_eq!(etat_partie_simple(&mut board, &tables), EtatPartie::Pat);
}
```

Test de position en cours:

```rust
#[test]
fn test_position_initiale_en_cours() {
    let tables = init_attack_tables();
    let mut board = board_from_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    assert_eq!(etat_partie_simple(&mut board, &tables), EtatPartie::EnCours);
}
```

Objectif de fin de journee:

- les tests mat, pat et en cours passent;
- les anciens tests `perft` passent encore.

## Jour 3: mettre a jour le compteur des 50 coups

But: utiliser correctement `halfmove_clock`.

Regle:

- si un pion bouge, le compteur revient a 0;
- si une capture arrive, le compteur revient a 0;
- sinon, le compteur augmente de 1.

Exemple de logique a placer dans la fonction qui joue un coup:

```rust
let reset_halfmove =
    matches!(mv.piece, Pieces::PionBlanc | Pieces::PionNoir)
    || mv.captured.is_some()
    || matches!(mv.flag, MoveFlag::EnPassant);

if reset_halfmove {
    board.halfmove_clock = 0;
} else {
    board.halfmove_clock += 1;
}
```

Attention avec `fullmove_number`:

- le numero de coup complet augmente apres un coup noir;
- il faut donc connaitre la couleur qui vient de jouer avant d'inverser `side_to_move`.

Exemple:

```rust
let couleur_avant_coup = board.side_to_move;

// jouer le coup ici

if matches!(couleur_avant_coup, Color::Noir) {
    board.fullmove_number += 1;
}
```

Objectif de fin de journee:

- un coup calme augmente `halfmove_clock`;
- un coup de pion remet `halfmove_clock` a 0;
- une capture remet `halfmove_clock` a 0;
- les tests `perft` passent encore.

## Jour 4: ajouter la nulle par regle des 50 coups

But: ajouter un nouvel etat de partie.

Nouveau type:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatPartie {
    EnCours,
    Mat,
    Pat,
    Nulle50Coups,
}
```

Exemple dans la detection:

```rust
if board.halfmove_clock >= 100 {
    return EtatPartie::Nulle50Coups;
}
```

Ordre conseille dans `etat_partie`:

1. verifier la regle des 50 coups;
2. generer les coups legaux;
3. detecter mat ou pat;
4. sinon retourner `EnCours`.

Idee de test:

```rust
#[test]
fn test_nulle_50_coups() {
    let tables = init_attack_tables();
    let mut board = board_from_fen(
        "7k/8/8/8/8/8/8/K7 w - - 100 80"
    ).unwrap();

    assert_eq!(etat_partie(&mut board, &tables), EtatPartie::Nulle50Coups);
}
```

Objectif de fin de journee:

- `halfmove_clock >= 100` donne une nulle;
- mat et pat fonctionnent toujours;
- les tests `perft` passent encore.

## Jour 5: creer une structure de partie

But: separer une position et une partie complete.

`CBoard` represente une position. Une partie doit contenir aussi l'historique.

Idee de structure:

```rust
pub struct Partie {
    pub board: CBoard,
    pub tables: AttackTables,
    pub coups_joues: Vec<Move>,
}
```

Idee de constructeur:

```rust
impl Partie {
    pub fn depuis_fen(fen: &str) -> Result<Self, String> {
        Ok(Self {
            board: board_from_fen(fen)?,
            tables: init_attack_tables(),
            coups_joues: Vec::new(),
        })
    }
}
```

Idee de methode pour jouer:

```rust
impl Partie {
    pub fn jouer_coup(&mut self, mv: Move) {
        make_move(&mut self.board, mv);
        self.coups_joues.push(mv);
    }
}
```

Objectif de fin de journee:

- une `Partie` peut etre creee depuis une FEN;
- elle peut jouer un coup legal;
- elle garde la liste des coups joues.

## Jour 6: creer une cle de position pour les repetitions

But: identifier une position sans tenir compte de `halfmove_clock` ni `fullmove_number`.

Une repetition depend de:

- pieces sur le plateau;
- couleur au trait;

- droits de roque;
- case en passant possible.

Elle ne depend pas de:

- `halfmove_clock`;
- `fullmove_number`.

Idee de cle:

```rust
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ClePosition {
    pub piece_bb: [u64; 14],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant_square: Option<u8>,
}
```

Idee de fonction:

```rust
pub fn cle_position(board: &CBoard) -> ClePosition {
    ClePosition {
        piece_bb: board.piece_bb,
        side_to_move: board.side_to_move,
        castling_rights: board.castling_rights,
        en_passant_square: board.en_passant_square,
    }
}
```

Attention:

- pour utiliser `Hash`, `PartialEq` et `Eq`, il faudra peut-etre ajouter des derives a `Color`;
- plus tard, cette cle pourra etre remplacee par un hash Zobrist.

Objectif de fin de journee:

- deux positions identiques donnent la meme cle;
- deux positions avec un trait different donnent deux cles differentes;
- deux positions avec droits de roque differents donnent deux cles differentes.

## Jour 7: detecter la repetition trois fois

But: compter combien de fois chaque position est apparue.

Ajouter dans `Partie`:

```rust
use std::collections::HashMap;

pub struct Partie {
    pub board: CBoard,
    pub tables: AttackTables,
    pub coups_joues: Vec<Move>,
    pub repetitions: HashMap<ClePosition, u32>,
}
```

Quand la partie commence:

```rust
let mut repetitions = HashMap::new();
let cle = cle_position(&board);
repetitions.insert(cle, 1);
```

Apres chaque coup:

```rust
let cle = cle_position(&self.board);
let compteur = self.repetitions.entry(cle).or_insert(0);
*compteur += 1;
```

Nouvel etat:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatPartie {
    EnCours,
    Mat,
    Pat,
    Nulle50Coups,
    NulleRepetition,
}
```

Detection:

```rust
if self.repetitions.get(&cle_position(&self.board)).copied().unwrap_or(0) >= 3 {
    return EtatPartie::NulleRepetition;
}
```

Objectif de fin de journee:

- une position vue trois fois donne `NulleRepetition`;
- une position vue deux fois ne donne pas encore nulle;
- les anciens tests passent encore.

## Jour 8: lire un coup humain en notation

But: transformer une entree comme `e2e4` en vrai `Move` legal.

Principe:

1. lire la chaine;
2. convertir `from` et `to`;
3. generer les coups legaux;
4. chercher un coup legal qui correspond.

Exemple:

```rust
pub fn trouver_coup_legal(
    board: &mut CBoard,
    tables: &AttackTables,
    texte: &str,
) -> Option<Move> {
    if texte.len() < 4 {
        return None;
    }

    let from = coord_to_square_index(&texte[0..2]).ok()?;
    let to = coord_to_square_index(&texte[2..4]).ok()?;
    let promotion_demandee = if texte.len() == 5 {
        let lettre = texte.chars().nth(4)?;

        match (lettre, board.side_to_move) {
            ('q' | 'Q', Color::Blanc) => Some(Pieces::DameBlanche),
            ('r' | 'R', Color::Blanc) => Some(Pieces::TourBlanche),
            ('b' | 'B', Color::Blanc) => Some(Pieces::FouBlanc),
            ('n' | 'N', Color::Blanc) => Some(Pieces::CavalierBlanc),

            ('q' | 'Q', Color::Noir) => Some(Pieces::DameNoire),
            ('r' | 'R', Color::Noir) => Some(Pieces::TourNoire),
            ('b' | 'B', Color::Noir) => Some(Pieces::FouNoir),
            ('n' | 'N', Color::Noir) => Some(Pieces::CavalierNoir),

            _ => return None,
        }
    } else {
        None
    };
    generate_legal_move(board, tables)
        .into_iter()
        .find(|mv| mv.from == from && mv.to == to)
}
```

Attention:

- les promotions ont parfois une lettre en plus, par exemple `e7e8q`;
- il faudra comparer aussi `promotion` si le texte contient 5 caracteres.

Objectif de fin de journee:

- `e2e4` trouve un coup legal depuis la position initiale;
- `e2e5` est refuse;
- une promotion comme `a7a8q` peut etre reconnue plus tard.

## Jour 9: creer une boucle de partie simple

But: faire jouer une partie dans le terminal.

Schema:

```rust
loop {
    affichage_position_complete(&partie.board);

    match partie.etat() {
        EtatPartie::EnCours => {}
        EtatPartie::Mat => {
            println!("Echec et mat");
            break;
        }
        EtatPartie::Pat => {
            println!("Pat");
            break;
        }
        EtatPartie::Nulle50Coups => {
            println!("Nulle par regle des 50 coups");
            break;
        }
        EtatPartie::NulleRepetition => {
            println!("Nulle par repetition");
            break;
        }
    }

    // lire un coup humain, verifier, jouer
}
```

Objectif de fin de journee:

- la boucle affiche la position;
- elle accepte un coup legal;
- elle refuse un coup illegal;
- elle s'arrete sur mat, pat ou nulle.

## Jour 10: creer une evaluation materielle

But: donner un score a une position.

Valeurs simples:

- pion = 100;
- cavalier = 320;
- fou = 330;
- tour = 500;
- dame = 900;
- roi = 0.

Exemple:

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
```

Convention conseillee:

- score positif = avantage blanc;
- score negatif = avantage noir.

Objectif de fin de journee:

- la position initiale vaut 0;
- une position ou les blancs ont une dame en plus est positive;
- une position ou les noirs ont une dame en plus est negative.

## Jour 11: faire un minimax simple

But: choisir un coup automatiquement.

Idee:

```rust
pub fn minimax(board: &mut CBoard, tables: &AttackTables, depth: u32) -> i32 {
    if depth == 0 {
        return evaluation_materielle(board);
    }

    let coups = generate_legal_move(board, tables);

    if coups.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -100000;
        }
        return 0;
    }
    
    let mut meilleur = -1000000;

    for mv in coups {
        let old_board = board.clone();
        make_move(board, mv);
        let score = -minimax(board, tables, depth - 1);
        *board = old_board;

        if score > meilleur {
            meilleur = score;
        }
    }

    meilleur
}
```

Remarque:

- cette forme utilise le style `negamax`, plus simple que minimax classique;
- le score doit etre pense du point de vue du joueur qui doit jouer.

Objectif de fin de journee:

- l'IA peut choisir un coup a profondeur 1;
- elle prefere une capture gagnante a un coup calme;
- elle ne joue jamais un coup illegal.

## Jour 12: choisir le meilleur coup

But: retourner le `Move`, pas seulement le score.

Exemple:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    let coups = generate_legal_move(board, tables);
    let mut meilleur_mv = None;
    let mut meilleur_score = -1000000;

    for mv in coups {
        let old_board = board.clone();
        make_move(board, mv);
        let score = -minimax(board, tables, depth - 1);
        *board = old_board;

        if score > meilleur_score {
            meilleur_score = score;
            meilleur_mv = Some(mv);
        }
    }

    meilleur_mv
}
```

Objectif de fin de journee:

- `meilleur_coup` retourne un coup legal;
- le programme peut jouer humain contre IA;
- profondeur 2 fonctionne sur une position simple.

## Jour 13: ajouter alpha-beta

But: accelerer la recherche.

Exemple:

```rust
pub fn negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if depth == 0 {
        return evaluation_materielle(board);
    }

    let coups = generate_legal_move(board, tables);

    if coups.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -100000;
        }
        return 0;
    }

    let mut meilleur = -1000000;

    for mv in coups {
        let old_board = board.clone();
        make_move(board, mv);
        let score = -negamax_alpha_beta(board, tables, depth - 1, -beta, -alpha);
        *board = old_board;

        meilleur = meilleur.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;
        }
    }

    meilleur
}
```

Objectif de fin de journee:

- alpha-beta donne les memes choix que minimax sur petites profondeurs;
- la recherche devient plus rapide;
- profondeur 3 devient plus confortable.

## Jour 14: ordonner les coups

But: rendre alpha-beta plus efficace.

Idee simple:

- captures d'abord;
- promotions ensuite;
- coups calmes apres.

Exemple:

```rust
pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::Promotion | MoveFlag::PromotionCapture => 1000,
        MoveFlag::Capture | MoveFlag::EnPassant => 500,
        MoveFlag::Castling => 100,
        _ => 0,
    }
}
```

Trier avant la recherche:

```rust
let mut coups = generate_legal_move(board, tables);
coups.sort_by_key(|mv| -score_ordre_coup(mv));
```

Objectif de fin de journee:

- alpha-beta visite moins de positions;
- l'IA reste correcte;
- les tests passent encore.

# Plan d'evolution du moteur d'echecs a partir de l'etape 15

Objectif: faire evoluer le moteur actuel sans repartir de zero.

Tu as deja une base fonctionnelle pour l'IA:

- evaluation materielle;
- evaluation adaptee au negamax;
- recherche negamax avec alpha-beta;
- choix du meilleur coup;
- ordre simple des coups;
- generation des coups legaux;
- detection de l'echec;
- `make_move` avec restauration par `board.clone()`.

Le but maintenant n'est donc pas de refaire les etapes 10 a 14, mais de les nettoyer legerement puis de faire evoluer l'evaluation vers une structure plus propre:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);

    score
}
```

## Etat actuel du code

Tu as actuellement quelque chose du type:

```rust
use crate::attack_tables::AttackTables;
use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::{Move, MoveFlag};
use crate::legal_move::generate_legal_move;
use crate::legality::is_king_in_check;
use crate::make_move::make_move;
```

Avec une evaluation materielle:

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

    for (pieces, valeur) in valeurs {
        let nombre = board.piece_bb[pieces as usize].count_ones() as i32;
        score += nombre * valeur;
    }

    score
}
```

Et une fonction adaptee au negamax:

```rust
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_materielle(board);

    if board.side_to_move == Color::Blanc {
        return score;
    } else {
        return -score;
    }
}
```

Cette logique est bonne dans l'idee.

La convention actuelle est:

```text
score positif = avantage blanc
score negatif = avantage noir
```

Puis `evaluation_negamax` transforme ce score pour qu'il soit lu du point de vue du joueur qui doit jouer.

C'est exactement ce qu'il faut garder.

## Probleme actuel

Le probleme est que `evaluation_negamax` appelle directement:

```rust
let score = evaluation_materielle(board);
```

Donc ton IA ne regarde que le materiel.

Elle peut donc:

- trop aimer les captures;
- mal placer ses cavaliers;
- ne pas chercher a roquer;
- ne pas comprendre qu'une piece au centre est meilleure qu'une piece au bord;
- faire des coups legalement corrects mais positionnellement faibles.

La solution est de creer une couche intermediaire:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32
```

Cette fonction additionnera plusieurs criteres.

Ensuite `evaluation_negamax` n'appellera plus `evaluation_materielle`, mais `evaluation_blanc`.

## Etape 15.1 - Renommer mentalement la recherche

Ta fonction s'appelle probablement:

```rust
pub fn minimax(...)
```

Mais en realite, avec cette ligne:

```rust
let score = -minimax(board, tables, depth - 1, -beta, -alpha);
```

ce n'est pas un minimax classique.

C'est un negamax avec alpha-beta.

Tu peux donc renommer la fonction pour plus de clarte:

```rust
pub fn negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    // code actuel de minimax
}
```

Ce changement n'est pas obligatoire pour que le code marche, mais il rend le projet plus lisible.

Si tu renommes la fonction, il faut aussi remplacer les appels:

```rust
-minimax(board, tables, depth - 1, -beta, -alpha)
```

par:

```rust
-negamax_alpha_beta(board, tables, depth - 1, -beta, -alpha)
```

## Etape 15.2 - Ajouter des constantes propres

Actuellement, tu as des valeurs comme:

```rust
-10000
100000
```

C'est mieux de creer des constantes.

En haut du fichier IA ou evaluation:

```rust
const INF: i32 = 1_000_000;
const SCORE_MAT: i32 = 100_000;
```

Puis dans la recherche:

```rust
let mut meilleure = -INF;
```

Et pour le mat:

```rust
return -SCORE_MAT + depth as i32;
```

Pourquoi `+ depth as i32` ?

Parce que cela permet de preferer les mats rapides.

Exemple:

```text
mater en 1 doit etre meilleur que mater en 3
etre mate en 5 doit etre moins mauvais qu'etre mate en 1
```

## Etape 15.3 - Corriger legerement `meilleur_coup`

Dans ton code actuel, tu as deja une fonction du type:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| -score_ordre_coup(mv));

    let mut meilleur_mv = None;
    let mut meilleur_score = -100000;
    let mut alpha = -100000;
    let mut beta = 100000;

    for mv in coups {
        let old_board = board.clone();
        make_move(board, mv);
        let score = -minimax(board, tables, depth - 1, -beta, -alpha);
        *board = old_board;

        if score > meilleur_score {
            meilleur_mv = Some(mv);
            meilleur_score = score;
        }
    }

    meilleur_mv
}
```

Il faut corriger deux details.

Premier detail: `beta` n'a pas besoin d'etre mutable.

```rust
let beta = INF;
```

Deuxieme detail: apres chaque coup teste, tu peux mettre a jour `alpha`.

```rust
alpha = alpha.max(score);
```

Version corrigee:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| -score_ordre_coup(mv));

    let mut meilleur_mv = None;
    let mut meilleur_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for mv in coups {
        let old_board = board.clone();

        make_move(board, mv);
        let score = -negamax_alpha_beta(board, tables, depth - 1, -beta, -alpha);
        *board = old_board;

        if score > meilleur_score {
            meilleur_mv = Some(mv);
            meilleur_score = score;
        }

        alpha = alpha.max(score);
    }

    meilleur_mv
}
```

## Etape 15.4 - Creer `evaluation_blanc`

C'est l'etape principale.

Actuellement:

```rust
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_materielle(board);

    if board.side_to_move == Color::Blanc {
        return score;
    } else {
        return -score;
    }
}
```

Tu vas remplacer par:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);

    score
}
```

Puis:

```rust
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_blanc(board);

    match board.side_to_move {
        Color::Blanc => score,
        Color::Noir => -score,
    }
}
```

Au debut, `evaluation_blanc` ne fait que rappeler `evaluation_materielle`.

Donc le comportement de ton moteur ne change presque pas.

C'est volontaire.

L'objectif est d'abord de changer la structure proprement, sans modifier toute la force de l'IA d'un coup.

## Etape 15.5 - Ajouter des fonctions utilitaires

Pour evaluer les pieces, tu dois parcourir les bitboards.

Ajoute cette fonction:

```rust
fn pop_lsb(bb: &mut u64) -> Option<u8> {
    if *bb == 0 {
        return None;
    }

    let square = bb.trailing_zeros() as u8;
    *bb &= *bb - 1;

    Some(square)
}
```

Elle sert a extraire les cases occupees une par une.

Exemple:

```rust
let mut bb = board.piece_bb[Pieces::CavalierBlanc as usize];

while let Some(square) = pop_lsb(&mut bb) {
    println!("Cavalier blanc sur la case {}", square);
}
```

Ajoute aussi une fonction miroir pour les noirs.

Avec ton mapping:

```text
0 = a1
1 = b1
...
7 = h1
8 = a2
...
56 = a8
63 = h8
```

La fonction miroir est:

```rust
fn mirror_square(square: u8) -> u8 {
    square ^ 56
}
```

Exemples:

```text
a1 -> a8
b1 -> b8
e2 -> e7
e4 -> e5
```

Cette fonction est importante parce que les tables de cases sont souvent ecrites du point de vue des blancs.

## Etape 15.6 - Ajouter une table de cases pour les cavaliers

Les cavaliers sont les pieces les plus faciles a evaluer positionnellement.

Un cavalier au centre est fort.

Un cavalier au bord est faible.

Ajoute cette table:

```rust
const BONUS_CAVALIER: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];
```

Puis ajoute:

```rust
fn evaluation_cavaliers(board: &CBoard) -> i32 {
    let mut score = 0;

    let mut cavaliers_blancs = board.piece_bb[Pieces::CavalierBlanc as usize];
    while let Some(square) = pop_lsb(&mut cavaliers_blancs) {
        score += BONUS_CAVALIER[square as usize];
    }

    let mut cavaliers_noirs = board.piece_bb[Pieces::CavalierNoir as usize];
    while let Some(square) = pop_lsb(&mut cavaliers_noirs) {
        let mirrored = mirror_square(square);
        score -= BONUS_CAVALIER[mirrored as usize];
    }

    score
}
```

Puis modifie `evaluation_blanc`:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);

    score
}
```

A ce stade, ton IA devrait commencer a preferer les cavaliers vers le centre.

## Etape 15.7 - Ajouter la paire de fous

La paire de fous est un petit bonus positionnel simple.

Ajoute:

```rust
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
```

Puis:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);

    score
}
```

Pourquoi seulement `30` ?

Parce qu'il ne faut pas que le bonus positionnel devienne plus important qu'un pion.

Un pion vaut `100`, donc `30` reste raisonnable.

## Etape 15.8 - Ajouter un bonus de roque simple

Le roque est important pour la securite du roi.

Avec ton mapping:

```text
c1 = 2
g1 = 6
c8 = 58
g8 = 62
```

Tu peux faire:

```rust
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
```

Puis:

```rust
pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);

    score
}
```

Attention: ce bonus est simple.

Il ne verifie pas si le roque est vraiment bon dans la position.

Il dit seulement:

```text
roi deja roque = leger bonus
```

C'est suffisant pour une premiere version.

## Version complete conseillee du fichier IA / evaluation

Tu peux garder tout dans ton fichier actuel au debut.

Plus tard, tu pourras separer en:

```text
src/evaluation.rs
src/search.rs
```

Mais pour l'instant, si ton projet est encore petit, tu peux garder ensemble.

Version propre:

```rust
use crate::attack_tables::AttackTables;
use crate::board::{CBoard, Color, Pieces};
use crate::chess_move::{Move, MoveFlag};
use crate::legal_move::generate_legal_move;
use crate::legality::is_king_in_check;
use crate::make_move::make_move;

const INF: i32 = 1_000_000;
const SCORE_MAT: i32 = 100_000;

const BONUS_CAVALIER: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

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

pub fn evaluation_blanc(board: &CBoard) -> i32 {
    let mut score = 0;

    score += evaluation_materielle(board);
    score += evaluation_cavaliers(board);
    score += evaluation_paire_de_fous(board);
    score += evaluation_roque(board);

    score
}

pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_blanc(board);

    match board.side_to_move {
        Color::Blanc => score,
        Color::Noir => -score,
    }
}

fn evaluation_cavaliers(board: &CBoard) -> i32 {
    let mut score = 0;

    let mut cavaliers_blancs = board.piece_bb[Pieces::CavalierBlanc as usize];
    while let Some(square) = pop_lsb(&mut cavaliers_blancs) {
        score += BONUS_CAVALIER[square as usize];
    }

    let mut cavaliers_noirs = board.piece_bb[Pieces::CavalierNoir as usize];
    while let Some(square) = pop_lsb(&mut cavaliers_noirs) {
        let mirrored = mirror_square(square);
        score -= BONUS_CAVALIER[mirrored as usize];
    }

    score
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

pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::Promotion | MoveFlag::PromotionCapture => 1000,
        MoveFlag::Capture | MoveFlag::EnPassant => 500,
        MoveFlag::Castling => 100,
        _ => 0,
    }
}

pub fn negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if depth == 0 {
        return evaluation_negamax(board);
    }

    let mut moves = generate_legal_move(board, tables);
    moves.sort_by_key(|mv| -score_ordre_coup(mv));

    if moves.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -SCORE_MAT + depth as i32;
        }

        return 0;
    }

    let mut meilleure = -INF;

    for mv in moves {
        let old_board = board.clone();

        make_move(board, mv);
        let score = -negamax_alpha_beta(board, tables, depth - 1, -beta, -alpha);
        *board = old_board;

        meilleure = meilleure.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;
        }
    }

    meilleure
}

pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| -score_ordre_coup(mv));

    let mut meilleur_mv = None;
    let mut meilleur_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for mv in coups {
        let old_board = board.clone();

        make_move(board, mv);
        let score = -negamax_alpha_beta(board, tables, depth - 1, -beta, -alpha);
        *board = old_board;

        if score > meilleur_score {
            meilleur_mv = Some(mv);
            meilleur_score = score;
        }

        alpha = alpha.max(score);
    }

    meilleur_mv
}
```

## Tests a ajouter apres l'etape 15

Il faut tester l'evaluation seule avant de tester l'IA.

Cree par exemple:

```text
tests/evaluation_tests.rs
```

### Test 1 - La position initiale vaut environ 0

Avec les bonus de cavaliers, roque et paire de fous, la position initiale doit rester proche de 0.

Normalement elle vaut exactement 0 si les bonus sont symetriques.

```rust
#[test]
fn evaluation_initiale_zero() {
    let board = board_from_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    assert_eq!(evaluation_blanc(&board), 0);
}
```

### Test 2 - Une dame blanche en plus donne un score positif

```rust
#[test]
fn dame_blanche_en_plus_score_positif() {
    let board = board_from_fen(
        "7k/8/8/8/8/8/8/KQ6 w - - 0 1"
    ).unwrap();

    assert!(evaluation_blanc(&board) > 0);
}
```

### Test 3 - Une dame noire en plus donne un score negatif

```rust
#[test]
fn dame_noire_en_plus_score_negatif() {
    let board = board_from_fen(
        "6qk/8/8/8/8/8/8/K7 w - - 0 1"
    ).unwrap();

    assert!(evaluation_blanc(&board) < 0);
}
```

### Test 4 - Cavalier blanc au centre meilleur que cavalier blanc au bord

```rust
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
```

Si un test ne compile pas, adapte les imports selon ton architecture.

Exemple probable:

```rust
use moteur_ia::fen::board_from_fen;
use moteur_ia::ia::{evaluation_blanc};
```

ou:

```rust
use moteur_ia::evaluation::evaluation_blanc;
```

selon le nom de ton module.

## Ce qu'il ne faut pas encore faire

Ne commence pas tout de suite:

- table de transposition;
- Zobrist;
- UCI;
- gestion du temps;
- evaluation ultra complexe des pions;
- evaluation avancee de la securite du roi;
- moteur d'ouverture;
- null move pruning;
- late move reductions.

Ces elements sont utiles, mais trop tot.

La priorite est:

```text
1. evaluation propre
2. recherche stable
3. tests simples
4. IA qui joue legalement
5. IA qui evite les grosses erreurs tactiques
```

## Plan apres cette evolution

### Etape 16 - MVV-LVA pour l'ordre des captures

Ton ordre actuel met toutes les captures au meme niveau.

Actuellement:

```rust
MoveFlag::Capture | MoveFlag::EnPassant => 500
```

Mais une capture n'a pas toujours la meme valeur.

Exemples:

```text
pion prend dame = tres bon
reine prend pion = pas forcement prioritaire
```

Il faudra ajouter:

```rust
fn valeur_piece(piece: Pieces) -> i32
```

Puis pour une capture:

```rust
score = 10 * valeur_piece(piece_capturee) - valeur_piece(piece_attaquante)
``` 