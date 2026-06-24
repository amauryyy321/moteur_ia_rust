# Tutoriel guide : optimiser la quiescence search

## Objectif du document

Ce document explique comment rendre ta quiescence search plus correcte et plus rapide, en restant en mono-thread.

Le but n'est pas de bricoler un petit patch qui marche une fois. Le but est de construire une base solide que tu ne devras pas jeter dans deux semaines.

Tu vas apprendre :

```text
1. pourquoi ta quiescence actuelle est rapide mais presque inactive;
2. comment l'activer sans exploser le temps de calcul;
3. comment gerer correctement les positions d'echec;
4. comment generer directement les coups tactiques;
5. comment trier les captures;
6. comment ajouter delta pruning;
7. comment preparer SEE, Static Exchange Evaluation;
8. quels tests lancer a chaque etape.
```

Important : ce document est un tutoriel. Il te montre quoi modifier, ou le mettre, et pourquoi. Il ne modifie pas le code tout seul.

---

# 1. Etat actuel de ta quiescence

Dans ton code actuel, la recherche appelle la quiescence ici :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, quand depth == 0.
```

Tu as actuellement une logique de ce type :

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta, 0, stats, limits);
}
```

Le probleme est le dernier argument :

```rust
0
```

Ensuite, dans `quiescence`, tu as :

```rust
if qdepth == 0 {
    return evaluation_negamax(board);
}
```

Donc ta quiescence retourne presque directement l'evaluation statique.

En pratique :

```text
alpha-beta arrive a depth 0
quiescence est appelee
qdepth vaut 0
quiescence retourne evaluation_negamax
```

Resultat :

```text
ton moteur va vite
mais il ne resout pas vraiment l'effet horizon
```

L'effet horizon, c'est quand le moteur s'arrete juste avant une capture importante.

Exemple :

```text
depth 0 :
les Blancs viennent de jouer un coup qui attaque la dame noire
si tu evalues tout de suite, tu ne vois pas encore la capture/reponse
le score est trompeur
```

La quiescence sert a continuer seulement les coups tactiques jusqu'a une position plus calme.

---

# 2. Ce qu'une bonne quiescence doit faire

Une quiescence correcte suit cette idee :

```text
si le roi est en echec :
    generer tous les coups legaux pour sortir de l'echec
    ne pas faire stand_pat

sinon :
    calculer stand_pat = evaluation actuelle
    si stand_pat >= beta : cutoff
    alpha = max(alpha, stand_pat)
    generer seulement les coups tactiques
    essayer captures, promotions, en passant
```

Pourquoi cette difference ?

Parce que si ton roi est en echec, la position n'est pas calme. Tu n'as pas le droit de dire :

```rust
return evaluation_negamax(board);
```

Tu dois sortir de l'echec.

Exemple :

```text
roi noir en echec
seul bon coup : Kg8
ce coup n'est pas une capture
si la quiescence ne regarde que les captures, elle peut croire que c'est mat
```

Donc la regle de base :

```text
quiescence hors echec -> coups tactiques seulement
quiescence en echec   -> tous les coups legaux
```

---

# 3. Ordre d'implementation recommande

Ne fais pas tout d'un coup. La quiescence peut vite rendre un moteur tres lent ou tres bugge.

Ordre conseille :

```text
1. Ajouter une constante QUIESCENCE_DEPTH.
2. Appeler quiescence avec qdepth = QUIESCENCE_DEPTH.
3. Corriger la structure de quiescence pour gerer les echecs.
4. Garder generate_tactical_legal_move au debut, meme si ce n'est pas optimal.
5. Tester.
6. Remplacer la generation tactique par une vraie generation pseudo-tactique directe.
7. Tester.
8. Ajouter delta pruning.
9. Tester.
10. Ajouter SEE plus tard.
```

Pourquoi cet ordre ?

Parce que si tu changes en meme temps :

```text
qdepth
generation tactique
delta pruning
SEE
ordre des coups
```

et que le moteur devient mauvais, tu ne sauras pas ou est le bug.

---

# 4. Etape 1 : ajouter des constantes propres

Où le faire :

```text
src/eval.rs, pres des constantes SCORE_MAT et INF.
```

Ajoute :

```rust
const QUIESCENCE_DEPTH: u32 = 2;
const DELTA_MARGIN: i32 = 200;
```

Pourquoi commencer avec `2` ?

