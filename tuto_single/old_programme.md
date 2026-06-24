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
# Optimiser ton moteur d'échecs Rust pour atteindre une profondeur plus grande

## Objectif

Ton problème actuel n'est pas seulement que la quiescence search existe. Le problème principal est qu'elle coûte trop cher par nœud.

Dans ton code actuel, à chaque feuille de l'alpha-beta, tu appelles :

```rust
return quiescence(board, tables, alpha, beta, 4);
```

Puis dans `quiescence`, tu fais :

```rust
let mut moves = generate_legal_move(board, tables);

moves.retain(|mv| {
    mv.flag == MoveFlag::Capture
        || mv.flag == MoveFlag::EnPassant
        || mv.flag == MoveFlag::PromotionCapture
});
```

Donc le moteur génère tous les coups légaux, y compris les coups calmes, puis les supprime. C'est le point le plus coûteux.

En pratique, ta profondeur réelle ressemble à :

```text
profondeur principale 5
+ quiescence jusqu'à 4 demi-coups tactiques
+ génération complète des coups légaux à chaque nœud de quiescence
```

C'est normal que le temps explose.

Le but de ce document est de te donner un ordre d'optimisation propre pour passer progressivement de :

```text
profondeur 4/5 lente
```

vers :

```text
profondeur 6/7 plus stable
```

puis plus tard :

```text
profondeur 8+ avec table de transposition + iterative deepening
```

## Repères dans ton projet

Dans ce document, quand je dis d'ajouter ou de remplacer du code, voici les fichiers à viser dans ton projet actuel :

```text
src/eval.rs
```

Pour la recherche IA : `SearchStats`, `evaluation_negamax_alpha_beta`, `meilleur_coup`, `quiescence`, `score_ordre_coup`, `SearchLimits`, killer moves et history heuristic.

```text
src/legal_move.rs
```

Pour la génération de coups : `is_tactical_move`, `generate_legal_move`, `generate_tactical_legal_move`.

```text
src/position_key.rs
```

Pour la table de transposition : `ClePosition`, `TTFlag`, `TTEntry`, `TranspositionTable`.

```text
src/make_move.rs
```

Pour la future optimisation `make_move` / `unmake_move`.

```text
src/web_server.rs
```

Seulement si tu veux changer la profondeur jouée par l'IA web ou remplacer `meilleur_coup` par une version iterative deepening.

---

# 1. Priorité absolue : vérifier que tu lances en release

Avant toute optimisation, vérifie que tu ne testes pas ton moteur en mode debug.

À utiliser :

```bash
cargo run --release
```

ou :

```bash
cargo test --release
```

Ne benchmark jamais ton IA avec :

```bash
cargo run
```

Le mode debug Rust peut être énormément plus lent sur un moteur d'échecs, parce que tu fais énormément de récursion, de clones, de génération de coups et de parcours de vecteurs.

Tu peux aussi ajouter dans `Cargo.toml` :

Où le mettre :

```text
Cargo.toml, à la racine du projet, tout en bas du fichier.
```

Si un bloc `[profile.release]` existe déjà, ne le duplique pas : complète le bloc existant.

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
debug = false
```

Ce n'est pas magique, mais c'est une base obligatoire.

---

# 2. Ajouter des compteurs avant d'optimiser

Il ne faut pas optimiser à l'aveugle. Tu dois savoir combien de nœuds ton moteur visite.

Ajoute une structure de statistiques :

Où le mettre :

```text
src/eval.rs, près du début du fichier, juste après les constantes comme SCORE_MAT et INF.
```

```rust
#[derive(Default, Debug, Clone)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,
    pub cutoffs: u64,
    pub qcutoffs: u64,
}
```

Puis modifie progressivement tes fonctions pour recevoir :

Où le mettre :

```text
src/eval.rs, dans la signature de evaluation_negamax_alpha_beta puis dans la signature de quiescence.
```

Chaque fonction qui appelle `evaluation_negamax_alpha_beta` ou `quiescence` devra ensuite transmettre `stats`.

```rust
stats: &mut SearchStats
```

Exemple dans alpha-beta :

```rust
pub fn evaluation_negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;

    if depth == 0 {
        return quiescence(board, tables, alpha, beta, 2, stats);
    }

    // reste du code
}
```

Dans quiescence :

```rust
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
    qdepth: u32,
    stats: &mut SearchStats,
) -> i32 {
    stats.qnodes += 1;

    // reste du code
}
```

Dans `meilleur_coup`, mesure le temps :

Où le mettre :

```text
src/eval.rs, dans la fonction meilleur_coup.
```

Place `let mut stats = SearchStats::default();` et `let start = Instant::now();` au début de `meilleur_coup`, avant la génération des coups.

Place les `println!` à la fin de `meilleur_coup`, juste avant `meilleur_mv`.

L'import suivant se met tout en haut de `src/eval.rs`, avec les autres `use` :

```rust
use std::time::Instant;
```

Code à ajouter dans `meilleur_coup` :

```rust
let mut stats = SearchStats::default();
let start = Instant::now();
```

Puis, dans la boucle de `meilleur_coup`, passe `&mut stats` à l'appel alpha-beta :

```rust
let score = -evaluation_negamax_alpha_beta(
    board,
    tables,
    depth - 1,
    -beta,
    -alpha,
    &mut stats,
);
```

Enfin, juste avant le `return meilleur_mv` ou la dernière ligne `meilleur_mv` :

```rust
let elapsed = start.elapsed();
println!("Temps: {} ms", elapsed.as_millis());
println!("Nodes: {}", stats.nodes);
println!("QNodes: {}", stats.qnodes);
println!("Cutoffs: {}", stats.cutoffs);
println!("QCutoffs: {}", stats.qcutoffs);
```

Ce que tu veux surveiller :

```text
si qnodes >> nodes, alors la quiescence mange tout ton temps.
```

C'est probablement ton cas actuellement.

---

# 3. Première correction immédiate : baisser qdepth

Dans ton code actuel, tu appelles :

Où modifier :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, dans le bloc if depth == 0.
```

