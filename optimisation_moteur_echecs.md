# Optimisation du moteur d'echecs : tutoriel guide

## Objectif

Ce document explique comment implementer proprement les grosses optimisations qui manquent encore a ton moteur.

Il est ecrit pour ton code actuel, en mono-thread. Le but n'est pas de copier une astuce de Stockfish au hasard. Le but est de comprendre :

```text
1. ce que chaque optimisation resout;
2. ou la mettre dans ton projet;
3. comment elle change la recherche;
4. quels bouts de code ajouter;
5. quels tests lancer avant de continuer.
```

Liste traitee dans ce document :

```text
Zobrist hash au lieu de ClePosition avec [u64; 14]
table de transposition fixe au lieu de HashMap classique
killer moves
history heuristic
MVV-LVA propre
principal variation move ordering
aspiration windows
null move pruning
late move reductions
quiescence efficace
```

Important : ce document est un tutoriel. Les blocs de code montrent quoi mettre et ou, mais ce fichier ne modifie pas le moteur.

---

# 1. Etat actuel de ton moteur

Ton moteur a deja une base serieuse :

```text
make/unmake move
generation de coups legaux
negamax alpha-beta
table de transposition simple
best_move dans TTEntry
move ordering de base
quiescence presente mais presque inactive
iterative deepening
version parallel root experimentale
```

Le point le plus important : les prochaines optimisations doivent aider alpha-beta a couper plus vite.

Alpha-beta est tres sensible a l'ordre des coups.

Avec un mauvais ordre :

```text
tu cherches presque tout
```

Avec un bon ordre :

```text
tu trouves vite un bon coup
alpha monte
les autres branches coupent plus vite
```

Donc avant de penser "plus de threads", il faut rendre le mono-thread plus intelligent.

---

# 2. Ordre recommande

Je te conseille cet ordre :

```text
1. Mesure propre : nodes, qnodes, cutoffs, temps.
2. MVV-LVA propre pour les captures.
3. Principal variation move ordering.
4. Killer moves.
5. History heuristic.
6. Zobrist hash.
7. Table de transposition fixe.
8. Aspiration windows.
9. Quiescence efficace.
10. Null move pruning.
11. Late move reductions.
```

Pourquoi pas directement `null move pruning` ou `LMR` ?

Parce que ce sont des optimisations plus risquees. Si ton move ordering et ta table de transposition ne sont pas solides, elles peuvent te faire rater des tactiques ou donner des scores instables.

---

# 3. Mesure propre avant optimisation

Avant chaque optimisation, garde une mesure simple.

Où le faire :

```text
src/eval.rs
```

Tu as deja :

```rust
#[derive(Default, Debug, Clone)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,
    pub cutoffs: u64,
    pub qcutoffs: u64,
}
```

Tu peux l'etendre progressivement :

```rust
#[derive(Default, Debug, Clone)]
pub struct SearchStats {
    pub nodes: u64,
    pub qnodes: u64,
    pub cutoffs: u64,
    pub qcutoffs: u64,
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
    pub beta_cutoffs_first_move: u64,
    pub null_cutoffs: u64,
    pub lmr_researches: u64,
}
```

Pourquoi ?

Parce qu'une optimisation doit se voir dans les chiffres.

Exemples :

```text
TT meilleure       -> tt_hits augmente, nodes baisse
move ordering      -> beta_cutoffs_first_move augmente
quiescence active  -> qnodes augmente, mais tactique meilleure
null move pruning  -> null_cutoffs augmente, nodes baisse
LMR                -> nodes baisse, re-searches pas trop haut
```

Commande de benchmark mono-thread :

```bash
RAYON_NUM_THREADS=1 cargo run --release
```

Garde toujours :

```text
meme position
meme profondeur
meme limite de temps
mode release
1 thread si tu testes le moteur mono-thread
```

---

# 4. MVV-LVA propre

## Objectif

MVV-LVA signifie :

```text
Most Valuable Victim - Least Valuable Attacker
```

En francais :

```text
prendre une grosse piece avec une petite piece est prioritaire
prendre une petite piece avec une grosse piece est moins prioritaire
```

Exemples :

```text
pion prend dame     -> tres bon en premier
cavalier prend tour -> bon
dame prend pion     -> a tester plus tard
```

## Où le faire

```text
src/eval.rs, pres de score_ordre_coup.
```

Tu as deja `valeur_piece_abs`. Garde cette idee.

