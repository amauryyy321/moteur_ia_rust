# 13. Reduire le cout de `board.clone()` avec `make_move` / `unmake_move`

## Objectif

Dans le code actuel, beaucoup de boucles font ceci :

```rust
let old_board = board.clone();
make_move(board, mv);
let score = -search(...);
*board = old_board;
```

C'est simple, lisible et correct pour apprendre.

Mais dans un moteur d'echecs, ce code est execute des milliers ou millions de fois. A grande profondeur, copier tout le `CBoard` a chaque coup finit par couter cher.

L'optimisation consiste a passer progressivement vers :

```rust
let undo = make_move(board, mv);
let score = -search(...);
unmake_move(board, mv, undo);
```

L'idee est simple :

```text
make_move   -> joue le coup et sauvegarde seulement ce qu'il faudra restaurer
unmake_move -> annule le coup avec ces informations
```

Important : ne commence pas par remplacer tous les `board.clone()` du projet. C'est une optimisation puissante, mais fragile. Un mauvais `unmake_move` peut creer des bugs tres difficiles a trouver.

L'ordre recommande reste :

```text
1. quiescence moins couteuse
2. compteurs de recherche
3. table de transposition
4. move ordering
5. seulement ensuite make/unmake
```

## Fichiers concernes

Le coeur de l'API doit aller ici :

```text
src/make_move.rs
```

Les remplacements de `board.clone()` se feront ensuite surtout ici :

```text
src/eval.rs
src/legal_move.rs
src/perft.rs
```

Tu n'as pas besoin de modifier tout de suite :

```text
src/partie.rs
tests/move_tests.rs
```

Ces fichiers jouent de vrais coups sur une vraie partie ou testent seulement `make_move`. Ils peuvent continuer a appeler `make_move(&mut board, mv);` meme si `make_move` retourne un `UndoMove`, car Rust autorise a ignorer une valeur de retour.

## Etape 1 : ajouter `UndoMove`

Où le faire :

```text
src/make_move.rs, pres des imports, juste avant make_move.
```

Ajoute cette structure :

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

Pourquoi ces champs ?

```text
captured            -> piece capturee a remettre sur l'echiquier
castling_rights     -> droits de roque avant le coup
en_passant_square   -> case en passant avant le coup
halfmove_clock      -> compteur des 50 coups avant le coup
fullmove_number     -> numero de coup complet avant le coup
white_king_square   -> position du roi blanc avant le coup
black_king_square   -> position du roi noir avant le coup
```

Tu ne stockes pas tout le `CBoard`, sinon tu reviens au meme probleme que `board.clone()`.

Tu ne stockes pas forcement `side_to_move`, parce que `make_move` inverse toujours le trait, donc `unmake_move` peut simplement l'inverser une deuxieme fois. Si tu veux une version plus defensive au debut, tu peux ajouter `side_to_move: Color`, mais ce n'est pas obligatoire.

## Etape 2 : transformer `make_move` pour retourner `UndoMove`

Où le faire :

```text
src/make_move.rs, dans la signature de make_move.
```

Remplace :

```rust
pub fn make_move(board: &mut CBoard, mv: Move) {
```

par :

```rust
pub fn make_move(board: &mut CBoard, mv: Move) -> UndoMove {
```

Puis, tout au debut de la fonction, avant de modifier le board, cree le `undo` :

```rust
let undo = UndoMove {
    captured: mv.captured,
    castling_rights: board.castling_rights,
    en_passant_square: board.en_passant_square,
    halfmove_clock: board.halfmove_clock,
    fullmove_number: board.fullmove_number,
    white_king_square: board.white_king_square,
    black_king_square: board.black_king_square,
};
```

Exemple du debut de fonction :

```rust
pub fn make_move(board: &mut CBoard, mv: Move) -> UndoMove {
    let undo = UndoMove {
        captured: mv.captured,
        castling_rights: board.castling_rights,
        en_passant_square: board.en_passant_square,
        halfmove_clock: board.halfmove_clock,
        fullmove_number: board.fullmove_number,
        white_king_square: board.white_king_square,
        black_king_square: board.black_king_square,
    };

    let from_bb = 1u64 << mv.from;
    let to_bb = 1u64 << mv.to;
    let couleur_avant_coup = board.side_to_move;

    // suite de ton code actuel...
```

Ensuite, tout a la fin de `make_move`, apres :

```rust
board.update_occupancies();
```

ajoute :

```rust
undo
```

Donc la fin devient :

```rust
board.update_occupancies();
undo
```

Point important : ne mets pas `#[must_use]` au debut. Sinon le compilateur peut commencer a te signaler les anciens appels qui ignorent volontairement le retour pendant la migration.