```rust
return quiescence(board, tables, alpha, beta, 4);
```

Pour stabiliser, commence avec :

Au même endroit, remplace seulement le dernier argument de profondeur de quiescence :

```rust
return quiescence(board, tables, alpha, beta, 2);
```

Si tu as déjà ajouté `SearchStats`, l'appel aura probablement un argument en plus. Dans ce cas, tu changes seulement `4` en `2` :

```rust
return quiescence(board, tables, alpha, beta, 2, stats);
```

Pourquoi ?

Parce qu'une quiescence profondeur 4 appelée à toutes les feuilles d'une recherche profondeur 5 peut devenir énorme.

Ordre conseillé :

```text
profondeur principale 4 + qdepth 2
profondeur principale 5 + qdepth 2
profondeur principale 6 + qdepth 1 ou 2
```

Ne cherche pas tout de suite :

```text
profondeur principale 5 + qdepth 4
```

Ce n'est pas forcément meilleur, parce que tu passes ton temps dans des suites de captures parfois peu utiles.

---

# 4. Corriger la quiescence : ne pas toujours faire `stand_pat`

Ta version actuelle fait :

```rust
let stand_pat = evaluation_negamax(board);
```

puis autorise indirectement le moteur à dire :

```text
je ne fais rien, j'évalue la position comme ça
```

C'est correct seulement si le roi du joueur au trait n'est pas en échec.

Si le joueur au trait est en échec, il n'a pas le droit de faire “rien”. Il doit sortir de l'échec.

Version plus correcte :

Où le mettre :

```text
src/eval.rs, remplace entièrement la fonction quiescence existante par cette version.
```

Imports nécessaires en haut de `src/eval.rs`, avec les autres `use`, si tu ne les as pas déjà :

```rust
use std::cmp::Reverse;
use crate::legality::is_king_in_check;
use crate::legal_move::generate_legal_move;
```

```rust
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
    qdepth: u32,
    stats: &mut SearchStats,
) -> i32 {
    stats.qnodes += 1;

    let in_check = is_king_in_check(board, tables, board.side_to_move);

    if qdepth == 0 {
        return evaluation_negamax(board);
    }

    if !in_check {
        let stand_pat = evaluation_negamax(board);

        if stand_pat >= beta {
            stats.qcutoffs += 1;
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }
    }

    let mut moves = generate_legal_move(board, tables);

    if !in_check {
        moves.retain(|mv| is_tactical_move(mv));
    }

    moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    if moves.is_empty() {
        if in_check {
            return -SCORE_MAT;
        }
        return alpha;
    }

    for mv in moves {
        let old_board = board.clone();
        make_move(board, mv);

        let score = -quiescence(board, tables, -beta, -alpha, qdepth - 1, stats);

        *board = old_board;

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
```

Et ajoute :

Où le mettre :

```text
src/eval.rs, juste au-dessus de la fonction quiescence si tu fais seulement cette étape.
```

Si tu appliques ensuite l'étape 5, mets plutôt ce helper dans `src/legal_move.rs`, juste au-dessus de `generate_tactical_legal_move`, et rends-le public avec `pub fn`, car il servira aussi au générateur tactique.

