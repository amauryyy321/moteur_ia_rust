# Evolution propre de l'evaluation et de la recherche IA

## Objectif du document

Ce document explique comment faire evoluer ton IA d'echecs Rust progressivement.

Tu as deja une base importante:

* generation des coups legaux;
* `make_move`;
* detection d'echec;
* evaluation materielle;
* negamax;
* alpha-beta;
* choix du meilleur coup;
* debut d'ordre des coups;
* debut d'evaluation positionnelle.

L'objectif maintenant n'est pas de refaire ton moteur.

L'objectif est de passer progressivement de:

```rust
let score = evaluation_materielle(board);
```

vers:

```rust
let score = evaluation_blanc(board);
```

Puis vers une IA capable de mieux eviter les erreurs tactiques simples.

---

# 1. Etat actuel de ton code

Ton evaluation principale est maintenant organisee comme ceci:

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

C'est une bonne structure.

Elle permet de separer les idees:

```text
evaluation_materielle       -> combien valent les pieces
evaluation_cavaliers        -> est-ce que les cavaliers sont bien places
evaluation_paire_de_fous    -> bonus si on a deux fous
evaluation_roque            -> bonus si le roi est roque
```

Ensuite, pour utiliser cette evaluation dans negamax, tu fais:

```rust
pub fn evaluation_negamax(board: &CBoard) -> i32 {
    let score = evaluation_blanc(board);

    match board.side_to_move {
        Color::Blanc => score,
        Color::Noir => -score,
    }
}
```

C'est correct.

Pourquoi?

Parce que `evaluation_blanc` donne toujours un score du point de vue des Blancs.

Exemples:

```text
+300  -> les Blancs sont mieux
-300  -> les Noirs sont mieux
```

Mais dans negamax, on veut toujours evaluer du point de vue du joueur qui doit jouer.

Donc:

```text
si c'est aux Blancs de jouer : on garde le score
si c'est aux Noirs de jouer  : on inverse le score
```

---

# 2. Corrections importantes a faire dans ton code actuel

Avant de continuer vers la quiescence search ou Zobrist, il faut corriger quelques points.

---

## 2.1 Corriger `score_ordre_coup`

Dans ton code actuel, tu as ceci:

```rust
MoveFlag::Capture | MoveFlag::EnPassant => (10 * (mv.piece) -valeur_piece(mv.captured)),
```

Ce code ne peut pas marcher correctement.

Problemes:

1. `mv.piece` est une valeur de type `Pieces`, pas un nombre.
2. `mv.captured` est un `Option<Pieces>`, pas directement une piece.
3. Ta fonction `valeur_piece` retourne des valeurs positives pour les Blancs et negatives pour les Noirs.
4. Pour MVV-LVA, on ne veut pas une valeur signee. On veut la valeur absolue de la piece.

Pour l'ordre des captures, il faut une fonction qui donne toujours une valeur positive.

Exemple:

```rust
fn valeur_piece_abs(piece: Pieces) -> i32 {
    match piece {
        Pieces::PionBlanc | Pieces::PionNoir => 100,
        Pieces::CavalierBlanc | Pieces::CavalierNoir => 320,
        Pieces::FouBlanc | Pieces::FouNoir => 330,
        Pieces::TourBlanche | Pieces::TourNoire => 500,
        Pieces::DameBlanche | Pieces::DameNoire => 900,
        Pieces::RoiBlanc | Pieces::RoiNoir => 20_000,
        _ => 0,
    }
}
```

Ensuite, tu peux corriger ton ordre de coups comme ceci:

```rust
pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::Promotion | MoveFlag::PromotionCapture => {
            let mut score = 8000;

            if let Some(promotion) = mv.promotion {
                score += valeur_piece_abs(promotion);
            }

            if let Some(piece_capturee) = mv.captured {
                score += 10 * valeur_piece_abs(piece_capturee) - valeur_piece_abs(mv.piece);
            }

            score
        }

        MoveFlag::Capture | MoveFlag::EnPassant => {
            let valeur_capturee = match mv.captured {
                Some(piece) => valeur_piece_abs(piece),
                None => 100,
            };

            let valeur_attaquante = valeur_piece_abs(mv.piece);

            1000 + 10 * valeur_capturee - valeur_attaquante
        }

        MoveFlag::Castling => 100,

        _ => 0,
    }
}
```