Version propre :

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

fn score_capture_mvv_lva(mv: &Move) -> i32 {
    let victim = mv.captured.map(valeur_piece_abs).unwrap_or(100);
    let attacker = valeur_piece_abs(mv.piece);

    10 * victim - attacker
}
```
score_ordre
Puis :

```rust
pub fn score_ordre_coup(mv: &Move) -> i32 {
    match mv.flag {
        MoveFlag::PromotionCapture => {
            let promotion = mv.promotion.map(valeur_piece_abs).unwrap_or(0);
            20_000 + promotion + score_capture_mvv_lva(mv)
        }

        MoveFlag::Promotion => {
            let promotion = mv.promotion.map(valeur_piece_abs).unwrap_or(0);
            15_000 + promotion
        }

        MoveFlag::Capture | MoveFlag::EnPassant => {
            10_000 + score_capture_mvv_lva(mv)
        }

        MoveFlag::Castling => 100,

        _ => 0,
    }
}
```

## Pourquoi ces grandes constantes ?

On veut forcer l'ordre :

```text
promotion-capture
promotion
capture
roque
quiet move
```

Sans grandes constantes, une capture pourrait parfois passer devant une promotion importante.

## Tests rapides

Où le faire :

```text
src/eval.rs, tests de score_ordre_coup
```

Idee :

```rust
assert!(score_ordre_coup(&pion_prend_dame) > score_ordre_coup(&dame_prend_pion));
assert!(score_ordre_coup(&promotion_dame) > score_ordre_coup(&capture_pion));
```

---

# 5. Principal variation move ordering

## Objectif

L'iterative deepening sert a trouver un bon coup a faible profondeur, puis a le tester en premier a la profondeur suivante.

Exemple :

```text
depth 4 trouve que e2e4 est meilleur
depth 5 teste e2e4 en premier
si e2e4 est vraiment bon, alpha monte vite
les autres coups coupent plus vite
```

Tu as deja une base avec :

```rust
TTEntry {
    best_move: Option<Move>,
}
```

et :

```rust
let tt_best = tt.get(&key).and_then(|entry| entry.best_move);
moves.sort_by_key(|mv| Reverse(score_ordre_coup_avec_tt(mv, tt_best)));
```

C'est deja une forme de PV move ordering.

## Amelioration a faire

Où le faire :

```text
src/eval.rs, dans meilleur_coup_iterative et meilleur_coup.
```

Le root doit aussi garder explicitement le meilleur coup de la profondeur precedente.

Idee :

```rust
pub fn meilleur_coup_iterative(...) -> Option<Move> {
    let mut best_move = None;
    let mut tt = TranspositionTable::new();

    for depth in 1..=max_depth {
        let mv = meilleur_coup(board, tables, depth, &mut tt, &limits, best_move);

        if !limits.should_stop() && mv.is_some() {
            best_move = mv;
        }
    }

    best_move
}
```

Puis dans `meilleur_coup`, trier :

```rust
fn score_root_move(mv: &Move, previous_best: Option<Move>, tt_best: Option<Move>) -> i32 {
    if Some(*mv) == previous_best {
        return 2_000_000;
    }

    if Some(*mv) == tt_best {
        return 1_000_000;
    }

    score_ordre_coup(mv)
}
```

Et :

```rust
coups.sort_by_key(|mv| Reverse(score_root_move(mv, previous_best, tt_best)));
```

## Pourquoi `previous_best` avant `tt_best` ?

Parce que `previous_best` est le meilleur coup de la racine a la profondeur precedente.

Il est tres souvent meilleur que les autres pour commencer la profondeur suivante.

## Ce que ca change

Sans PV ordering :

```text
depth 8 commence peut-etre par un coup moyen
alpha monte lentement
```

Avec PV ordering :

```text
depth 8 commence par le meilleur coup de depth 7
alpha monte vite
```

---

# 6. Killer moves

## Objectif

Un killer move est un coup calme qui provoque un beta cutoff.

Exemple :

```text
a une certaine profondeur, un coup calme comme Re1 coupe la recherche
plus tard, a la meme profondeur, un autre noeud similaire arrive
on essaie Re1 tot
```

Les killer moves sont utiles pour les coups calmes.

Ils ne remplacent pas MVV-LVA :

```text
captures -> MVV-LVA
coups calmes -> killer/history
```

## Nouvelle notion : ply

Pour utiliser les killer moves, ta recherche doit connaitre le `ply`, c'est-a-dire la profondeur depuis la racine.

Exemple :

```text
root       -> ply 0
reponse    -> ply 1
coup apres -> ply 2
```

Où le faire :

```text
src/eval.rs, ajouter un parametre ply a evaluation_negamax_alpha_beta.
```

Signature idee :

```rust
pub fn evaluation_negamax_alpha_beta(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    tt: &mut TranspositionTable,
    limits: &SearchLimits,
    ply: usize,
    heuristics: &mut SearchHeuristics,
) -> i32
```

Oui, ca fait une signature plus longue. C'est normal dans un moteur.

## Structure des heuristiques

Où le faire :

```text
src/eval.rs, pres de SearchStats.
```

```rust
const MAX_PLY: usize = 128;