```rust
fn is_tactical_move(mv: &Move) -> bool {
    matches!(
        mv.flag,
        MoveFlag::Capture
            | MoveFlag::EnPassant
            | MoveFlag::Promotion
            | MoveFlag::PromotionCapture
    )
}
```

Différence importante :

```text
si le roi n'est pas en échec -> on cherche seulement les captures/promotions
si le roi est en échec -> on cherche tous les coups légaux pour sortir de l'échec
```

---

# 5. Optimisation majeure : ne plus générer tous les coups légaux dans la quiescence

Actuellement, la quiescence fait :

```rust
generate_legal_move(...)
```

puis filtre.

C'est très coûteux.

Ce qu'il faut faire à terme :

```rust
generate_tactical_legal_move(...)
```

Cette fonction doit générer seulement :

```text
captures
promotions
en passant
```

Première version simple si tu as accès à `generate_pseudo_legal_move` :

Où le mettre :

```text
src/legal_move.rs, après is_tactical_move et avant generate_legal_move.
```

Imports nécessaires en haut de `src/legal_move.rs`, si absents :

```rust
use crate::pseudo_legal_move::generate_pseudo_legal_move;
use crate::make_move::make_move;
use crate::legality::is_king_in_check;
```

```rust
pub fn generate_tactical_legal_move(
    board: &mut CBoard,
    tables: &AttackTables,
) -> Vec<Move> {
    let side = board.side_to_move;

    let mut moves = generate_pseudo_legal_move(board, tables);

    moves.retain(|mv| is_tactical_move(mv));

    let mut legal = Vec::new();

    for mv in moves {
        let old_board = board.clone();
        make_move(board, mv);

        if !is_king_in_check(board, tables, side) {
            legal.push(mv);
        }

        *board = old_board;
    }

    legal
}
```

Puis dans `quiescence` :

Où modifier :

```text
src/eval.rs, dans quiescence, à l'endroit où tu construis let mut moves.
```

Ajoute aussi l'import en haut de `src/eval.rs` :

```rust
use crate::legal_move::{generate_legal_move, generate_tactical_legal_move};
```

```rust
let mut moves = if in_check {
    generate_legal_move(board, tables)
} else {
    generate_tactical_legal_move(board, tables)
};
```

Même cette version est déjà meilleure que générer tous les coups légaux puis filtrer, parce que tu évites une partie du travail inutile.

Version encore meilleure plus tard : créer directement des fonctions spécialisées :

Où les mettre plus tard :

```text
src/pseudo_legal_move.rs, près des générateurs par pièce existants.
```

Puis expose une fonction publique dans `src/legal_move.rs` qui les appelle et vérifie la légalité des coups, comme `generate_tactical_legal_move`.

```rust
generate_pawn_captures(...)
generate_knight_captures(...)
generate_bishop_captures(...)
generate_rook_captures(...)
generate_queen_captures(...)
generate_king_captures(...)
```

Mais ne commence pas par là. Fais d'abord une version simple.

---

# 6. Ajouter un filtre de captures perdantes dans la quiescence

La quiescence ne doit pas forcément analyser toutes les captures.

Exemple :

```text
Dame prend pion défendu
```

Souvent, ce n'est pas prioritaire. Si tu analyses toutes les mauvaises captures, ton arbre explose.

La vraie solution s'appelle SEE :

```text
Static Exchange Evaluation
```

Mais tu peux commencer avec une approximation simple.

Ajoute :

Où le mettre :

```text
src/eval.rs, dans la zone des petites fonctions helper, par exemple juste après valeur_piece_abs et avant score_ordre_coup.
```

```rust
fn capture_probablement_mauvaise(mv: &Move) -> bool {
    if !matches!(mv.flag, MoveFlag::Capture | MoveFlag::EnPassant | MoveFlag::PromotionCapture) {
        return false;
    }

    let captured_value = match mv.captured {
        Some(piece) => valeur_piece_abs(piece),
        None => 100,
    };

    let attacker_value = valeur_piece_abs(mv.piece);

    captured_value + 100 < attacker_value
}
```

Puis dans la quiescence, seulement hors échec :

Où le mettre :

```text
src/eval.rs, dans quiescence, au tout début de la boucle for mv in moves, avant let old_board = board.clone().
```

```rust
if !in_check && capture_probablement_mauvaise(&mv) {
    continue;
}
```

Ce n'est pas aussi bon qu'un vrai SEE, mais ça évite certains cas absurdes.