Pourquoi mettre `1000 +` devant les captures?

Parce que tu veux que les captures soient generalement regardees avant les coups calmes.

Ensuite MVV-LVA classe les captures entre elles.

Exemples:

```text
pion prend dame       -> tres prioritaire
cavalier prend tour   -> interessant
dame prend pion       -> moins prioritaire
tour prend pion       -> moins prioritaire
```

La formule:

```rust
10 * valeur_piece(piece_capturee) - valeur_piece(piece_attaquante)
```

sert uniquement a ordonner les captures.

Elle ne remplace pas l'evaluation materielle.

L'evaluation materielle dit:

```text
apres avoir joue le coup, est-ce que ma position est bonne?
```

MVV-LVA dit:

```text
dans quel ordre dois-je tester les coups pour rendre alpha-beta plus rapide?
```

Ce sont deux choses differentes.

---

## 2.2 Trier les coups proprement

Tu as actuellement:

```rust
moves.sort_by_key(|mv| -score_ordre_coup(mv));
```

Cela fonctionne souvent, mais une version plus propre est d'utiliser `Reverse`.

Ajoute en haut du fichier:

```rust
use std::cmp::Reverse;
```

Puis fais:

```rust
moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));
```

Même chose dans `meilleur_coup`.

---

## 2.3 Corriger le bug du mat

Dans ton alpha-beta, tu as:

```rust
return --SCORE_MAT + depth as i32;
```

Il faut corriger en:

```rust
return -SCORE_MAT + depth as i32;
```

Pourquoi `+ depth as i32`?

Parce que cela permet de preferer les mats rapides.

Exemple:

```text
mat en 1 -> score tres bon
mat en 3 -> score bon, mais moins urgent
```

En negamax, quand le joueur au trait est mat, c'est mauvais pour lui.

Donc on retourne:

```rust
-SCORE_MAT + depth as i32
```

---

## 2.4 Remplacer `-10000` par `-INF`

Dans ton code, tu as:

```rust
let mut meilleure = -10000;
```

Mais tu as deja defini:

```rust
const INF: i32 = 1_000_000;
```

Donc il vaut mieux faire:

```rust
let mut meilleure = -INF;
```

C'est plus coherent et plus robuste.

A corriger dans:

```rust
evaluation_min_max
evaluation_negamax_alpha_beta
```

---

## 2.5 Corriger l'evaluation des cavaliers noirs

Tu as actuellement:

```rust
while let Some(square) = pop_lsb(&mut cavaliers_noirs) {
    let mirrored = mirror_square(square);
    score -= BONUS_CAVALIER[square as usize];
}
```

Probleme:

Tu calcules `mirrored`, mais tu ne l'utilises pas.

Il faut faire:

```rust
while let Some(square) = pop_lsb(&mut cavaliers_noirs) {
    let mirrored = mirror_square(square);
    score -= BONUS_CAVALIER[mirrored as usize];
}
```

Pourquoi?

Ton tableau `BONUS_CAVALIER` est ecrit du point de vue des Blancs.

Un cavalier blanc avance vers le haut du plateau.

Un cavalier noir avance vers le bas du plateau.

Donc pour evaluer une piece noire avec une table blanche, il faut retourner verticalement la case.

---

# 3. Explication de `mirror_square`

Tu as cette fonction:

```rust
fn mirror_square(square: u8) -> u8 {
    square ^ 56
}
```

Elle sert a retourner une case verticalement.

Avec ton systeme:

```text
0  = a1
1  = b1
2  = c1
...
7  = h1

8  = a2
...
56 = a8
63 = h8
```

La fonction:

```rust
square ^ 56
```

fait une symetrie verticale.

Exemples:

```text
a1 -> a8
b1 -> b8
e2 -> e7
d4 -> d5
a8 -> a1
```