#[derive(Clone)]
pub struct SearchHeuristics {
    pub killer_moves: [[Option<Move>; 2]; MAX_PLY],
}

impl Default for SearchHeuristics {
    fn default() -> Self {
        Self {
            killer_moves: [[None; 2]; MAX_PLY],
        }
    }
}
```

Pourquoi deux killer moves ?

Parce qu'un seul coup killer peut etre trop fragile.

On garde :

```text
killer_moves[ply][0] -> meilleur killer recent
killer_moves[ply][1] -> deuxieme killer recent
```

## Mettre a jour les killer moves

Où le faire :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, au moment du beta cutoff.
```

Quand :

```rust
if alpha >= beta {
    stats.cutoffs += 1;
    break;
}
```

Avant le `break`, si le coup est calme :

```rust
if is_quiet_move(&coups) {
    store_killer_move(heuristics, ply, coups);
}
```

Helpers :

```rust
fn is_quiet_move(mv: &Move) -> bool {
    matches!(mv.flag, MoveFlag::Quiet | MoveFlag::DoublePawnPush | MoveFlag::Castling)
}

fn store_killer_move(heuristics: &mut SearchHeuristics, ply: usize, mv: Move) {
    if ply >= MAX_PLY {
        return;
    }

    if heuristics.killer_moves[ply][0] == Some(mv) {
        return;
    }

    heuristics.killer_moves[ply][1] = heuristics.killer_moves[ply][0];
    heuristics.killer_moves[ply][0] = Some(mv);
}
```

## Utiliser les killer moves dans le tri

Où le faire :

```text
src/eval.rs, dans score_ordre_coup_avec_tt ou une nouvelle fonction score_search_move.
```

```rust
fn score_search_move(
    mv: &Move,
    tt_best: Option<Move>,
    heuristics: &SearchHeuristics,
    ply: usize,
) -> i32 {
    if Some(*mv) == tt_best {
        return 2_000_000;
    }

    if ply < MAX_PLY {
        if heuristics.killer_moves[ply][0] == Some(*mv) {
            return 900_000;
        }

        if heuristics.killer_moves[ply][1] == Some(*mv) {
            return 800_000;
        }
    }

    score_ordre_coup(mv)
}
```

Puis :

```rust
moves.sort_by_key(|mv| Reverse(score_search_move(mv, tt_best, heuristics, ply)));
```

## Attention

Ne donne pas un score killer plus haut que le TT move.

Ordre conseille :

```text
1. TT move / PV move
2. captures fortes
3. killer move 1
4. killer move 2
5. history heuristic
6. quiet moves faibles
```

Selon les moteurs, captures et killer peuvent etre interclasses. Pour ton moteur, commence simple :

```text
TT move avant tout
captures avec MVV-LVA
killer moves pour les quiets
```

---

# 7. History heuristic

## Objectif

La history heuristic donne un score aux coups calmes qui ont souvent provoque des cutoffs.

Killer moves :

```text
depend du ply
stocke seulement 2 coups
```

History heuristic :

```text
globale
apprend que certains coups calmes sont souvent bons
```

## Structure simple

Où le faire :

```text
src/eval.rs, dans SearchHeuristics.
```

Version simple :

```rust
pub struct SearchHeuristics {
    pub killer_moves: [[Option<Move>; 2]; MAX_PLY],
    pub history: [[i32; 64]; 64],
}
```

Index :

```text
history[from][to]
```

Default :

```rust
impl Default for SearchHeuristics {
    fn default() -> Self {
        Self {
            killer_moves: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 64],
        }
    }
}
```

## Mise a jour