Attention : ce filtre est volontairement prudent avec `+ 100`. Tu ne veux pas supprimer trop agressivement des captures tactiques correctes.

---

# 7. Ajouter le delta pruning dans la quiescence

Le delta pruning coupe certaines captures qui ne peuvent probablement pas améliorer alpha.

Idée :

```text
si même en gagnant la pièce capturée, je reste très en dessous d'alpha,
alors inutile d'analyser cette capture.
```

Exemple simple :

Où le mettre :

```text
src/eval.rs, près des autres helpers d'évaluation, juste après capture_probablement_mauvaise.
```

```rust
fn valeur_capture_mv(mv: &Move) -> i32 {
    match mv.captured {
        Some(piece) => valeur_piece_abs(piece),
        None => 100,
    }
}
```

Dans quiescence :

Où modifier :

```text
src/eval.rs, dans quiescence.
```

Pour que le code compile facilement, garde `stand_pat` visible dans toute la fonction : déclare-le avant le bloc `if !in_check`, puis utilise-le dans ce bloc et dans la boucle.

```rust
let stand_pat = evaluation_negamax(board);
let margin = 200;

// dans la boucle des coups tactiques, hors échec :
if !in_check {
    let max_gain = valeur_capture_mv(&mv) + margin;

    if stand_pat + max_gain < alpha {
        continue;
    }
}
```

À ne pas utiliser si :

```text
le roi est en échec
le coup est une promotion
le coup donne mat potentiellement immédiat
```

Version prudente :

Où le mettre :

```text
src/eval.rs, dans quiescence, dans la boucle for mv in moves, avant let old_board = board.clone().
```

Si tu as aussi ajouté `capture_probablement_mauvaise`, mets le delta pruning juste après ce filtre.

```rust
if !in_check
    && !matches!(mv.flag, MoveFlag::Promotion | MoveFlag::PromotionCapture)
{
    let max_gain = valeur_capture_mv(&mv) + 200;

    if stand_pat + max_gain < alpha {
        continue;
    }
}
```

Le delta pruning peut réduire fortement les `qnodes`.

---

# 8. Corriger deux détails dans ton alpha-beta actuel

Dans ton code, tu as :

Où chercher :

```text
src/eval.rs, dans evaluation_min_max et/ou evaluation_negamax_alpha_beta.
```

```rust
let mut meilleure = - 10000;
```

Remplace par :

```rust
let mut meilleure = -INF;
```

Parce que tu as déjà :

```rust
const INF: i32 = 1_000_000;
```

Ensuite, remplace :

Où chercher :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta et meilleur_coup, juste après generate_legal_move.
```

```rust
moves.sort_by_key(|mv| -score_ordre_coup(mv));
```

par :

Ajoute l'import en haut de `src/eval.rs` si besoin :

```rust
use std::cmp::Reverse;
```

```rust
moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));
```

C'est plus propre et cohérent avec ta quiescence.

---

# 9. Ajouter une table de transposition simple

Quand tu cherches à profondeur 5 ou 6, beaucoup de positions sont recalculées plusieurs fois par des ordres de coups différents.

Exemple :

```text
Cavalier joue puis fou joue
Fou joue puis cavalier joue
```

Ces deux chemins peuvent parfois arriver à la même position.

Une table de transposition sert à mémoriser :

```text
cette position a déjà été calculée à telle profondeur avec tel score
```

## 9.1 Version simple avec ta clé de position actuelle

Si tu as déjà une `ClePosition` pour la répétition, tu peux l'utiliser temporairement.

Structure :

Où le mettre :

```text
src/position_key.rs, après la définition de ClePosition et de la fonction cle_position.
```

Ajoute aussi cet import en haut de `src/position_key.rs`, parce que `TTEntry` stocke un `Move` :

```rust
use crate::chess_move::Move;
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub depth: u32,
    pub score: i32,
    pub flag: TTFlag,
    pub best_move: Option<Move>,
}
```

Table :

Où le mettre :

```text
src/position_key.rs, dans le même fichier, juste après TTEntry ou juste avant les structs TT.
```

Si `HashMap` n'est pas déjà importé, mets l'import en haut de `src/position_key.rs`.

```rust
use std::collections::HashMap;