## Etape 3 : ajouter `unmake_move`

Où le faire :

```text
src/make_move.rs, juste apres make_move.
```

Ajoute cette fonction :

```rust
pub fn unmake_move(board: &mut CBoard, mv: Move, undo: UndoMove) {
    let from_bb = 1u64 << mv.from;
    let to_bb = 1u64 << mv.to;

    board.side_to_move = match board.side_to_move {
        Color::Blanc => Color::Noir,
        Color::Noir => Color::Blanc,
    };

    board.castling_rights = undo.castling_rights;
    board.en_passant_square = undo.en_passant_square;
    board.halfmove_clock = undo.halfmove_clock;
    board.fullmove_number = undo.fullmove_number;
    board.white_king_square = undo.white_king_square;
    board.black_king_square = undo.black_king_square;

    if let Some(promotion) = mv.promotion {
        board.piece_bb[promotion as usize] &= !to_bb;
        board.piece_bb[mv.piece as usize] |= from_bb;
    } else {
        board.piece_bb[mv.piece as usize] &= !to_bb;
        board.piece_bb[mv.piece as usize] |= from_bb;
    }

    if mv.flag == MoveFlag::Castling {
        match (mv.piece, mv.from, mv.to) {
            (Pieces::RoiBlanc, 4, 6) => {
                board.piece_bb[Pieces::TourBlanche as usize] &= !(1u64 << 5);
                board.piece_bb[Pieces::TourBlanche as usize] |= 1u64 << 7;
            }
            (Pieces::RoiBlanc, 4, 2) => {
                board.piece_bb[Pieces::TourBlanche as usize] &= !(1u64 << 3);
                board.piece_bb[Pieces::TourBlanche as usize] |= 1u64 << 0;
            }
            (Pieces::RoiNoir, 60, 62) => {
                board.piece_bb[Pieces::TourNoire as usize] &= !(1u64 << 61);
                board.piece_bb[Pieces::TourNoire as usize] |= 1u64 << 63;
            }
            (Pieces::RoiNoir, 60, 58) => {
                board.piece_bb[Pieces::TourNoire as usize] &= !(1u64 << 59);
                board.piece_bb[Pieces::TourNoire as usize] |= 1u64 << 56;
            }
            _ => {}
        }
    }

    if mv.flag == MoveFlag::EnPassant {
        match mv.piece {
            Pieces::PionBlanc => {
                let captured_square = mv.to - 8;
                board.piece_bb[Pieces::PionNoir as usize] |= 1u64 << captured_square;
            }
            Pieces::PionNoir => {
                let captured_square = mv.to + 8;
                board.piece_bb[Pieces::PionBlanc as usize] |= 1u64 << captured_square;
            }
            _ => {}
        }
    } else if let Some(captured_piece) = undo.captured {
        board.piece_bb[captured_piece as usize] |= to_bb;
    }

    board.update_occupancies();
}
```

Ce que cette fonction annule :

```text
coup normal       -> remet la piece de mv.to vers mv.from
capture           -> remet la piece capturee sur mv.to
promotion         -> retire la piece promue et remet le pion sur mv.from
promotion capture -> retire la piece promue, remet le pion, remet la capture sur mv.to
prise en passant  -> remet le pion capture sur la case derriere mv.to
roque             -> remet aussi la tour a sa case de depart
compteurs         -> restaure halfmove_clock et fullmove_number
etat special      -> restaure roque, en passant et positions des rois
trait             -> inverse side_to_move pour revenir au joueur precedent
```

## Etape 4 : ajouter des tests avant de remplacer les clones

Ne remplace pas encore les boucles de recherche.

Ajoute d'abord des tests dedies.

Où le faire :

```text
tests/undo_move_tests.rs
```

Exemple :

```rust
use moteur_ia::attack_tables::init_attack_tables;
use moteur_ia::board::CBoard;
use moteur_ia::fen::board_from_fen;
use moteur_ia::legal_move::generate_legal_move;
use moteur_ia::make_move::{make_move, unmake_move};

fn assert_make_unmake_restores_all_legal_moves(mut board: CBoard) {
    let tables = init_attack_tables();
    let moves = generate_legal_move(&mut board, &tables);

    for mv in moves {
        let before = board;
        let undo = make_move(&mut board, mv);
        unmake_move(&mut board, mv, undo);

        assert_eq!(board, before, "make/unmake ne restaure pas {:?}", mv);
    }
}

#[test]
fn make_unmake_restores_start_position_moves() {
    assert_make_unmake_restores_all_legal_moves(CBoard::init_position_depart());
}

#[test]
fn make_unmake_restores_special_positions() {
    let positions = [
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "7k/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/p7/4K3 b - - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ];

    for fen in positions {
        let board = board_from_fen(fen).unwrap();
        assert_make_unmake_restores_all_legal_moves(board);
    }
}
```