Quand un coup calme provoque un beta cutoff :

```rust
fn update_history(heuristics: &mut SearchHeuristics, mv: Move, depth: u32) {
    if !is_quiet_move(&mv) {
        return;
    }

    let bonus = (depth * depth) as i32;
    heuristics.history[mv.from as usize][mv.to as usize] += bonus;
}
```

Pourquoi `depth * depth` ?

Un cutoff profond est plus important qu'un cutoff proche de la feuille.

Exemple :

```text
cutoff depth 2 -> bonus 4
cutoff depth 6 -> bonus 36
```

## Eviter que les scores explosent

Ajoute une petite fonction de decay :

```rust
fn maybe_decay_history(heuristics: &mut SearchHeuristics) {
    let max_value = heuristics
        .history
        .iter()
        .flatten()
        .copied()
        .max()
        .unwrap_or(0);

    if max_value < 100_000 {
        return;
    }

    for row in heuristics.history.iter_mut() {
        for value in row.iter_mut() {
            *value /= 2;
        }
    }
}
```

Où l'appeler :

```text
apres chaque depth dans meilleur_coup_iterative
```

## Utilisation dans le score

Dans `score_search_move` :

```rust
if is_quiet_move(mv) {
    return heuristics.history[mv.from as usize][mv.to as usize];
}
```

Mais attention a l'echelle :

```rust
let history_score = heuristics.history[mv.from as usize][mv.to as usize];
return history_score.min(700_000);
```

Pourquoi ?

Tu ne veux pas qu'un quiet move history depasse le TT move.

---

# 8. Zobrist hash

## Probleme actuel

Aujourd'hui, ta cle de transposition ressemble a :

```rust
pub struct ClePosition {
    pub piece_bb: [u64; 14],
    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant_square: Option<u8>,
}
```

Et ta table :

```rust
pub type TranspositionTable = HashMap<ClePosition, TTEntry>;
```

C'est correct pour apprendre.

Mais c'est lourd :

```text
la cle contient [u64; 14]
le HashMap doit hasher toute cette structure
la table alloue dynamiquement
les lookups coutent plus cher
```

Zobrist remplace toute la position par :

```text
u64
```

## Idee

Chaque element de position a un nombre aleatoire 64 bits :

```text
piece blanche pion sur a2 -> random u64
piece noire dame sur d8   -> random u64
trait aux noirs           -> random u64
droits de roque KQkq      -> random u64
en passant e3             -> random u64
```

La cle finale :

```text
XOR de tous les elements presents
```

## Fichier a creer

```text
src/zobrist.rs
```

Et dans :

```text
src/lib.rs
```

ajouter :

```rust
pub mod zobrist;
```

## Structure des cles

```rust
pub struct ZobristKeys {
    pub pieces: [[u64; 64]; 12],
    pub side_to_move: u64,
    pub castling: [u64; 16],
    pub en_passant_file: [u64; 8],
}
```

Pourquoi `[12][64]` et pas `[14][64]` ?

Parce que `PiecesBlanches` et `PiecesNoires` sont des bitboards d'occupation, pas de vraies pieces.

Les vraies pieces sont :

```text
0..12
PionBlanc a RoiNoir
```

## Generateur deterministe

Ne prends pas `rand` au debut. Tu peux utiliser SplitMix64 pour generer les nombres.

```rust
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
```

Puis :

```rust
impl ZobristKeys {
    pub fn new() -> Self {
        let mut seed = 0x1234_5678_9ABC_DEF0;
        let mut pieces = [[0u64; 64]; 12];

        for piece in 0..12 {
            for square in 0..64 {
                pieces[piece][square] = splitmix64(&mut seed);
            }
        }

        let side_to_move = splitmix64(&mut seed);

        let mut castling = [0u64; 16];
        for value in castling.iter_mut() {
            *value = splitmix64(&mut seed);
        }

        let mut en_passant_file = [0u64; 8];
        for value in en_passant_file.iter_mut() {
            *value = splitmix64(&mut seed);
        }

        Self {
            pieces,
            side_to_move,
            castling,
            en_passant_file,
        }
    }
}
```

## Calculer la cle depuis le board