pub type TranspositionTable = HashMap<ClePosition, TTEntry>;
```

Dans `meilleur_coup`, crée :

Où le mettre :

```text
src/eval.rs, dans meilleur_coup, au début de la fonction, après SearchStats.
```

```rust
let mut tt = TranspositionTable::new();
```

Puis passe `&mut tt` à alpha-beta. Cela veut dire que la signature de `evaluation_negamax_alpha_beta` dans `src/eval.rs` doit aussi recevoir :

```rust
tt: &mut TranspositionTable
```

et que chaque appel récursif à `evaluation_negamax_alpha_beta` doit retransmettre `tt`.

## 9.2 Utilisation dans alpha-beta

Au début de `evaluation_negamax_alpha_beta` :

Où le mettre :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, juste après stats.nodes += 1.
```

Imports nécessaires en haut de `src/eval.rs` :

```rust
use crate::position_key::{cle_position, TranspositionTable, TTEntry, TTFlag};
```

```rust
let alpha_original = alpha;
let key = cle_position(board);

if let Some(entry) = tt.get(&key) {
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

        if alpha >= beta {
            return entry.score;
        }
    }
}
```

Après avoir calculé le meilleur score :

Où le mettre :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, après la boucle for mv in moves et juste avant de retourner le meilleur score.
```

Pendant la boucle, garde aussi une variable `best_move` / `meilleur_mv` pour savoir quel coup stocker.

```rust
let flag = if meilleur_score <= alpha_original {
    TTFlag::UpperBound
} else if meilleur_score >= beta {
    TTFlag::LowerBound
} else {
    TTFlag::Exact
};

tt.insert(
    key,
    TTEntry {
        depth,
        score: meilleur_score,
        flag,
        best_move: meilleur_mv,
    },
);
```

Cette version est imparfaite, mais elle te permet déjà de comprendre le principe.

Plus tard, remplace `ClePosition` par une clé Zobrist `u64`, beaucoup plus rapide.

---

# 10. Utiliser le meilleur coup de la table pour trier les coups

Une table de transposition n'est pas seulement utile pour retourner un score déjà connu.

Elle sert aussi à dire :

```text
la dernière fois, le meilleur coup dans cette position était celui-ci
```

Donc tu dois le tester en premier.

Exemple :

Où le mettre :

```text
src/eval.rs, juste après score_ordre_coup.
```

```rust
fn score_ordre_coup_avec_tt(mv: &Move, tt_best: Option<Move>) -> i32 {
    if Some(*mv) == tt_best {
        return 1_000_000;
    }

    score_ordre_coup(mv)
}
```

Puis :

Où le mettre :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, après let mut moves = generate_legal_move(...) et avant moves.sort_by_key(...).
```

La variable `key` doit être celle calculée au début de `evaluation_negamax_alpha_beta` avec `cle_position(board)`.

```rust
let tt_best = tt.get(&key).and_then(|entry| entry.best_move);

moves.sort_by_key(|mv| Reverse(score_ordre_coup_avec_tt(mv, tt_best)));
```

Impact : alpha-beta coupe beaucoup plus tôt si les bons coups sont analysés d'abord.

---

# 11. Ajouter iterative deepening

L'iterative deepening consiste à chercher :

```text
profondeur 1
profondeur 2
profondeur 3
profondeur 4
...
```

au lieu de lancer directement :

```text
profondeur 6
```

À première vue, ça semble plus lent, mais en pratique c'est souvent meilleur parce que :

```text
la profondeur 1 trouve un bon coup candidat
la profondeur 2 le confirme
la profondeur 3 donne un meilleur ordre de coups
la table de transposition aide la profondeur suivante
```

Structure simple :

Où le mettre :

```text
src/eval.rs, juste après meilleur_coup, ou juste avant meilleur_coup si tu préfères que meilleur_coup appelle cette version.
```

```rust
pub fn meilleur_coup_iterative(
    board: &mut CBoard,
    tables: &AttackTables,
    max_depth: u32,
) -> Option<Move> {
    let mut best_move = None;
    let mut tt = TranspositionTable::new();

    for depth in 1..=max_depth {
        let mv = meilleur_coup_avec_tt(board, tables, depth, &mut tt);

        if mv.is_some() {
            best_move = mv;
        }

        println!("depth {} -> {:?}", depth, best_move);
    }

    best_move
}
```

Le snippet utilise `meilleur_coup_avec_tt`. Tu peux le créer dans `src/eval.rs` juste à côté de `meilleur_coup`, ou adapter ton `meilleur_coup` actuel pour recevoir `tt: &mut TranspositionTable`.

Plus tard, ajoute une limite de temps.

Pour l'utiliser dans l'interface web :

```text
src/web_server.rs, dans jouer_coup_ia, remplace l'appel à meilleur_coup par meilleur_coup_iterative.
```

Pense aussi à adapter l'import en haut de `src/web_server.rs` :