Pourquoi c'est utile?

Parce que ton tableau `BONUS_CAVALIER` est pense pour les Blancs.

Si un cavalier blanc est en d4, tu regardes directement:

```rust
BONUS_CAVALIER[d4]
```

Mais si un cavalier noir est en d5, c'est l'equivalent d'un cavalier blanc en d4.

Donc tu dois faire:

```rust
BONUS_CAVALIER[mirror_square(d5)]
```

Version corrigee:

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

---

# 4. Ce qu'il ne faut pas encore faire

Ne commence pas maintenant:

```text
table de transposition
Zobrist
UCI
gestion du temps avancee
evaluation ultra complexe des pions
evaluation avancee de la securite du roi
moteur d'ouverture
null move pruning
late move reductions
futility pruning
aspiration windows
```

Ces elements sont utiles, mais trop tot pour ton moteur actuel.

La priorite est:

```text
1. evaluation propre
2. recherche stable
3. tests simples
4. IA qui joue legalement
5. IA qui evite les grosses erreurs tactiques
```

Si tu vas trop vite vers Zobrist ou la table de transposition, tu vas rendre les bugs beaucoup plus difficiles a comprendre.

---

# 5. Etape 15 — Stabiliser l'evaluation actuelle

## 15.1 Ajouter `INF` et `SCORE_MAT`

Tu as deja:

```rust
const SCORE_MAT: i32 = 100_000;
const INF: i32 = 1_000_000;
```

C'est bien.

Utilisation conseillee:

```rust
let mut meilleur_score = -INF;
let beta = INF;
let mut alpha = -INF;
```

Et pour le mat:

```rust
return -SCORE_MAT + depth as i32;
```

---

## 15.2 Garder une seule recherche principale

Tu peux garder `evaluation_min_max` pour apprendre, mais la vraie fonction a utiliser doit etre:

```rust
evaluation_negamax_alpha_beta
```

Idealement, a terme:

```rust
pub fn negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    alpha: i32,
    beta: i32,
) -> i32
```

Le nom `evaluation_negamax_alpha_beta` marche, mais il est un peu long.

Plus tard, tu pourras renommer:

```rust
evaluation_negamax_alpha_beta
```

en:

```rust
negamax_alpha_beta
```

Parce que ce n'est pas juste une evaluation.

C'est une recherche.

---

## 15.3 Corriger `meilleur_coup`

Ta fonction est deja presque bonne:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    let mut meilleur_mv = None;
    let mut meilleur_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for mv in coups {
        let old_board = board.clone();

        make_move(board, mv);

        let score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
        );

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

Attention:

Si `depth == 0`, alors `depth - 1` pose probleme.

Tu peux securiser:

```rust
if depth == 0 {
    return None;
}
```

Au debut de la fonction.