```rust
pub fn zobrist_hash(board: &CBoard, keys: &ZobristKeys) -> u64 {
    let mut hash = 0u64;

    for piece_index in 0..12 {
        let mut bb = board.piece_bb[piece_index];

        while bb != 0 {
            let square = bb.trailing_zeros() as usize;
            hash ^= keys.pieces[piece_index][square];
            bb &= bb - 1;
        }
    }

    if matches!(board.side_to_move, Color::Noir) {
        hash ^= keys.side_to_move;
    }

    hash ^= keys.castling[board.castling_rights as usize];

    if let Some(square) = board.en_passant_square {
        let file = (square % 8) as usize;
        hash ^= keys.en_passant_file[file];
    }

    hash
}
```

## Premiere version : calcul complet

Au debut, ne fais pas de hash incremental dans `make_move`.

Commence par :

```text
calculer zobrist_hash(board, keys) quand tu as besoin de la cle
```

C'est deja souvent plus rapide que hasher une grosse `ClePosition`.

Plus tard seulement :

```text
stocker board.zobrist_key
mettre a jour incrementale dans make/unmake
```

## Tests indispensables

```rust
#[test]
fn zobrist_same_position_same_hash() {
    let keys = ZobristKeys::new();
    let board1 = CBoard::init_position_depart();
    let board2 = CBoard::init_position_depart();

    assert_eq!(zobrist_hash(&board1, &keys), zobrist_hash(&board2, &keys));
}
```

Et :

```rust
#[test]
fn zobrist_changes_after_move_and_restores_after_unmake() {
    let keys = ZobristKeys::new();
    let mut board = CBoard::init_position_depart();
    let before = zobrist_hash(&board, &keys);

    let mv = trouver_coup_legal(&mut board, &tables, "e2e4").unwrap();
    let undo = make_move(&mut board, mv);

    assert_ne!(zobrist_hash(&board, &keys), before);

    unmake_move(&mut board, mv, undo);
    assert_eq!(zobrist_hash(&board, &keys), before);
}
```

---

# 9. Table de transposition fixe

## Probleme avec HashMap

`HashMap` est pratique, mais dans un moteur :

```text
allocations
rehash
cles lourdes
controle memoire moins precis
moins cache-friendly
```

Une table fixe ressemble a :

```text
Vec<TTEntry>
index = zobrist_key % taille
```

## Fichier concerne

```text
src/position_key.rs
```

Tu peux renommer plus tard en :

```text
src/transposition.rs
```

mais ce n'est pas obligatoire.

## Structure conseillee

```rust
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub flag: TTFlag,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize,
}
```

Taille puissance de 2 :

```rust
impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let raw_len = (size_mb * 1024 * 1024) / entry_size;
        let len = raw_len.next_power_of_two().max(1);

        Self {
            entries: vec![None; len],
            mask: len - 1,
        }
    }

    fn index(&self, key: u64) -> usize {
        key as usize & self.mask
    }
}
```

Pourquoi puissance de 2 ?

Parce que :

```rust
key as usize & mask
```

est plus rapide que :

```rust
key as usize % len
```

## Lecture

```rust
pub fn get(&self, key: u64) -> Option<TTEntry> {
    let entry = self.entries[self.index(key)]?;

    if entry.key == key {
        Some(entry)
    } else {
        None
    }
}
```

## Insertion avec remplacement

Regle simple :

```text
remplacer si la case est vide
remplacer si la nouvelle profondeur est >= ancienne profondeur
```

```rust
pub fn insert(&mut self, entry: TTEntry) {
    let index = self.index(entry.key);

    let replace = match self.entries[index] {
        None => true,
        Some(old) => entry.depth >= old.depth,
    };

    if replace {
        self.entries[index] = Some(entry);
    }
}
```

## Dans la recherche

Avant :

```rust
let key = cle_position(board);
if let Some(entry) = tt.get(&key) { ... }
tt.insert(key, TTEntry { ... });
```

Apres :

```rust
let key = zobrist_hash(board, keys);
if let Some(entry) = tt.get(key) { ... }
tt.insert(TTEntry {
    key,
    depth,
    score: meilleure,
    flag,
    best_move: meilleur_mv,
});
```

## Attention aux collisions

Deux positions peuvent avoir le meme index.

C'est pour ca que `TTEntry` stocke :

```rust
key: u64
```

Si l'index est pareil mais la cle differente :

```text
collision -> on ignore l'entree
```

---

# 10. Aspiration windows

## Objectif

Au lieu de chercher chaque profondeur avec :

```rust
alpha = -INF
beta = INF
```

on utilise le score de la profondeur precedente.