```rust
use crate::eval::meilleur_coup_iterative;
```

---

# 12. Ajouter une limite de temps propre

Pour une interface web, il vaut mieux dire :

```text
l'IA a 1 seconde pour jouer
```

plutôt que :

```text
l'IA doit absolument atteindre profondeur 6
```

Version simple :

Où le mettre :

```text
src/eval.rs, près de SearchStats, au début du fichier.
```

Remplace l'import `Instant` par celui-ci si tu utilises déjà le chronométrage :

```rust
use std::time::{Duration, Instant};
```

Puis ajoute la structure :

```rust
pub struct SearchLimits {
    pub start: Instant,
    pub max_time: Duration,
}

impl SearchLimits {
    pub fn should_stop(&self) -> bool {
        self.start.elapsed() >= self.max_time
    }
}
```

Dans la recherche :

Où le mettre :

```text
src/eval.rs, au début de evaluation_negamax_alpha_beta, juste après stats.nodes += 1.
```

La signature de `evaluation_negamax_alpha_beta` doit alors recevoir `limits: &SearchLimits`, et les appels récursifs doivent retransmettre `limits`.

```rust
if limits.should_stop() {
    return evaluation_negamax(board);
}
```

Et avec iterative deepening :

Où le mettre :

```text
src/eval.rs, dans meilleur_coup_iterative, dans la boucle for depth in 1..=max_depth.
```

```rust
for depth in 1..=max_depth {
    if limits.should_stop() {
        break;
    }

    let mv = meilleur_coup_avec_tt(...);

    if !limits.should_stop() && mv.is_some() {
        best_move = mv;
    }
}
```

Important : retourne toujours le meilleur coup de la dernière profondeur terminée proprement.

---

# 13. Réduire le coût de `board.clone()`

Dans ton code, à chaque coup tu fais :

```rust
let old_board = board.clone();
make_move(board, mv);
let score = -search(...);
*board = old_board;
```

C'est simple et correct pour apprendre.

Mais à grande profondeur, ce coût devient réel.

Optimisation future : transformer `make_move` en :

Où le faire :

```text
src/make_move.rs, en ajoutant une nouvelle fonction qui retourne UndoMove, puis une fonction unmake_move.
```

Ne remplace pas brutalement tous les `board.clone()` au début : commence par créer l'API dans `src/make_move.rs`, puis adapte seulement la recherche quand les tests passent.

```rust
let undo = make_move(board, mv);
// search
unmake_move(board, mv, undo);
```

Avec une structure :

Où le mettre :

```text
src/make_move.rs, près de make_move, avant ou après la fonction.
```

```rust
pub struct UndoMove {
    pub captured: Option<Pieces>,
    pub castling_rights: u8,
    pub en_passant_square: Option<u8>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub white_king_square: u8,
    pub black_king_square: u8,
}
```

Ne fais pas ça en premier.

Quand l'API `unmake_move` sera prête, les remplacements se feront principalement ici :

```text
src/eval.rs, dans les boucles de evaluation_min_max, evaluation_negamax_alpha_beta, meilleur_coup et quiescence.
src/legal_move.rs, dans generate_legal_move et generate_tactical_legal_move.
src/perft.rs, dans perft si tu veux aussi optimiser les tests de performance.
```

Ordre recommandé :

```text
1. quiescence moins coûteuse
2. compteurs
3. table de transposition
4. move ordering
5. seulement ensuite make/unmake
```

Pourquoi ?

Parce qu'un mauvais `unmake_move` crée des bugs très difficiles à détecter.

---

# 14. Ajouter killer moves

Les killer moves sont des coups calmes qui ont provoqué une coupure beta à une profondeur donnée.

Idée :

```text
si un coup calme a réfuté une ligne ailleurs,
il peut être intéressant de le tester tôt dans une autre branche de même profondeur.
```

Structure simple :

Où le mettre :

```text
src/eval.rs, près de SearchStats, au début du fichier.
```

```rust
pub struct KillerMoves {
    pub killers: [[Option<Move>; 2]; 128],
}
```

Quand tu as une coupure beta sur un coup non-capture :

Où le mettre :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, dans la boucle des coups, dans le bloc if score >= beta ou if alpha >= beta.
```

Pour cela, la signature de `evaluation_negamax_alpha_beta` doit recevoir `ply: u32` et `killers: &mut KillerMoves`.

```rust
if score >= beta {
    if !is_tactical_move(&mv) {
        killers.killers[ply as usize][1] = killers.killers[ply as usize][0];
        killers.killers[ply as usize][0] = Some(mv);
    }

    return beta;
}
```

Dans le tri des coups :

Où le mettre :

```text
src/eval.rs, dans une fonction de score de tri, par exemple score_ordre_coup_avec_tt_ou_killer.
```

Cette fonction doit recevoir `ply: u32` et `killers: &KillerMoves`, puis être utilisée dans le `moves.sort_by_key(...)` de `evaluation_negamax_alpha_beta`.

```rust
if Some(*mv) == killers.killers[ply as usize][0] {
    return 900_000;
}