Version plus sure:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
) -> Option<Move> {
    if depth == 0 {
        return None;
    }

    let mut coups = generate_legal_move(board, tables);
    coups.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    let mut meilleur_mv = None;
    let mut meilleur_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for mv in coups {
        let old_board = board.clone();

        make_move(board, mv);

        let score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
        );

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

---

# 6. Etape 16 — MVV-LVA pour l'ordre des captures

## 16.1 Probleme actuel

Avant MVV-LVA, toutes les captures peuvent etre considerees comme presque equivalentes.

Mais ce n'est pas vrai.

Exemples:

```text
pion prend dame       -> excellent a regarder en premier
cavalier prend dame   -> tres interessant
dame prend pion       -> souvent moins prioritaire
tour prend pion       -> pas forcement urgent
```

Alpha-beta devient beaucoup plus efficace si les bons coups sont testes en premier.

---

## 16.2 Principe de MVV-LVA

MVV-LVA signifie:

```text
Most Valuable Victim - Least Valuable Attacker
```

En francais:

```text
Victime la plus chere - Attaquant le moins cher
```

L'idee est simple:

```text
capturer une piece chere avec une piece faible est souvent interessant
capturer une piece faible avec une piece chere est moins prioritaire
```

Formule:

```rust
score = 10 * valeur_piece(piece_capturee) - valeur_piece(piece_attaquante)
```

Exemple:

```text
pion prend dame:
10 * 900 - 100 = 8900

dame prend pion:
10 * 100 - 900 = 100
```

Donc le moteur regardera avant:

```text
pion prend dame
```

et seulement apres:

```text
dame prend pion
```

---

## 16.3 Attention importante

MVV-LVA ne dit pas que le coup est bon.

MVV-LVA dit seulement:

```text
ce coup merite d'etre analyse tot
```

C'est la recherche alpha-beta qui dira ensuite si le coup est vraiment bon.

Exemple:

```text
pion prend dame
```

peut etre mauvais si la dame est empoisonnee et que tu te fais mater juste apres.

Donc MVV-LVA est un outil d'ordre des coups, pas une evaluation finale.

---

# 7. Etape 17 — Quiescence search simple

## 17.1 Probleme a resoudre: l'effet horizon

Ton moteur s'arrete quand:

```rust
depth == 0
```

Et il retourne:

```rust
evaluation_negamax(board)
```

Probleme:

Il peut s'arreter au milieu d'une suite de captures.

Exemple:

```text
1. ton IA prend une dame
2. profondeur terminee
3. evaluation: super, je gagne une dame
4. mais au coup suivant l'adversaire reprend ta dame
```

Le moteur croit avoir gagne du materiel, mais il s'est arrete trop tot.

C'est ce qu'on appelle l'effet horizon.

---

## 17.2 Idee de la quiescence search

Au lieu de s'arreter brutalement a profondeur 0, on continue uniquement les coups tactiques.

Premiere version conseillee:

```text
a depth == 0:
    ne pas retourner directement evaluation
    lancer quiescence_search
```

La quiescence search regarde seulement:

```text
captures
```

Pas encore:

```text
echecs
promotions complexes
coups calmes
menaces
```

---

## 17.3 Premiere version simple

Structure conseillee:

```rust
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    let stand_pat = evaluation_negamax(board);

    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut moves = generate_legal_move(board, tables);

    moves.retain(|mv| {
        mv.flag == MoveFlag::Capture
            || mv.flag == MoveFlag::EnPassant
            || mv.flag == MoveFlag::PromotionCapture
    });

    moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

    for mv in moves {
        let old_board = board.clone();

        make_move(board, mv);

        let score = -quiescence(board, tables, -beta, -alpha);

        *board = old_board;

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
```

Ensuite dans alpha-beta:

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta);
}
```

Au lieu de:

```rust
if depth == 0 {
    return evaluation_negamax(board);
}
```

---

## 17.4 Pourquoi `stand_pat`?

`stand_pat` signifie:

```text
score de la position si je ne fais aucune capture supplementaire
```

Exemple:

```rust
let stand_pat = evaluation_negamax(board);
```

Si la position est deja assez bonne pour depasser beta, tu peux couper:

```rust
if stand_pat >= beta {
    return beta;
}
```

Sinon tu testes les captures pour voir si tu peux ameliorer alpha.

---

## 17.5 Limite de securite possible

Au debut, ta quiescence peut parfois devenir trop longue s'il y a beaucoup de captures.

Tu peux ajouter une profondeur limite:

```rust
pub fn quiescence(
    board: &mut CBoard,
    tables: &AttackTables,
    mut alpha: i32,
    beta: i32,
    qdepth: u32,
) -> i32 {
    if qdepth == 0 {
        return evaluation_negamax(board);
    }

    // reste du code
}
```

Et appeler:

```rust
let score = -quiescence(board, tables, -beta, -alpha, qdepth - 1);
```

Version simple conseillee:

```text
qdepth = 4 ou 6
```

---

# 8. Etape 18 — Tests IA tactiques simples

Avant d'ajouter des optimisations, il faut ajouter des tests.

Le but n'est pas d'avoir 100 tests.

Le but est d'avoir quelques tests fiables.

---

## 18.1 Test: l'IA prend une dame gratuite

Position a construire:

```text
un roi blanc
un roi noir
une dame noire attaquable gratuitement
une piece blanche qui peut la capturer
```

Objectif:

```text
meilleur_coup doit choisir la capture de la dame
```

Pseudo-test:

```rust
#[test]
fn ia_prend_dame_gratuite() {
    let tables = init_attack_tables();

    let mut board = board_from_fen(
        "7k/8/8/8/3q4/4N3/8/K7 w - - 0 1"
    ).unwrap();

    let mv = meilleur_coup(&mut board, &tables, 2).unwrap();

    assert_eq!(mv.to, 27); // a adapter selon la case de la dame
}
```

Il faut verifier les index exacts selon ta position.

---

## 18.2 Test: l'IA trouve un mat en 1

Objectif:

```text
si un mat immediat existe, l'IA doit le jouer
```

Ce test verifie que:

```text
generation legale OK
detection echec OK
score de mat OK
negamax OK
```

---

## 18.3 Test: l'IA ne joue pas un coup illegal

Objectif:

```text
meilleur_coup doit toujours retourner un coup contenu dans generate_legal_move
```

Exemple:

```rust
#[test]
fn ia_joue_un_coup_legal() {
    let tables = init_attack_tables();

    let mut board = board_from_fen(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    ).unwrap();

    let coups_legaux = generate_legal_move(&mut board, &tables);
    let mv = meilleur_coup(&mut board, &tables, 3).unwrap();

    assert!(coups_legaux.contains(&mv));
}
```

---

## 18.4 Test: l'IA prefere promouvoir en dame

Objectif:

```text
si un pion peut promouvoir, l'IA doit preferer la dame
```

La promotion en dame doit generalement avoir le meilleur score.

Attention:

Ce test suppose que ta generation des promotions donne bien plusieurs choix:

```text
promotion dame
promotion tour
promotion fou
promotion cavalier
```

---

## 18.5 Test: l'IA evite une capture perdante

Ce test est plus difficile.

Objectif:

```text
a profondeur suffisante, l'IA ne doit pas prendre une piece si elle perd encore plus derriere
```

Exemple conceptuel:

```text
dame prend pion
mais ensuite une tour prend la dame
```

Ce test sera plus fiable apres la quiescence search.

---

# 9. Etape 19 — Interface terminal propre

Avant de faire l'interface web, il faut un terminal clair.

Objectif:

```text
voir ce que l'IA fait
comprendre pourquoi elle joue
debugger plus facilement
```

Afficher a chaque tour:

```text
trait
score actuel
profondeur IA
meilleur coup choisi
temps de calcul
etat de partie
nombre de coups legaux
```

Exemple d'affichage:

```text
Trait: Blanc
Score: +120
Profondeur IA: 4
Coups legaux: 32
Meilleur coup IA: e2e4
Temps de calcul: 243 ms
Etat: EnCours
```

---

## 9.1 Mesurer le temps de calcul

Utiliser:

```rust
use std::time::Instant;
```

Exemple:

```rust
let start = Instant::now();