Exemple :

```text
depth 7 score = +42
depth 8 commence avec fenetre [42 - 50, 42 + 50]
donc [-8, 92]
```

Une fenetre plus petite provoque plus de cutoffs.

## Changement necessaire

`meilleur_coup` doit retourner :

```text
le meilleur coup
le score
```

Au lieu de :

```rust
Option<Move>
```

vise :

```rust
Option<(Move, i32)>
```

ou :

```rust
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
}
```

Je conseille :

```rust
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
}
```

## Dans iterative deepening

Où le faire :

```text
src/eval.rs, dans meilleur_coup_iterative.
```

Pseudo-code :

```rust
let mut previous_score = 0;
let mut window = 50;

for depth in 1..=max_depth {
    let mut alpha = previous_score - window;
    let mut beta = previous_score + window;

    loop {
        let result = meilleur_coup_with_window(
            board,
            tables,
            depth,
            alpha,
            beta,
            ...
        );

        if result.score <= alpha {
            alpha -= window;
            window *= 2;
            continue;
        }

        if result.score >= beta {
            beta += window;
            window *= 2;
            continue;
        }

        previous_score = result.score;
        best_move = result.best_move;
        break;
    }
}
```

## Explication

Si le score sort de la fenetre :

```text
fail low  -> score <= alpha -> fenetre trop haute
fail high -> score >= beta  -> fenetre trop basse
```

Alors on relance avec une fenetre plus large.

## Quand l'ajouter ?

Apres :

```text
PV ordering
TT stable
move ordering correct
```

Sinon tu risques de relancer trop souvent.

---

# 11. Null move pruning

## Objectif

Null move pruning teste :

```text
si je passe mon tour et que ma position est encore tellement bonne,
alors les vrais coups sont probablement bons aussi
```

On fait une recherche reduite apres un "coup nul".

Si meme en passant le tour on depasse beta :

```text
cutoff
```

## Attention

Aux echecs, on ne peut pas passer son tour.

Donc null move interdit si :

```text
le roi est en echec
```

Il faut aussi etre prudent en finales de pions, a cause du zugzwang.

## Fichier concerne

```text
src/eval.rs
src/make_move.rs si tu ajoutes make_null_move
```

## Fonctions null move

Où le faire :

```text
src/make_move.rs
```

Structure :

```rust
pub struct UndoNullMove {
    pub en_passant_square: Option<u8>,
    pub side_to_move: Color,
}
```

Fonction :

```rust
pub fn make_null_move(board: &mut CBoard) -> UndoNullMove {
    let undo = UndoNullMove {
        en_passant_square: board.en_passant_square,
        side_to_move: board.side_to_move,
    };

    board.en_passant_square = None;
    board.side_to_move = match board.side_to_move {
        Color::Blanc => Color::Noir,
        Color::Noir => Color::Blanc,
    };

    undo
}

pub fn unmake_null_move(board: &mut CBoard, undo: UndoNullMove) {
    board.en_passant_square = undo.en_passant_square;
    board.side_to_move = undo.side_to_move;
}
```

Pas besoin de changer les pieces.

## Dans alpha-beta

Où le faire :

```text
src/eval.rs, dans evaluation_negamax_alpha_beta, apres le test TT et avant generate_legal_move.
```

Conditions :

```rust
let in_check = is_king_in_check(board, tables, board.side_to_move);

if depth >= 3 && !in_check && has_non_pawn_material(board) {
    let reduction = 2;
    let undo = make_null_move(board);

    let score = -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 1 - reduction,
        -beta,
        -beta + 1,
        stats,
        tt,
        limits,
        ply + 1,
        heuristics,
    );

    unmake_null_move(board, undo);

    if score >= beta {
        stats.null_cutoffs += 1;
        return beta;
    }
}
```

## `has_non_pawn_material`

Où le faire :

```text
src/eval.rs
```

```rust
fn has_non_pawn_material(board: &CBoard) -> bool {
    let pieces = [
        Pieces::CavalierBlanc,
        Pieces::FouBlanc,
        Pieces::TourBlanche,
        Pieces::DameBlanche,
        Pieces::CavalierNoir,
        Pieces::FouNoir,
        Pieces::TourNoire,
        Pieces::DameNoire,
    ];

    pieces
        .iter()
        .any(|piece| board.piece_bb[*piece as usize] != 0)
}
```

Pourquoi ?