if Some(*mv) == killers.killers[ply as usize][1] {
    return 800_000;
}
```

Priorité de tri recommandée :

```text
1. coup de la table de transposition
2. promotions
3. bonnes captures MVV-LVA
4. killer moves
5. roque
6. autres coups calmes
```

---

# 15. Ajouter history heuristic

La history heuristic donne un score aux coups calmes qui ont souvent provoqué des coupures.

Version simple :

Où le mettre :

```text
src/eval.rs, près de SearchStats et KillerMoves.
```

```rust
pub struct HistoryHeuristic {
    pub table: [[i32; 64]; 64],
}
```

Quand un coup calme coupe :

Où le mettre :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, dans le même bloc de coupure beta que les killer moves.
```

La signature doit recevoir `history: &mut HistoryHeuristic`, et les appels récursifs doivent le retransmettre.

```rust
history.table[mv.from as usize][mv.to as usize] += (depth * depth) as i32;
```

Dans le tri :

Où le mettre :

```text
src/eval.rs, dans la fonction de score de tri des coups, après le score TT/killer et avant le score calme par défaut.
```

```rust
let history_score = history.table[mv.from as usize][mv.to as usize];
```

Ce n'est pas prioritaire avant la table de transposition, mais c'est utile.

---

# 16. Ordre d'implémentation recommandé

Ne fais pas tout en même temps. Voici l'ordre exact que je te conseille.

## Étape 1 — Benchmark propre

À faire immédiatement :

```text
cargo run --release
ajouter SearchStats dans src/eval.rs
mesurer nodes/qnodes/cutoffs/qcutoffs dans src/eval.rs, dans meilleur_coup
```

Objectif : savoir si le problème vient surtout de la quiescence.

## Étape 2 — Quiescence moins profonde

Remplacer temporairement :

```rust
quiescence(..., 4)
```

par :

```rust
quiescence(..., 2)
```