Lance ensuite :

```bash
cargo test
```

Puis :

```bash
cargo test --release perft
```

Si ces tests ne passent pas, ne touche pas encore a `src/eval.rs`.

## Etape 5 : remplacer dans `src/eval.rs`

Commence par la recherche, pas par tout le projet.

Où le faire :

```text
src/eval.rs, au debut du fichier.
```

Remplace :

```rust
use crate::make_move::make_move;
```

par :

```rust
use crate::make_move::{make_move, unmake_move};
```

### 5.1 Dans `quiescence`

Où le faire :

```text
src/eval.rs, dans la boucle for mv in moves de quiescence.
```

Remplace :

```rust
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
```

par :

```rust
for mv in moves {
    let undo = make_move(board, mv);
    let score = -quiescence(board, tables, -beta, -alpha, qdepth - 1, stats);
    unmake_move(board, mv, undo);

    if score >= beta {
        stats.qcutoffs += 1;
        return beta;
    }

    if score > alpha {
        alpha = score;
    }
}
```

Pourquoi commencer ici ?

La quiescence peut etre appelee enormement de fois. C'est donc un bon premier endroit pour enlever les clones, une fois que `unmake_move` est teste.

### 5.2 Dans `evaluation_negamax_alpha_beta`

Où le faire :

```text
src/eval.rs, dans la boucle for coups in moves de evaluation_negamax_alpha_beta.
```

Remplace :

```rust
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
```

par :

```rust
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
    );
    unmake_move(board, coups, undo);

    if score > meilleure {
        meilleure = meilleure.max(score);
        meilleur_mv = Some(coups);
    }

    alpha = alpha.max(score);

    if alpha >= beta {
        stats.cutoffs += 1;
        break;
    }
}
```

Regle importante : si tu ajoutes plus tard un `return` dans cette boucle, tu dois toujours appeler `unmake_move` avant le `return`.

### 5.3 Dans `meilleur_coup`

Où le faire :

```text
src/eval.rs, dans la boucle for mv in coups de meilleur_coup.
```

Remplace :

```rust
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
```

par :

```rust
for mv in coups {
    let undo = make_move(board, mv);
    let score = -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 1,
        -beta,
        -alpha,
        &mut stats,
        tt,
    );
    unmake_move(board, mv, undo);

    if score > meilleur_score {
        meilleur_mv = Some(mv);
        meilleur_score = score;
    }

    alpha = alpha.max(score);
}
```

### 5.4 Dans `evaluation_min_max`

Cette fonction semble surtout etre une ancienne version de recherche. Tu peux la migrer aussi pour rester coherent.

Où le faire :

```text
src/eval.rs, dans la boucle for coups in moves de evaluation_min_max.
```

Remplace :

```rust
for coups in moves{
    let old_board = board.clone();
    make_move(board,coups );
    let score = -evaluation_min_max(board,tables,depth - 1);
    *board = old_board;
    meilleure = meilleure.max(score);
}
```

par :

```rust
for coups in moves {
    let undo = make_move(board, coups);
    let score = -evaluation_min_max(board, tables, depth - 1);
    unmake_move(board, coups, undo);

    meilleure = meilleure.max(score);
}
```

Apres ces remplacements :

```bash
cargo test
```

Puis :

```bash
cargo test --release perft
```

## Etape 6 : remplacer dans `src/legal_move.rs`

Cette etape est plus sensible, parce que toute la generation de coups legaux depend de ce fichier.

Où le faire :

```text
src/legal_move.rs, au debut du fichier.
```

Remplace :

```rust
use crate::make_move::make_move;
```

par :

```rust
use crate::make_move::{make_move, unmake_move};
```

### 6.1 Dans `generate_tactical_legal_move`

Remplace :

```rust
for mv in moves {
    let old_board = board.clone();
    make_move(board,mv);
    if !is_king_in_check(board,tables,side){
        legal.push(mv);
    }
    *board = old_board;
}
```

par :

```rust
for mv in moves {
    let undo = make_move(board, mv);

    if !is_king_in_check(board, tables, side) {
        legal.push(mv);
    }

    unmake_move(board, mv, undo);
}
```

### 6.2 Dans `generate_legal_move`

Remplace :

```rust
for mv in gen_pseudo_move {
    let old_board = board.clone();
    make_move(board, mv);
    if !is_king_in_check(board, tables, color) {
        legal_move.push(mv);
    }
    *board = old_board;
}
```