let mv = meilleur_coup(&mut board, &tables, depth);

let duree = start.elapsed();

println!("Temps de calcul: {} ms", duree.as_millis());
```

---

## 9.2 Afficher le score

Tu peux afficher:

```rust
let score = evaluation_blanc(&board);
```

Puis:

```rust
println!("Score: {}", score);
```

Interpretation:

```text
+100  -> avantage blanc d'environ un pion
+300  -> avantage blanc d'environ une piece mineure
-500  -> avantage noir d'environ une tour
```

---

# 10. Etape 20 — Zobrist hashing

A ne faire qu'apres stabilisation.

Actuellement, tu peux identifier une position avec:

```rust
ClePosition {
    piece_bb,
    side_to_move,
    castling_rights,
    en_passant_square,
}
```

C'est correct pour la repetition.

Mais pour une table de transposition, ce sera trop lourd.

Objectif futur:

```text
remplacer progressivement ClePosition par une cle u64 rapide
```

Cette cle s'appelle une cle Zobrist.

Principe:

```text
chaque piece sur chaque case correspond a un nombre aleatoire u64
on XOR les nombres correspondant a la position
```

Exemple conceptuel:

```rust
hash ^= zobrist_piece[piece][square];
hash ^= zobrist_side_to_move;
hash ^= zobrist_castling[castling_rights];
hash ^= zobrist_en_passant[file];
```

Mais pas maintenant.

---

# 11. Etape 21 — Table de transposition

Apres Zobrist, tu pourras ajouter une table de transposition.

Objectif:

```text
eviter de recalculer plusieurs fois la meme position
```

Structure future:

```rust
pub struct EntreeTransposition {
    pub depth: u32,
    pub score: i32,
    pub flag: TypeNoeud,
    pub meilleur_coup: Option<Move>,
}
```

Avec:

```rust
pub enum TypeNoeud {
    Exact,
    LowerBound,
    UpperBound,
}
```

La table pourra etre:

```rust
HashMap<u64, EntreeTransposition>
```

Mais seulement quand:

```text
negamax fonctionne
alpha-beta fonctionne
quiescence fonctionne
tests tactiques OK
```

---

# 12. Etape 22 — Iterative deepening

L'iterative deepening consiste a chercher successivement:

```text
profondeur 1
profondeur 2
profondeur 3
profondeur 4
...
```

Au lieu de lancer directement:

```text
profondeur 5
```

Avantages:

```text
meilleur controle du temps
meilleur ordre de coups
meilleure utilisation de la table de transposition
possibilite d'arreter proprement la recherche
```

Exemple futur:

```rust
for current_depth in 1..=max_depth {
    let mv = meilleur_coup(board, tables, current_depth);
    best_move = mv;
}
```

---

# 13. Etape 23 — Gestion du temps

Apres iterative deepening, tu pourras ajouter une limite de temps.

Exemple:

```text
l'IA a 2 secondes pour jouer
elle retourne le meilleur coup de la derniere profondeur terminee
```

Utiliser:

```rust
std::time::Instant
```

Principe:

```rust
let start = Instant::now();