Où :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, dans le bloc depth == 0.
```

Objectif : retrouver un temps acceptable.

## Étape 3 — Quiescence correcte en cas d'échec

Modifier la quiescence pour ne pas utiliser `stand_pat` si le roi est en échec.

Où :

```text
src/eval.rs, dans la fonction quiescence.
```

Objectif : éviter une erreur logique importante.

## Étape 4 — Générateur tactique

Créer :

```rust
generate_tactical_legal_move
```

Où :

```text
src/legal_move.rs, après is_tactical_move et avant generate_legal_move.
```

Puis l'appeler depuis :

```text
src/eval.rs, dans quiescence, quand le roi n'est pas en échec.
```

Objectif : ne plus générer tous les coups légaux à chaque nœud de quiescence.

C'est probablement le plus gros gain immédiat.

## Étape 5 — Delta pruning prudent

Ajouter un delta pruning simple dans la quiescence.

Où :

```text
src/eval.rs, helper près des fonctions d'évaluation, filtre dans la boucle de quiescence.
```

Objectif : réduire les branches tactiques inutiles.

## Étape 6 — Filtre de captures mauvaises

Ajouter un filtre simple avant un vrai SEE.

Où :

```text
src/eval.rs, helper près des fonctions d'évaluation, filtre dans la boucle de quiescence.
```

Objectif : éviter de chercher trop de captures absurdes.

## Étape 7 — Table de transposition

Commencer avec `ClePosition`, puis passer à Zobrist plus tard.

Où :

```text
src/position_key.rs pour TTFlag, TTEntry et TranspositionTable.
src/eval.rs pour consulter et remplir la table dans evaluation_negamax_alpha_beta.
```

Objectif : éviter de recalculer les mêmes positions.

## Étape 8 — Iterative deepening

Chercher 1, puis 2, puis 3, etc.

Où :

```text
src/eval.rs, nouvelle fonction meilleur_coup_iterative.
src/web_server.rs seulement si tu veux que l'IA web utilise cette nouvelle fonction.
```

Objectif : améliorer l'ordre des coups et préparer la gestion du temps.

## Étape 9 — TT best move ordering

Tester en premier le meilleur coup connu par la table.

Où :

```text
src/eval.rs, dans la fonction de score de tri et dans evaluation_negamax_alpha_beta.
```

Objectif : rendre alpha-beta beaucoup plus efficace.

## Étape 10 — Killer moves + history heuristic

Optimiser l'ordre des coups calmes.

Où :

```text
src/eval.rs, structures près de SearchStats, mise à jour dans les coupures beta.
```

Objectif : réduire encore le nombre de nœuds.

## Étape 11 — Make/unmake

Remplacer les clones par `make_move` + `unmake_move`.

Où :

```text
src/make_move.rs pour UndoMove et unmake_move.
src/eval.rs et src/legal_move.rs pour remplacer progressivement les clones.
```

Objectif : réduire le coût par nœud.

À faire seulement quand les tests sont solides.

---

# 17. Version cible simplifiée de ta recherche

À terme, la structure devrait ressembler à ça :

Où le mettre :

```text
src/eval.rs, cette version sert de modèle pour remplacer evaluation_negamax_alpha_beta.
```

Elle suppose que `src/eval.rs` importe la table avec :

```rust
use crate::position_key::{cle_position, TranspositionTable, TTEntry, TTFlag};
```

```rust
pub fn negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    ply: u32,
    mut alpha: i32,
    beta: i32,
    tt: &mut TranspositionTable,
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;

    if depth == 0 {
        return quiescence(board, tables, alpha, beta, 2, stats);
    }

    let alpha_original = alpha;
    let key = cle_position(board);

    let tt_best = if let Some(entry) = tt.get(&key) {
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

            if alpha >= beta {
                return entry.score;
            }
        }

        entry.best_move
    } else {
        None
    };

    let mut moves = generate_legal_move(board, tables);

    if moves.is_empty() {
        if is_king_in_check(board, tables, board.side_to_move) {
            return -SCORE_MAT + ply as i32;
        }
        return 0;
    }

    moves.sort_by_key(|mv| Reverse(score_ordre_coup_avec_tt(mv, tt_best)));

    let mut best_move = None;
    let mut best_score = -INF;

    for mv in moves {
        let old_board = board.clone();
        make_move(board, mv);

        let score = -negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            tt,
            stats,
        );

        *board = old_board;

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        if score > alpha {
            alpha = score;
        }

        if alpha >= beta {
            stats.cutoffs += 1;
            break;
        }
    }

    let flag = if best_score <= alpha_original {
        TTFlag::UpperBound
    } else if best_score >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    tt.insert(
        key,
        TTEntry {
            depth,
            score: best_score,
            flag,
            best_move,
        },
    );

    best_score
}
```

Ce code est un modèle d'architecture, pas forcément un copier-coller direct. Il dépend de tes modules exacts, notamment `cle_position(board)` et `generate_pseudo_legal_move`.

---

# 18. Ce que tu dois éviter maintenant

Évite pour l'instant :

```text
null move pruning
late move reductions
aspiration windows
futility pruning agressif
SEE complet trop tôt
réécriture complète du moteur
```

Ces optimisations sont réelles, mais elles peuvent casser ton moteur si les bases ne sont pas stabilisées.

Pour ton niveau actuel, les gains les plus propres sont :

```text
1. release mode
2. stats nodes/qnodes
3. quiescence moins profonde
4. génération tactique dans quiescence
5. table de transposition
6. meilleur ordre des coups
```

---

# 19. Résumé opérationnel

Ton problème vient très probablement de cette combinaison :

```text
alpha-beta profondeur 5
+ quiescence qdepth 4
+ generate_legal_move complet dans chaque qnode
+ board.clone() à chaque coup
+ pas de table de transposition
```

La correction la plus importante n'est pas d'augmenter brutalement la profondeur.

La correction importante est de réduire le coût de chaque nœud et de réduire le nombre de nœuds visités.

Ordre minimal à appliquer :

```text
1. cargo run --release
2. ajouter SearchStats dans src/eval.rs
3. passer qdepth de 4 à 2 dans src/eval.rs, dans evaluation_negamax_alpha_beta
4. corriger quiescence dans src/eval.rs
5. créer generate_tactical_legal_move dans src/legal_move.rs
6. ajouter delta pruning léger dans src/eval.rs, dans quiescence
7. ajouter table de transposition dans src/position_key.rs puis l'utiliser dans src/eval.rs
8. ajouter iterative deepening dans src/eval.rs
```

Avec seulement les étapes 1 à 5, tu devrais déjà sentir une nette différence.

Avec les étapes 6 à 8, tu commences à avoir une recherche beaucoup plus sérieuse.