Parce que ton moteur est deja profond. Passer directement a `4` peut multiplier brutalement le nombre de noeuds.

Progression conseillee :

```text
qdepth 0 -> quiescence inactive, rapide
qdepth 1 -> regarde une capture
qdepth 2 -> regarde capture + reponse tactique
qdepth 3 -> plus solide tactiquement
qdepth 4 -> seulement quand la generation tactique est optimisee
```

Mon conseil :

```text
commence avec QUIESCENCE_DEPTH = 2
puis teste 3
ne mets 4 que plus tard
```

---

# 5. Etape 2 : appeler vraiment la quiescence

Où le faire :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta.
```

Tu as actuellement l'idee :

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta, 0, stats, limits);
}
```

Remplace le `0` par la constante :

```rust
if depth == 0 {
    return quiescence(
        board,
        tables,
        alpha,
        beta,
        QUIESCENCE_DEPTH,
        stats,
        limits,
    );
}
```

Ce changement active enfin la quiescence.

Mais attention : si tu fais seulement ca, ton moteur peut ralentir beaucoup. C'est normal. Les prochaines etapes servent a controler ce cout.

---

# 6. Etape 3 : structure robuste de `quiescence`

Où le faire :

```text
src/eval.rs, remplacer le corps de la fonction quiescence.
```

Structure solide :

```rust
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
    qdepth: u32,
    stats: &mut SearchStats,
    limits: &SearchLimits,
) -> i32 {
    stats.qnodes += 1;

    if limits.should_stop() {
        return evaluation_negamax(board);
    }

    let in_check = is_king_in_check(board, tables, board.side_to_move);

    if !in_check {
        let stand_pat = evaluation_negamax(board);

        if stand_pat >= beta {
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        if qdepth == 0 {
            return alpha;
        }
    }

    let mut moves = if in_check {
        generate_legal_move(board, tables)
    } else {
        generate_tactical_legal_move(board, tables)
    };

    if moves.is_empty() {
        if in_check {
            return -SCORE_MAT;
        }

        return alpha;
    }

    moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    for mv in moves {
        let undo = make_move(board, mv);
        let score = -quiescence(
            board,
            tables,
            -beta,
            -alpha,
            qdepth.saturating_sub(1),
            stats,
            limits,
        );
        unmake_move(board, mv, undo);

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

## Explication detaillee

### `stand_pat`

```rust
let stand_pat = evaluation_negamax(board);
```

`stand_pat` signifie :

```text
si je ne joue aucun coup tactique, combien vaut la position maintenant ?
```

Si ce score est deja trop bon :

```rust
if stand_pat >= beta {
    return beta;
}
```

Alors on coupe.

Pourquoi ?

Parce que l'adversaire avait deja une meilleure alternative avant d'arriver ici.

### Pourquoi pas de stand_pat en echec ?

Si le roi est en echec, tu ne peux pas dire :

```text
je ne joue rien, la position vaut X
```

Aux echecs, tu dois repondre a l'echec.

Donc :

```rust
if !in_check {
    // stand_pat seulement ici
}
```

### Pourquoi `generate_legal_move` en echec ?

Parce qu'une sortie d'echec peut etre :

```text
une capture
un blocage
un deplacement du roi
```

Les deux derniers peuvent etre des coups calmes.

Donc en echec :

```rust
generate_legal_move(board, tables)
```

Hors echec :

```rust
generate_tactical_legal_move(board, tables)
```

### Pourquoi `qdepth.saturating_sub(1)` ?

Pour eviter un underflow.

Avec `u32`, si tu fais :

```rust
qdepth - 1
```

quand `qdepth == 0`, c'est dangereux.

`saturating_sub(1)` donne :

```text
3 -> 2
2 -> 1
1 -> 0
0 -> 0
```

---

# 7. Etape 4 : verifier avant d'optimiser plus

Apres avoir active la quiescence avec `QUIESCENCE_DEPTH = 2`, lance :

```bash
cargo test
```

Puis :

```bash
cargo test --release perft
```

Perft ne teste pas directement la quiescence, mais il verifie que la generation de coups et make/unmake restent corrects.

Ensuite teste en mono-thread :

```bash
RAYON_NUM_THREADS=1 cargo run --release
```

Note :

```text
depth atteint
temps de calcul
nodes
qnodes
qcutoffs
```

Si `qnodes` devient enorme par rapport a `nodes`, la quiescence mange tout le temps.

---

# 8. Le vrai probleme actuel : generation tactique trop chere

Aujourd'hui, ta fonction :

```text
src/legal_move.rs -> generate_tactical_legal_move
```

fait ceci :

```rust
let mut moves = generate_pseudo_legal_move(board, tables);
moves.retain(|mv| is_tactical_move(mv));
```

Donc elle genere d'abord :

```text
tous les coups calmes
toutes les captures
tous les roques
tous les coups de pions
toutes les promotions
```

Puis elle supprime les coups non tactiques.

C'est simple, mais couteux.

Dans la quiescence, tu veux appeler cette fonction des milliers ou millions de fois.

Donc il faut finir par avoir :

```text
generate_pseudo_tactical_move
```

qui genere directement :

```text
captures
en passant
promotions
promotion-captures
```

Sans generer :

```text
coups calmes de pieces
double pawn push
roque
coups calmes du roi
coups calmes de cavaliers/fous/tours/dames
```

---

# 9. Etape 5 : creer une vraie generation pseudo-tactique

Où le faire :

```text
src/pseudo_legal_move.rs
```

Tu as actuellement :

```rust
pub fn generate_pseudo_legal_move(board: &CBoard, tables: &AttackTables) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    generer_mouvement_pions(board, &mut moves);
    generer_mouvement_cavaliers(board, tables, &mut moves);
    generer_mouvement_fous(board, tables, &mut moves);
    generer_mouvement_tours(board, tables, &mut moves);
    generer_mouvement_rois(board, tables, &mut moves);
    generer_mouvement_dames(board, tables, &mut moves);

    moves
}
```

La version solide consiste a ajouter un mode de generation.

## 9.1 Ajouter un mode

Où le faire :

```text
src/pseudo_legal_move.rs, pres du haut du fichier.
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveGenMode {
    All,
    TacticalOnly,
}
```

Pourquoi un mode ?

Parce que tu evites de dupliquer toute ta generation de coups.

Tu veux a terme deux entrees publiques :

```rust
pub fn generate_pseudo_legal_move(board: &CBoard, tables: &AttackTables) -> Vec<Move> {
    generate_pseudo_move_with_mode(board, tables, MoveGenMode::All)
}