Le null move est dangereux dans les finales de pions, parce que parfois etre oblige de jouer est un desavantage.

C'est le zugzwang.

## Quand l'ajouter ?

Apres :

```text
TT correcte
quiescence correcte
tests perft OK
```

---

# 12. Late Move Reductions, LMR

## Objectif

LMR part d'une idee :

```text
apres avoir trie les coups,
les premiers coups sont probablement importants
les coups tardifs et calmes sont souvent mauvais
```

Donc on cherche les coups tardifs moins profondement.

Si le coup semble finalement bon, on le re-cherche a profondeur complete.

## Conditions de base

Ne reduis pas :

```text
le premier coup
les captures
les promotions
les positions en echec
les petites profondeurs
```

Conditions typiques :

```rust
let can_reduce =
    depth >= 3
    && move_index >= 4
    && !in_check
    && is_quiet_move(&mv);
```

## Où le faire

```text
src/eval.rs, dans la boucle for moves de evaluation_negamax_alpha_beta.
```

Il faut connaitre `move_index`.

Au lieu de :

```rust
for coups in moves {
```

faire :

```rust
for (move_index, coups) in moves.into_iter().enumerate() {
```

## Code conceptuel

```rust
let undo = make_move(board, coups);

let mut score;

let can_reduce =
    depth >= 3
    && move_index >= 4
    && !in_check
    && is_quiet_move(&coups);

if can_reduce {
    score = -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 2,
        -alpha - 1,
        -alpha,
        stats,
        tt,
        limits,
        ply + 1,
        heuristics,
    );

    if score > alpha {
        stats.lmr_researches += 1;

        score = -evaluation_negamax_alpha_beta(
            board,
            tables,
            depth - 1,
            -beta,
            -alpha,
            stats,
            tt,
            limits,
            ply + 1,
            heuristics,
        );
    }
} else {
    score = -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 1,
        -beta,
        -alpha,
        stats,
        tt,
        limits,
        ply + 1,
        heuristics,
    );
}

unmake_move(board, coups, undo);
```

## Explication

Recherche reduite :

```rust
depth - 2
```

Fenetre nulle :

```rust
-alpha - 1, -alpha
```

Elle sert seulement a savoir :

```text
est-ce que ce coup peut battre alpha ?
```

Si oui, on re-cherche avec la vraie fenetre :

```rust
-beta, -alpha
```

## Risque

LMR peut rendre le moteur plus rapide mais moins stable si :

```text
move ordering mauvais
quiescence faible
TT instable
```

Donc ne l'ajoute pas trop tot.

---

# 13. Quiescence efficace

Tu as deja un document dedie :

```text
optimisation_moteur_echecs_quiescence_tutoriel.md
```

Le resume :

```text
1. ajouter QUIESCENCE_DEPTH
2. appeler quiescence avec qdepth > 0
3. si le roi est en echec, generer tous les coups legaux
4. sinon faire stand_pat
5. generer directement les coups tactiques
6. trier captures/promotions
7. ajouter delta pruning
8. plus tard ajouter SEE
```

Point cle :

```text
quiescence hors echec -> captures/promotions/en passant
quiescence en echec   -> tous les coups legaux
```

Pourquoi ?

Parce qu'une sortie d'echec peut etre un coup calme.

## Où sont les changements principaux

```text
src/eval.rs
    QUIESCENCE_DEPTH
    quiescence
    score_ordre_coup_quiescence
    delta pruning

src/pseudo_legal_move.rs
    generate_pseudo_tactical_move

src/legal_move.rs
    generate_tactical_legal_move branche sur generate_pseudo_tactical_move
```

## Quand faire la quiescence dans l'ordre global ?

Je conseille :

```text
apres MVV-LVA propre
avant null move pruning et LMR
```

Parce que null move et LMR sont plus dangereux si la feuille de recherche est tactiquement faible.

---

# 14. Relier toutes les optimisations dans le score des coups

A terme, tu veux une fonction centrale de scoring.

Où le faire :

```text
src/eval.rs
```

Signature :

```rust
fn score_search_move(
    mv: &Move,
    tt_best: Option<Move>,
    heuristics: &SearchHeuristics,
    ply: usize,
) -> i32
```

Ordre conseille :

