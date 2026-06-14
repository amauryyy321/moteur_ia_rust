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

```rust
use std::time::Instant;

let mut stats = SearchStats::default();
let start = Instant::now();

let mv = meilleur_coup(&mut board, &tables, depth, &mut stats);

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

```rust
return quiescence(board, tables, alpha, beta, 4);
```

Pour stabiliser, commence avec :

```rust
return quiescence(board, tables, alpha, beta, 2);
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
generate_tactical_legal_moves(...)
```

Cette fonction doit générer seulement :

```text
captures
promotions
en passant
```

Première version simple si tu as accès à `generate_pseudo_legal_move` :

```rust
pub fn generate_tactical_legal_moves(
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

```rust
let mut moves = if in_check {
    generate_legal_move(board, tables)
} else {
    generate_tactical_legal_moves(board, tables)
};
```

Même cette version est déjà meilleure que générer tous les coups légaux puis filtrer, parce que tu évites une partie du travail inutile.

Version encore meilleure plus tard : créer directement des fonctions spécialisées :

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

```rust
fn valeur_capture_mv(mv: &Move) -> i32 {
    match mv.captured {
        Some(piece) => valeur_piece_abs(piece),
        None => 100,
    }
}
```

Dans quiescence :

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

```rust
moves.sort_by_key(|mv| -score_ordre_coup(mv));
```

par :

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

```rust
use std::collections::HashMap;

pub type TranspositionTable = HashMap<ClePosition, TTEntry>;
```

Dans `meilleur_coup`, crée :

```rust
let mut tt = TranspositionTable::new();
```

Puis passe `&mut tt` à alpha-beta.

## 9.2 Utilisation dans alpha-beta

Au début de `negamax_alpha_beta` :

```rust
let alpha_original = alpha;
let key = ClePosition::from_board(board);

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

```rust
fn score_ordre_coup_avec_tt(mv: &Move, tt_best: Option<Move>) -> i32 {
    if Some(*mv) == tt_best {
        return 1_000_000;
    }

    score_ordre_coup(mv)
}
```

Puis :

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

Plus tard, ajoute une limite de temps.

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

```rust
use std::time::{Duration, Instant};

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

```rust
if limits.should_stop() {
    return evaluation_negamax(board);
}
```

Et avec iterative deepening :

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

```rust
let undo = make_move(board, mv);
// search
unmake_move(board, mv, undo);
```

Avec une structure :

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

```rust
pub struct KillerMoves {
    pub killers: [[Option<Move>; 2]; 128],
}
```

Quand tu as une coupure beta sur un coup non-capture :

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

```rust
pub struct HistoryHeuristic {
    pub table: [[i32; 64]; 64],
}
```

Quand un coup calme coupe :

```rust
history.table[mv.from as usize][mv.to as usize] += (depth * depth) as i32;
```

Dans le tri :

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
ajouter SearchStats
mesurer nodes/qnodes/cutoffs/qcutoffs
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

Objectif : retrouver un temps acceptable.

## Étape 3 — Quiescence correcte en cas d'échec

Modifier la quiescence pour ne pas utiliser `stand_pat` si le roi est en échec.

Objectif : éviter une erreur logique importante.

## Étape 4 — Générateur tactique

Créer :

```rust
generate_tactical_legal_moves
```

Objectif : ne plus générer tous les coups légaux à chaque nœud de quiescence.

C'est probablement le plus gros gain immédiat.

## Étape 5 — Delta pruning prudent

Ajouter un delta pruning simple dans la quiescence.

Objectif : réduire les branches tactiques inutiles.

## Étape 6 — Filtre de captures mauvaises

Ajouter un filtre simple avant un vrai SEE.

Objectif : éviter de chercher trop de captures absurdes.

## Étape 7 — Table de transposition

Commencer avec `ClePosition`, puis passer à Zobrist plus tard.

Objectif : éviter de recalculer les mêmes positions.

## Étape 8 — Iterative deepening

Chercher 1, puis 2, puis 3, etc.

Objectif : améliorer l'ordre des coups et préparer la gestion du temps.

## Étape 9 — TT best move ordering

Tester en premier le meilleur coup connu par la table.

Objectif : rendre alpha-beta beaucoup plus efficace.

## Étape 10 — Killer moves + history heuristic

Optimiser l'ordre des coups calmes.

Objectif : réduire encore le nombre de nœuds.

## Étape 11 — Make/unmake

Remplacer les clones par `make_move` + `unmake_move`.

Objectif : réduire le coût par nœud.

À faire seulement quand les tests sont solides.

---

# 17. Version cible simplifiée de ta recherche

À terme, la structure devrait ressembler à ça :

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
    let key = ClePosition::from_board(board);

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

Ce code est un modèle d'architecture, pas forcément un copier-coller direct. Il dépend de tes modules exacts, notamment `ClePosition::from_board` et `generate_pseudo_legal_move`.

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
2. ajouter SearchStats
3. passer qdepth de 4 à 2
4. corriger quiescence en cas d'échec
5. créer generate_tactical_legal_moves
6. ajouter delta pruning léger
7. ajouter table de transposition
8. ajouter iterative deepening
```

Avec seulement les étapes 1 à 5, tu devrais déjà sentir une nette différence.

Avec les étapes 6 à 8, tu commences à avoir une recherche beaucoup plus sérieuse.