pub fn generate_pseudo_tactical_move(board: &CBoard, tables: &AttackTables) -> Vec<Move> {
    generate_pseudo_move_with_mode(board, tables, MoveGenMode::TacticalOnly)
}
```

Puis une fonction commune :

```rust
fn generate_pseudo_move_with_mode(
    board: &CBoard,
    tables: &AttackTables,
    mode: MoveGenMode,
) -> Vec<Move> {
    let mut moves = Vec::new();

    generer_mouvement_pions_mode(board, &mut moves, mode);
    generer_mouvement_cavaliers_mode(board, tables, &mut moves, mode);
    generer_mouvement_fous_mode(board, tables, &mut moves, mode);
    generer_mouvement_tours_mode(board, tables, &mut moves, mode);
    generer_mouvement_rois_mode(board, tables, &mut moves, mode);
    generer_mouvement_dames_mode(board, tables, &mut moves, mode);

    moves
}
```

Oui, ca demande de modifier les generateurs de pieces. Mais c'est plus propre que de copier/coller tout ton fichier.

---

# 10. Regles exactes du mode TacticalOnly

Le mode `TacticalOnly` ne veut pas dire uniquement `MoveFlag::Capture`.

Il doit inclure :

```text
Capture
EnPassant
Promotion
PromotionCapture
```

Pourquoi inclure `Promotion` sans capture ?

Parce qu'une promotion calme est presque toujours tactique.

Exemple :

```text
un pion va a dame
ce n'est pas une capture
mais c'est beaucoup trop important pour l'ignorer
```

## Pions

En `TacticalOnly`, les pions doivent generer :

```text
captures
en passant
promotions simples
promotion-captures
```

Ils ne doivent pas generer :

```text
avance simple non-promotion
double pawn push
```

Donc dans `generer_mouvement_pion_blanc`, quand tu geres l'avance simple :

```rust
if promotion_rank {
    // garder meme en TacticalOnly
} else if mode == MoveGenMode::All {
    // coup calme normal
}
```

Et pour le double pawn push :

```rust
if mode == MoveGenMode::All {
    // generer double pawn push
}
```

Les captures restent generees dans les deux modes.

## Cavaliers, fous, tours, dames, rois

En `TacticalOnly`, ces pieces doivent seulement garder les destinations qui touchent une piece adverse.

La logique generale :

```rust
let target_mask = match mode {
    MoveGenMode::All => !friendly_pieces,
    MoveGenMode::TacticalOnly => enemy_pieces,
};
```

Puis :

```rust
let mut mouvement_possible = attaques & target_mask;
```

En mode complet :

```text
destination vide ou ennemie
```

En mode tactique :

```text
destination ennemie seulement
```

---

# 11. Helpers utiles pour eviter les erreurs

Où le faire :

```text
src/pseudo_legal_move.rs, pres de piece_on_square.
```

Tu peux ajouter des helpers :

```rust
fn friendly_pieces(board: &CBoard) -> u64 {
    match board.side_to_move {
        Color::Blanc => board.piece_bb[Pieces::PiecesBlanches as usize],
        Color::Noir => board.piece_bb[Pieces::PiecesNoires as usize],
    }
}