par :

```rust
for mv in gen_pseudo_move {
    let undo = make_move(board, mv);

    if !is_king_in_check(board, tables, color) {
        legal_move.push(mv);
    }

    unmake_move(board, mv, undo);
}
```

Apres cette etape, lance absolument :

```bash
cargo test
```

Puis :

```bash
cargo test --release perft
```

Si le perft change, le bug est probablement dans `unmake_move`, pas dans perft.

## Etape 7 : remplacer dans `src/perft.rs`

Ce remplacement est utile pour accelerer les tests de performance.

Où le faire :

```text
src/perft.rs, au debut du fichier.
```

Remplace :

```rust
use crate::make_move::make_move;
```

par :

```rust
use crate::make_move::{make_move, unmake_move};
```

Puis, dans la boucle :

```rust
for mv in moves {
    let old_board = board.clone();
    make_move(board, mv);
    nodes += perft(board, tables, depth - 1);
    *board = old_board;
}
```

remplace par :

```rust
for mv in moves {
    let undo = make_move(board, mv);
    nodes += perft(board, tables, depth - 1);
    unmake_move(board, mv, undo);
}
```

Lance :

```bash
cargo test --release perft
```

Les resultats attendus doivent rester identiques :

```text
position initiale :
depth 1 -> 20
depth 2 -> 400
depth 3 -> 8902
depth 4 -> 197281
depth 5 -> 4865609

kiwipete :
depth 1 -> 48
depth 2 -> 2039
depth 3 -> 97862
depth 4 -> 4085603
```

## Checklist de validation

Avant de considerer cette optimisation terminee :

```text
[ ] make_move retourne UndoMove
[ ] unmake_move compile
[ ] tests make/unmake ajoutes
[ ] cargo test passe
[ ] cargo test --release perft passe
[ ] src/eval.rs migre
[ ] cargo test repasse
[ ] src/legal_move.rs migre
[ ] cargo test --release perft repasse
[ ] src/perft.rs migre
[ ] aucun board.clone inutile dans les boucles critiques
```

Pour verifier les clones restants :

```bash
rg "board\\.clone\\(|old_board" src tests
```

Attention : tous les clones ne sont pas forcement mauvais. Un clone dans un test ou pour comparer deux positions peut etre parfaitement acceptable.

## Les bugs classiques a surveiller

### 1. Oublier la piece capturee

Si tu oublies :

```rust
board.piece_bb[captured_piece as usize] |= to_bb;
```

alors une capture supprime definitivement une piece apres `unmake_move`.

Symptome :

```text
perft faux des la profondeur 2 ou 3
```

### 2. Mal gerer la prise en passant

En passant est special, parce que la piece capturee n'est pas sur `mv.to`.

Pour un pion blanc :

```text
mv.to - 8
```

Pour un pion noir :

```text
mv.to + 8
```

Symptome :

```text
les perft simples passent, mais certaines positions avec en passant cassent
```

### 3. Oublier la tour pendant le roque

Le roi revient de `g1` vers `e1`, mais la tour doit aussi revenir de `f1` vers `h1`.

Symptome :

```text
positions avec roque fausses, kiwipete faux
```

### 4. Oublier les compteurs

Meme si les pieces sont correctes, il faut restaurer :

```text
castling_rights
en_passant_square
halfmove_clock
fullmove_number
white_king_square
black_king_square
```

Sinon la position semble correcte visuellement, mais la recherche, la regle des 50 coups, le roque ou la table de transposition peuvent devenir faux.

### 5. Appeler `return` avant `unmake_move`

Exemple dangereux :

```rust
let undo = make_move(board, mv);
let score = -search(...);

if score >= beta {
    return beta;
}

unmake_move(board, mv, undo);
```

Ce code est faux, parce que le board reste modifie.

Il faut toujours faire :

```rust
let undo = make_move(board, mv);
let score = -search(...);
unmake_move(board, mv, undo);

if score >= beta {
    return beta;
}
```

## Strategie conseillee

Fais cette optimisation en petites etapes :

```text
1. Ajouter UndoMove et unmake_move.
2. Ajouter les tests make/unmake.
3. Lancer cargo test.
4. Remplacer seulement quiescence.
5. Tester.
6. Remplacer evaluation_negamax_alpha_beta et meilleur_coup.
7. Tester.
8. Remplacer generate_legal_move et generate_tactical_legal_move.
9. Tester avec perft release.
10. Remplacer perft.
```

Ce n'est pas l'optimisation a faire en premier, mais quand elle est faite proprement, elle enleve un cout important dans les noeuds de recherche.