for depth in 1..=max_depth {
    if start.elapsed().as_millis() > temps_max_ms {
        break;
    }

    best_move = meilleur_coup(board, tables, depth);
}
```

Attention:

La vraie gestion du temps doit verifier le temps aussi dans la recherche, pas seulement entre deux profondeurs.

Mais pour une premiere version, verifier entre les profondeurs suffit.

---

# 14. Etape 24 — UCI minimal

Le protocole UCI permet d'utiliser ton moteur dans une interface externe comme:

```text
Arena
Cute Chess
BanksiaGUI
lichess-bot
```

Commandes minimales a gerer:

```text
uci
isready
position startpos moves e2e4 e7e5
go depth 5
bestmove g1f3
quit
```

A ne faire qu'une fois le moteur stable.

Pour l'instant, ce n'est pas prioritaire.

---

# 15. Ordre conseille maintenant

Ordre exact recommande:

```text
15.1 Verifier INF et SCORE_MAT
15.2 Corriger --SCORE_MAT en -SCORE_MAT
15.3 Remplacer les -10000 par -INF
15.4 Corriger valeur_piece_abs
15.5 Corriger score_ordre_coup avec MVV-LVA
15.6 Trier avec Reverse(score_ordre_coup)
15.7 Corriger evaluation_cavaliers pour utiliser mirrored
15.8 Ajouter tests d'evaluation simples
15.9 Ajouter test IA joue un coup legal
16. Ajouter MVV-LVA proprement
17. Ajouter quiescence search simple
18. Ajouter tests tactiques IA
19. Faire un terminal propre
20. Ajouter Zobrist
21. Ajouter table de transposition
22. Ajouter iterative deepening
23. Ajouter gestion du temps
24. Ajouter UCI minimal
```

---

# 16. Version corrigee minimale des fonctions importantes

## 16.1 `valeur_piece_abs`

```rust
fn valeur_piece_abs(piece: Pieces) -> i32 {
    match piece {
        Pieces::PionBlanc | Pieces::PionNoir => 100,
        Pieces::CavalierBlanc | Pieces::CavalierNoir => 320,
        Pieces::FouBlanc | Pieces::FouNoir => 330,
        Pieces::TourBlanche | Pieces::TourNoire => 500,
        Pieces::DameBlanche | Pieces::DameNoire => 900,
        Pieces::RoiBlanc | Pieces::RoiNoir => 20_000,
        _ => 0,
    }
}
```

---

## 16.2 `score_ordre_coup`

```rust
pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::Promotion | MoveFlag::PromotionCapture => {
            let mut score = 8000;

            if let Some(promotion) = mv.promotion {
                score += valeur_piece_abs(promotion);
            }

            if let Some(piece_capturee) = mv.captured {
                score += 10 * valeur_piece_abs(piece_capturee)
                    - valeur_piece_abs(mv.piece);
            }

            score
        }

        MoveFlag::Capture | MoveFlag::EnPassant => {
            let valeur_capturee = match mv.captured {
                Some(piece) => valeur_piece_abs(piece),
                None => 100,
            };

            let valeur_attaquante = valeur_piece_abs(mv.piece);

            1000 + 10 * valeur_capturee - valeur_attaquante
        }

        MoveFlag::Castling => 100,

        _ => 0,
    }
}
```

---

## 16.3 `evaluation_cavaliers`

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

---

## 16.4 `evaluation_negamax_alpha_beta`

```rust
pub fn evaluation_negamax_alpha_beta(
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
    moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));

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

        let score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
        );

        *board = old_board;

        meilleure = meilleure.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;
        }
    }

    meilleure
}
```

---

# 17. Tests d'evaluation a ajouter

## 17.1 Test cavalier centre meilleur que cavalier bord

Ton test est bon dans l'idee:

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

Ce test verifie que:

```text
un cavalier central vaut mieux qu'un cavalier au bord
```

---

## 17.2 Test paire de fous

```rust
#[test]
fn paire_de_fous_donne_bonus_aux_blancs() {
    let board_un_fou = board_from_fen(
        "7k/8/8/8/8/8/8/K2B4 w - - 0 1"
    ).unwrap();

    let board_deux_fous = board_from_fen(
        "7k/8/8/8/8/8/8/K2BB3 w - - 0 1"
    ).unwrap();

    assert!(evaluation_blanc(&board_deux_fous) > evaluation_blanc(&board_un_fou));
}
```

---

## 17.3 Test roque blanc

```rust
#[test]
fn roque_blanc_donne_bonus() {
    let board_non_roque = board_from_fen(
        "7k/8/8/8/8/8/8/R3K2R w KQ - 0 1"
    ).unwrap();

    let board_roque = board_from_fen(
        "7k/8/8/8/8/8/8/R4RK1 w - - 0 1"
    ).unwrap();

    assert!(evaluation_blanc(&board_roque) > evaluation_blanc(&board_non_roque));
}
```

Attention:

Ce test suppose que:

```text
roi blanc en g1 = square 6
```

Donc ton `evaluation_roque` donne bien:

```rust
if board.white_king_square == 6 || board.white_king_square == 2 {
    score += 40;
}
```

---

# 18. Resume important

Tu n'es pas en train de refaire ton IA.

Tu fais une evolution propre.

Tu as deja:

```text
materiel
negamax
alpha-beta
meilleur_coup
ordre de coups simple
evaluation positionnelle simple
```

Maintenant tu dois faire dans cet ordre:

```text
1. corriger les petits bugs actuels
2. rendre l'ordre de coups propre avec MVV-LVA
3. corriger l'evaluation des cavaliers noirs
4. ajouter quelques tests d'evaluation
5. ajouter la quiescence search
6. ajouter des tests tactiques
7. seulement apres, passer a Zobrist et table de transposition
```

La transition principale est celle-ci:

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

C'est le moment ou ton IA passe de:

```text
je compte les pieces
```

a:

```text
je commence a comprendre la position
```

Ne cherche pas encore a faire une evaluation parfaite.

Cherche une evaluation:

```text
simple
stable
testable
comprehensible
facile a ameliorer
```