fn enemy_pieces(board: &CBoard) -> u64 {
    match board.side_to_move {
        Color::Blanc => board.piece_bb[Pieces::PiecesNoires as usize],
        Color::Noir => board.piece_bb[Pieces::PiecesBlanches as usize],
    }
}

fn target_mask_for_mode(board: &CBoard, mode: MoveGenMode) -> u64 {
    match mode {
        MoveGenMode::All => !friendly_pieces(board),
        MoveGenMode::TacticalOnly => enemy_pieces(board),
    }
}
```

Pourquoi c'est utile ?

Parce que tu evites de reecrire partout :

```rust
match board.side_to_move { ... }
```

Et tu reduis les bugs couleur blanche/noire.

---

# 12. Etape 6 : brancher la generation tactique dans legal_move

Où le faire :

```text
src/legal_move.rs
```

Aujourd'hui :

```rust
use crate::pseudo_legal_move::generate_pseudo_legal_move;
```

Tu gardes cet import, mais tu ajoutes :

```rust
use crate::pseudo_legal_move::generate_pseudo_tactical_move;
```

Puis dans `generate_tactical_legal_move`, remplace :

```rust
let mut moves = generate_pseudo_legal_move(board, tables);
moves.retain(|mv| is_tactical_move(mv));
```

par :

```rust
let moves = generate_pseudo_tactical_move(board, tables);
```

Tu peux garder `is_tactical_move` quelques temps pour faire un assert de securite en debug :

```rust
debug_assert!(moves.iter().all(is_tactical_move));
```

Pourquoi c'est mieux ?

Avant :

```text
generer 35 coups
en garder 4
tester la legalite de 4 coups
```

Apres :

```text
generer directement 4 coups
tester la legalite de 4 coups
```

Tu gagnes surtout sur la generation, pas sur le test de legalite.

---

# 13. Etape 7 : ameliorer l'ordre des coups tactiques

Tu as deja une fonction :

```text
src/eval.rs -> score_ordre_coup
```

Elle fait deja une base MVV-LVA :

```text
Most Valuable Victim - Least Valuable Attacker
```

Exemple :

```text
prendre une dame avec un pion   -> excellent
prendre un pion avec une dame   -> souvent mauvais
```

Dans la quiescence, c'est tres important.

Où le faire :

```text
src/eval.rs, dans quiescence, avant la boucle.
```

Tu as :

```rust
moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));
```

C'est deja bien.

Une version plus explicite peut etre :

```rust
moves.sort_by_key(|mv| Reverse(score_ordre_coup_quiescence(mv)));
```

Puis :

```rust
fn score_ordre_coup_quiescence(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::PromotionCapture => 20_000 + score_ordre_coup(mv),
        MoveFlag::Promotion => {
            let promotion_value = mv.promotion.map(valeur_piece_abs).unwrap_or(0);
            15_000 + promotion_value
        }
        MoveFlag::Capture | MoveFlag::EnPassant => score_ordre_coup(mv),
        _ => 0,
    }
}
```

Pourquoi separer `score_ordre_coup_quiescence` ?

Parce que le tri des coups en recherche normale et le tri en quiescence n'ont pas exactement le meme but.

En recherche normale, tu veux :

```text
TT move
captures
promotions
killer moves
history
roque
coups calmes
```

En quiescence, tu veux surtout :

```text
promotions fortes
bonnes captures
en passant
mauvaises captures en dernier
```

---

# 14. Etape 8 : ajouter delta pruning

Delta pruning sert a eviter des captures qui ne peuvent pas remonter alpha.

Idee :

```text
stand_pat = score actuel
je capture une piece qui vaut au maximum 100
meme avec cette capture, stand_pat + 100 + marge <= alpha
alors cette capture ne peut pas aider
donc je la saute
```

Où le faire :

```text
src/eval.rs, dans quiescence, uniquement hors echec.
```

Ne fais jamais delta pruning quand le roi est en echec.

## 14.1 Fonction de gain tactique max

Où le faire :

```text
src/eval.rs, pres de score_ordre_coup.
```

```rust
fn gain_tactique_max(mv: &Move) -> i32 {
    let capture_gain = mv.captured.map(valeur_piece_abs).unwrap_or(0);

    let promotion_gain = match mv.promotion {
        Some(piece) => valeur_piece_abs(piece) - valeur_piece_abs(mv.piece),
        None => 0,
    };

    capture_gain + promotion_gain
}
```

Explication :

```text
capture_gain   -> valeur de la piece capturee
promotion_gain -> gain approximatif de promotion
```

Exemple :

```text
pion devient dame : 900 - 100 = 800
capture d'une tour : 500
promotion-capture : 800 + valeur capturee
```

## 14.2 Utilisation dans quiescence

Tu dois garder le `stand_pat` disponible dans la boucle.

Une structure possible :

```rust
let mut stand_pat_for_delta = None;