```rust
fn score_search_move(
    mv: &Move,
    tt_best: Option<Move>,
    heuristics: &SearchHeuristics,
    ply: usize,
) -> i32 {
    if Some(*mv) == tt_best {
        return 2_000_000;
    }

    if matches!(mv.flag, MoveFlag::Capture | MoveFlag::EnPassant | MoveFlag::Promotion | MoveFlag::PromotionCapture) {
        return 1_000_000 + score_ordre_coup(mv);
    }

    if ply < MAX_PLY {
        if heuristics.killer_moves[ply][0] == Some(*mv) {
            return 900_000;
        }

        if heuristics.killer_moves[ply][1] == Some(*mv) {
            return 800_000;
        }
    }

    if is_quiet_move(mv) {
        return heuristics.history[mv.from as usize][mv.to as usize].min(700_000);
    }

    0
}
```

Puis partout dans la recherche normale :

```rust
moves.sort_by_key(|mv| Reverse(score_search_move(mv, tt_best, heuristics, ply)));
```

Cette fonction devient le coeur du move ordering.

---

# 15. Tests et validation apres chaque etape

## Toujours lancer

```bash
cargo test
```

Puis :

```bash
cargo test --release perft
```

## Pour les changements de recherche

Teste une position fixe.

Note :

```text
depth
temps
nodes
qnodes
cutoffs
tt_hits
tt_cutoffs
best_move
score
```

## Pour Zobrist

Tests indispensables :

```text
meme position -> meme hash
position differente -> hash different
make/unmake -> hash revient pareil
side_to_move change -> hash change
castling_rights change -> hash change
en_passant_square change -> hash change
```

## Pour TT fixe

Tests :

```text
insert puis get meme key -> entree retrouvee
get autre key meme index -> None
remplacement profondeur faible -> ne remplace pas
remplacement profondeur forte -> remplace
```

## Pour killer/history

Pas besoin de tester la force directement.

Teste surtout :

```text
store_killer_move place le nouveau coup en slot 0
ancien slot 0 passe en slot 1
doublon ne duplique pas
history augmente sur cutoff calme
history ne change pas sur capture
```

---

# 16. Roadmap concrete

Voici une feuille de route realiste.

## Phase 1 : move ordering solide

```text
[ ] MVV-LVA propre
[ ] score_search_move central
[ ] PV move ordering root
[ ] killer moves
[ ] history heuristic
```

Objectif :

```text
plus de cutoffs
moins de nodes
meilleur premier coup
```

## Phase 2 : table de transposition plus rapide

```text
[ ] Zobrist hash calcule depuis board
[ ] remplacer ClePosition par u64
[ ] TT fixe Vec<Option<TTEntry>>
[ ] remplacement par profondeur
```

Objectif :

```text
lookup plus rapide
moins d'allocation
memoire controlee
```

## Phase 3 : recherche plus agressive

```text
[ ] aspiration windows
[ ] quiescence efficace
[ ] null move pruning
[ ] late move reductions
```

Objectif :

```text
atteindre une profondeur plus grande sans perdre trop de precision
```

---

# 17. Ce qu'il faut eviter

## Tout faire en meme temps

Mauvais plan :

```text
Zobrist + TT fixe + null move + LMR + quiescence en une seule fois
```

Si le moteur joue mal, tu ne sauras pas pourquoi.

## Ajouter LMR trop tot

LMR depend fortement du move ordering.

Si ton tri est mauvais :

```text
tu reduis peut-etre un bon coup
```

## Ajouter null move sans garde-fous

Null move est puissant, mais dangereux en zugzwang.

Toujours commencer avec :

```text
depth >= 3
pas en echec
has_non_pawn_material
reduction prudente
```

## Partager une HashMap avec Mutex en multithread

Ce document vise le mono-thread, mais note importante :

```text
Mutex<HashMap<...>> dans une TT appelee des millions de fois peut etre catastrophique
```

Pour le multi-thread, il faudra une TT pensee pour ca.

---

# 18. Resume ultra-court

Si tu veux le plus gros gain propre :

```text
1. meilleur move ordering
2. Zobrist
3. TT fixe
4. quiescence efficace
5. null move
6. LMR
```

La force d'un moteur ne vient pas seulement de chercher plus vite.

Elle vient de :

```text
chercher les bons coups en premier
memoriser les positions deja vues
couper les branches inutiles
ne pas s'arreter au milieu des captures
reduire prudemment les coups peu prometteurs
```

Si tu suis cet ordre, tu construis un moteur que tu peux garder et ameliorer, pas une suite de patchs fragiles.