if !in_check {
    let stand_pat = evaluation_negamax(board);

    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    if qdepth == 0 {
        return alpha;
    }

    stand_pat_for_delta = Some(stand_pat);
}
```

Puis dans la boucle :

```rust
for mv in moves {
    if let Some(stand_pat) = stand_pat_for_delta {
        let max_gain = gain_tactique_max(&mv);

        if stand_pat + max_gain + DELTA_MARGIN <= alpha {
            continue;
        }
    }

    let undo = make_move(board, mv);
    let score = -quiescence(
        board,
        tables,
        -beta,
        -alpha,
        qdepth.saturating_sub(1),
        stats,
        limits,
    );
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

Pourquoi `DELTA_MARGIN = 200` ?

Parce que l'evaluation n'est pas seulement materielle.

Tu as aussi :

```text
bonus pions
bonus cavaliers
paire de fous
roque
```

Donc il faut laisser une marge pour ne pas couper trop agressivement.

Valeurs conseillees :

```text
100 -> agressif
200 -> raisonnable
300 -> prudent
```

Commence avec :

```rust
const DELTA_MARGIN: i32 = 200;
```

---

# 15. Etape 9 : preparer SEE, Static Exchange Evaluation

Delta pruning est une estimation grossiere.

SEE est plus precise.

SEE repond a cette question :

```text
si je capture sur cette case,
et que l'adversaire recapture,
et que je recapture,
etc.,
est-ce que cette suite de captures gagne ou perd du materiel ?
```

Exemple :

```text
ma dame prend un pion protege
l'adversaire reprend ma dame
materiellement c'est catastrophique
```

MVV-LVA peut mettre ce coup assez haut parce que c'est une capture.

SEE permet de dire :

```text
cette capture perd du materiel, je peux la repousser ou la couper
```

## Interface conseillee

Où le faire plus tard :

```text
src/see.rs
```

ou :

```text
src/eval.rs, au debut, si tu veux commencer simple.
```

Interface :

```rust
pub fn see_ge(board: &CBoard, mv: Move, threshold: i32) -> bool {
    // retourne true si la capture semble gagner au moins threshold
}
```

Utilisation en quiescence :

```rust
if mv.captured.is_some() && !see_ge(board, mv, -50) {
    continue;
}
```

Ici `-50` veut dire :

```text
j'accepte une petite perte apparente
mais je refuse les captures clairement mauvaises
```

## Algorithme SEE, idee generale

SEE fait :

```text
1. identifier la case cible mv.to
2. simuler la premiere capture
3. trouver la piece adverse la moins chere qui attaque cette case
4. simuler la recapture
5. alterner les camps
6. construire la liste des gains
7. remonter la liste en minimax
```

Pseudo-code conceptuel :

```text
gain[0] = valeur(piece capturee)

camp = adversaire
occupancy = position apres la premiere capture

tant qu'il existe un attaquant du camp sur la case :
    attacker = attaquant le moins cher
    gain[i] = valeur(attacker precedent) - gain[i - 1]
    retirer attacker de occupancy
    changer camp

remonter gain[] depuis la fin :
    gain[i - 1] = -max(-gain[i - 1], gain[i])
```

Ce n'est pas une optimisation a faire en premier, parce qu'elle demande une bonne fonction :

```text
attaquants_vers_case(board, square, occupancy)
```

Donc ordre conseille :

```text
1. qdepth actif
2. in_check correct
3. generation tactique directe
4. delta pruning
5. SEE
```

---

# 16. Attention : quiescence en echec et qdepth

Il y a deux philosophies :

## Option A : qdepth limite tout

Quand `qdepth == 0`, tu retournes l'evaluation.

C'est rapide, mais moins correct si la position est encore en echec.

## Option B : toujours resoudre les echecs

Si le roi est en echec, tu generes les evasions meme si `qdepth == 0`.

C'est plus correct, mais ca peut augmenter le cout.

Pour ton moteur, je conseille un compromis :

```text
hors echec :
    si qdepth == 0 -> return alpha apres stand_pat

en echec :
    generer les coups legaux
    utiliser qdepth.saturating_sub(1)
```

Donc un echec au bout de la ligne tactique est traite proprement.

---

# 17. Tests a ajouter

Tu ne dois pas seulement regarder si le moteur joue un coup.

Tu veux des tests qui verifient :

```text
1. la quiescence ne casse pas les mats;
2. la quiescence traite les echecs;
3. la generation tactique ne genere pas de coups calmes inutiles;
4. les promotions sont incluses;
5. les prises en passant sont incluses;
6. le nombre de qnodes reste raisonnable.
```

## 17.1 Test : position en echec

But :

```text
verifier que la quiescence ne croit pas a un mat juste parce que les coups tactiques sont vides
```

Exemple de test conceptuel :

```rust
#[test]
fn quiescence_en_echec_genere_tous_les_coups_legaux() {
    let tables = init_attack_tables();
    let mut board = board_from_fen("7k/8/8/8/8/8/5q2/4K3 w - - 0 1").unwrap();
    let limits = SearchLimits {
        start: Instant::now(),
        max_time: Duration::from_millis(1000),
    };
    let mut stats = SearchStats::default();

    let score = quiescence(&mut board, &tables, -INF, INF, 2, &mut stats, &limits);

    assert!(score > -SCORE_MAT);
}
```

Ce test dependra de ta position exacte, donc il faudra peut-etre choisir une FEN plus propre.

L'idee est importante :

```text
en echec, qsearch doit regarder les evasions, pas seulement les captures
```

## 17.2 Test : promotion calme incluse

Tu veux verifier que `generate_pseudo_tactical_move` inclut :

```text
pion va en dame sans capture
```

Exemple :

```rust
#[test]
fn tactical_moves_include_quiet_promotion() {
    let tables = init_attack_tables();
    let board = board_from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();

    let moves = generate_pseudo_tactical_move(&board, &tables);

    assert!(moves.iter().any(|mv| {
        mv.from == 48
            && mv.to == 56
            && mv.flag == MoveFlag::Promotion
            && mv.promotion == Some(Pieces::DameBlanche)
    }));
}
```

## 17.3 Test : pas de quiet move normal en tactical

But :

```text
generate_pseudo_tactical_move ne doit pas generer e2e3
```

Exemple :

```rust
#[test]
fn tactical_moves_do_not_include_normal_quiet_moves() {
    let tables = init_attack_tables();
    let board = CBoard::init_position_depart();

    let moves = generate_pseudo_tactical_move(&board, &tables);

    assert!(!moves.iter().any(|mv| mv.flag == MoveFlag::Quiet));
    assert!(!moves.iter().any(|mv| mv.flag == MoveFlag::DoublePawnPush));
    assert!(!moves.iter().any(|mv| mv.flag == MoveFlag::Castling));
}
```

## 17.4 Test : perft toujours OK

Apres chaque changement dans `pseudo_legal_move.rs` ou `legal_move.rs` :

```bash
cargo test --release perft
```

La generation complete ne doit pas changer.

---

# 18. Benchmarks simples

En mono-thread :

```bash
RAYON_NUM_THREADS=1 cargo run --release
```

Mais il faut surtout afficher :

```text
depth
temps
nodes
qnodes
cutoffs
qcutoffs
```

Si tu veux comparer plusieurs versions :

```text
version A : qdepth = 0
version B : qdepth = 1
version C : qdepth = 2
version D : qdepth = 3
```

Regarde :

```text
nodes
qnodes
temps
qualite des coups
```

Un signe que la quiescence est trop chere :

```text
qnodes est 10x, 20x, 50x plus grand que nodes
```

Un signe qu'elle commence a etre utile :

```text
le moteur evite des captures tactiques simples
il ne donne plus une dame en un coup
il comprend mieux les recaptures
```

---

# 19. Ce qu'il ne faut pas faire

## Ne pas mettre qdepth 5 directement

Mauvais reflexe :

```rust
const QUIESCENCE_DEPTH: u32 = 5;
```

Ca peut exploser.

Commence :

```rust
const QUIESCENCE_DEPTH: u32 = 2;
```

Puis teste.

## Ne pas faire delta pruning en echec

En echec, tu dois sortir de l'echec.

Ne fais pas :

```rust
if stand_pat + gain + margin <= alpha {
    continue;
}
```

quand `in_check == true`.

## Ne pas supprimer les promotions calmes

Une promotion sans capture est tactique.

Il faut garder :

```text
MoveFlag::Promotion
```

## Ne pas oublier en passant

La prise en passant est une capture tactique.

Il faut garder :

```text
MoveFlag::EnPassant
```

## Ne pas faire confiance a la vitesse seule

Une quiescence inactive est tres rapide.

Mais elle rate des tactiques.

Le but n'est pas seulement :

```text
depth 9 rapide
```

Le but est :

```text
depth 7 ou 8 qui ne rate pas les captures evidentes
```

---

# 20. Ordre final conseille pour ton moteur

Voici l'ordre que je te conseille vraiment :

```text
1. Revenir sur une comparaison mono-thread propre.
2. Garder AI_DEPTH raisonnable pour tester, par exemple 6 ou 7.
3. Ajouter QUIESCENCE_DEPTH = 2.
4. Appeler quiescence avec QUIESCENCE_DEPTH.
5. Corriger quiescence en echec.
6. Tester cargo test et perft.
7. Ajouter generate_pseudo_tactical_move.
8. Brancher generate_tactical_legal_move dessus.
9. Tester cargo test et perft.
10. Ajouter score_ordre_coup_quiescence.
11. Ajouter delta pruning prudent.
12. Tester avec qdepth 2 puis qdepth 3.
13. Seulement ensuite penser SEE.
```

Ce qui devrait arriver :

```text
qdepth 2 + generation tactique directe :
    cout raisonnable
    tactique deja meilleure

qdepth 3 + delta pruning :
    meilleur compromis

qdepth 4 :
    seulement si qnodes reste controle
```

---

# 21. Checklist d'implementation

```text
[ ] ajouter QUIESCENCE_DEPTH
[ ] appeler quiescence avec QUIESCENCE_DEPTH
[ ] corriger le cas in_check dans quiescence
[ ] ne pas faire stand_pat en echec
[ ] utiliser generate_legal_move en echec
[ ] utiliser generate_tactical_legal_move hors echec
[ ] verifier qdepth.saturating_sub(1)
[ ] ajouter generate_pseudo_tactical_move
[ ] inclure Capture
[ ] inclure EnPassant
[ ] inclure Promotion
[ ] inclure PromotionCapture
[ ] exclure Quiet normal
[ ] exclure DoublePawnPush
[ ] exclure Castling hors echec
[ ] brancher generate_tactical_legal_move sur generate_pseudo_tactical_move
[ ] ajouter tri specifique quiescence
[ ] ajouter delta pruning hors echec seulement
[ ] tester cargo test
[ ] tester cargo test --release perft
[ ] benchmarker en mono-thread
```

---

# 22. Resume mental

La quiescence efficace, ce n'est pas :

```text
chercher plus profond partout
```

C'est :

```text
quand la recherche normale s'arrete,
continuer seulement les coups qui changent brutalement l'evaluation
```

Donc :

```text
captures
promotions
en passant
evasions d'echec
```

Et pour que ce soit rapide :

```text
ne pas generer les coups calmes
trier les captures
couper avec stand_pat
couper avec delta pruning
plus tard filtrer avec SEE
```

La bonne quiescence peut te faire perdre un peu de profondeur brute, mais elle rend les profondeurs atteintes beaucoup plus fiables.

Un moteur depth 7 avec bonne quiescence peut jouer mieux qu'un moteur depth 9 qui s'arrete au milieu des captures.
