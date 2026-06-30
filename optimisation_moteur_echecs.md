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

Au lieu de chercher chaque profondeur avec une fenetre complete:

```rust
alpha = -INF;
beta = INF;
```

on repart du score trouve a la profondeur precedente.

Exemple:

```text
depth 7 score = +42
depth 8 commence avec [42 - 50, 42 + 50]
donc [-8, 92]
```

Si le vrai score reste dans cette fenetre, alpha-beta coupe plus vite. Si le score sort de la fenetre, on relance avec une fenetre plus large.

## Etat de ton code avant cette etape

Dans `src/eval.rs`, tu as deja:

```text
meilleur_coup_iterative(...) -> Option<Move>
meilleur_coup(...) -> Option<Move>
evaluation_negamax_alpha_beta(..., alpha, beta, ...) -> i32
```

Tu as aussi deja commence a ajouter:

```rust
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
}
```

Si cette structure est deja presente dans `src/eval.rs`, ne la recree pas. Garde-la et utilise-la. Si elle n'est pas presente, ajoute-la comme indique ci-dessous.

## Fichiers a modifier

```text
src/eval.rs
```

Rien a modifier dans `src/web_server.rs`, car `meilleur_coup_iterative` doit continuer a retourner `Option<Move>`.

## Etape 10.1 : ajouter une constante de fenetre

Placement: dans `src/eval.rs`, au debut du fichier, pres de:

```rust
const SCORE_MAT: i32 = 100_000;
const INF: i32 = 1_000_000;
const MAX_PLY: usize = 128;
```

Ajouter:

```rust
const ASPIRATION_WINDOW: i32 = 50;
```

## Etape 10.2 : verifier ou ajouter `SearchResult`

Placement: dans `src/eval.rs`, juste apres `SearchHeuristics` ou juste avant `impl Default for SearchHeuristics`.

Si tu as deja ce bloc, garde-le:

```rust
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
}
```

Si tu veux le rendre plus pratique pour debug, tu peux remplacer sa declaration par:

```rust
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
}
```

## Etape 10.3 : changer la signature de `meilleur_coup`

Placement: dans `src/eval.rs`, chercher:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    tt: &mut TranspositionTable,
    keys : &ZobristKeys,
    limits: &SearchLimits,
    previous_best : Option<Move>,
    heuristics : &mut SearchHeuristics,
) -> Option<Move> {
```

Remplacer uniquement la signature par:

```rust
pub fn meilleur_coup(
    board: &mut CBoard,
    tables: &AttackTables,
    depth: u32,
    tt: &mut TranspositionTable,
    keys: &ZobristKeys,
    limits: &SearchLimits,
    previous_best: Option<Move>,
    heuristics: &mut SearchHeuristics,
    root_alpha: i32,
    root_beta: i32,
) -> SearchResult {
```

Ce que tu ajoutes concretement:

```text
root_alpha: i32
root_beta: i32
```

Ce que tu retires concretement:

```text
) -> Option<Move>
```

Ce que tu mets a la place:

```text
) -> SearchResult
```

## Etape 10.4 : modifier le debut de `meilleur_coup`

Dans `meilleur_coup`, chercher ce bloc:

```rust
let mut meilleur_mv = None;
let mut meilleur_score = -INF;
let mut alpha = -INF;
let beta = INF;
if depth == 0 {
    return None;
}
```

Remplacer par:

```rust
let mut meilleur_mv = None;
let mut meilleur_score = -INF;
let mut alpha = root_alpha;
let beta = root_beta;

if depth == 0 {
    return SearchResult {
        best_move: None,
        score: evaluation_negamax(board),
    };
}

if coups.is_empty() {
    let score = if is_king_in_check(board, tables, board.side_to_move) {
        -SCORE_MAT
    } else {
        0
    };

    return SearchResult {
        best_move: None,
        score,
    };
}
```

Pourquoi ajouter le cas `coups.is_empty()` ici?

Parce qu'avant, si la racine n'avait aucun coup legal, `meilleur_score` restait a `-INF`. Avec `SearchResult`, il faut retourner un score propre.

## Etape 10.5 : ajouter un cutoff a la racine

Dans `meilleur_coup`, chercher la fin de la boucle:

```rust
if score > meilleur_score {
    meilleur_mv = Some(mv);
    meilleur_score = score;
}
alpha = alpha.max(score);
```

Remplacer par:

```rust
if score > meilleur_score {
    meilleur_mv = Some(mv);
    meilleur_score = score;
}

alpha = alpha.max(score);

if alpha >= beta {
    break;
}
```

Pourquoi?

Avec une fenetre d'aspiration, `beta` peut etre proche du score attendu. Si un coup depasse `beta`, cette recherche est un `fail high`; inutile de continuer tous les coups racine, l'iterative deepening relancera avec une fenetre plus large.

## Etape 10.6 : changer le retour final de `meilleur_coup`

Dans `meilleur_coup`, chercher tout a la fin:

```rust
meilleur_mv
```

Remplacer par:

```rust
SearchResult {
    best_move: meilleur_mv,
    score: meilleur_score,
}
```

Si tu as des `println!` de stats juste avant, garde-les. Ils restent utiles.

## Etape 10.7 : remplacer l'appel dans `meilleur_coup_iterative`

Placement: dans `src/eval.rs`, dans `meilleur_coup_iterative`.

Chercher le bloc actuel:

```rust
for depth in 1..=max_depth {
    if limits.should_stop() {
        break;
    }
    let mv = meilleur_coup(board, tables, depth, &mut tt,&keys, &limits,best_move,&mut heuristics);

    if !limits.should_stop() && mv.is_some() {
        best_move = mv;
    }

    println!("deph {} -> {:?}", depth, best_move);
}
```

Avant ce `for`, ajouter:

```rust
let mut previous_score = 0;
```

Puis remplacer tout le bloc `for depth in 1..=max_depth { ... }` par:

```rust
for depth in 1..=max_depth {
    if limits.should_stop() {
        break;
    }

    let use_aspiration = depth >= 2 && best_move.is_some();
    let mut window = ASPIRATION_WINDOW;
    let mut alpha = if use_aspiration {
        (previous_score - window).max(-INF)
    } else {
        -INF
    };
    let mut beta = if use_aspiration {
        (previous_score + window).min(INF)
    } else {
        INF
    };

    loop {
        if limits.should_stop() {
            break;
        }

        let result = meilleur_coup(
            board,
            tables,
            depth,
            &mut tt,
            &keys,
            &limits,
            best_move,
            &mut heuristics,
            alpha,
            beta,
        );

        if limits.should_stop() {
            break;
        }

        if result.best_move.is_none() {
            break;
        }

        if use_aspiration && result.score <= alpha {
            alpha = (alpha - window).max(-INF);
            window = (window * 2).min(INF);
            continue;
        }

        if use_aspiration && result.score >= beta {
            beta = (beta + window).min(INF);
            window = (window * 2).min(INF);
            continue;
        }

        previous_score = result.score;
        best_move = result.best_move;
        break;
    }

    println!("depth {} -> {:?}, score {}", depth, best_move, previous_score);
}
```

Ce que tu retires:

```text
let mv = meilleur_coup(...)
if mv.is_some() { best_move = mv; }
```

Ce que tu mets a la place:

```text
let result = meilleur_coup(..., alpha, beta)
si fail low  -> elargir alpha
si fail high -> elargir beta
sinon previous_score = result.score et best_move = result.best_move
```

## Etape 10.8 : verifier les appels restants

Commande:

```bash
rg -n "meilleur_coup\\(" src
```

Il doit rester:

```text
la definition de meilleur_coup
un appel dans meilleur_coup_iterative
```

Si un autre appel existe, il faut lui ajouter les deux derniers arguments:

```rust
-INF,
INF,
```

## Verification

Compiler:

```bash
cargo check
```

Tester:

```bash
cargo test
```

Puis lancer une recherche et verifier dans les logs:

```text
depth 1 -> ...
depth 2 -> ...
depth 3 -> ...
```

Si tu veux voir les relances, ajoute temporairement dans les deux branches fail low/fail high:

```rust
println!("aspiration fail low depth {}, score {}, alpha {}", depth, result.score, alpha);
println!("aspiration fail high depth {}, score {}, beta {}", depth, result.score, beta);
```

Retire ces `println!` apres debug.

## Quand l'ajouter

Seulement quand:

```text
PV ordering fonctionne
TT stable
move ordering correct
```

Sinon le score varie trop entre les profondeurs et la fenetre relance souvent.

---

# 11. Null move pruning

## Objectif

Null move pruning coupe une branche quand la position est tellement bonne que meme "passer son tour" reste suffisant pour depasser `beta`.

Attention: aux echecs on ne peut pas passer son tour. C'est une approximation. Elle est dangereuse en zugzwang, surtout dans les finales de pions.

## Fichiers a modifier

```text
src/make_move.rs
src/eval.rs
```

## Etape 11.1 : ajouter `UndoNullMove`

Placement: dans `src/make_move.rs`, juste apres la struct `UndoMove`.

Ajouter:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoNullMove {
    pub en_passant_square: Option<u8>,
    pub side_to_move: Color,
}
```

Tu n'as pas besoin de stocker les bitboards, car un null move ne bouge aucune piece.

## Etape 11.2 : ajouter `make_null_move` et `unmake_null_move`

Placement: dans `src/make_move.rs`, tout en bas du fichier, apres `unmake_move`.

Ajouter:

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

Ce que tu ne modifies pas:

```text
piece_bb
occupe_bb
vide_bb
white_king_square
black_king_square
castling_rights
```

## Etape 11.3 : importer les fonctions null move dans `eval.rs`

Placement: en haut de `src/eval.rs`.

Chercher:

```rust
use crate::make_move::{make_move, unmake_move};
```

Remplacer par:

```rust
use crate::make_move::{make_move, make_null_move, unmake_move, unmake_null_move};
```

## Etape 11.4 : ajouter `has_non_pawn_material`

Placement: dans `src/eval.rs`, juste apres `is_quiet_move`.

Ajouter:

```rust
fn has_non_pawn_material(board: &CBoard, color: Color) -> bool {
    let pieces = match color {
        Color::Blanc => [
            Pieces::CavalierBlanc,
            Pieces::FouBlanc,
            Pieces::TourBlanche,
            Pieces::DameBlanche,
        ],
        Color::Noir => [
            Pieces::CavalierNoir,
            Pieces::FouNoir,
            Pieces::TourNoire,
            Pieces::DameNoire,
        ],
    };

    pieces
        .iter()
        .any(|piece| board.piece_bb[*piece as usize] != 0)
}
```

Pourquoi par couleur?

Parce que le danger de zugzwang concerne surtout le camp qui doit jouer. Si le camp au trait n'a que des pions, le null move est plus risque.

## Etape 11.5 : inserer le null move dans alpha-beta

Placement: dans `src/eval.rs`, dans `evaluation_negamax_alpha_beta`.

Prerequis pratique: les blocs ci-dessous utilisent `QUIESCENCE_DEPTH`. Si cette constante n'existe pas encore dans ton code, fais d'abord les etapes 13.1 et 13.2, ou garde temporairement le `4` en dur dans les deux blocs de ce chapitre.

Chercher ce bloc:

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta, QUIESCENCE_DEPTH, stats, limits);
}

let mut moves = generate_legal_move(board, tables);
```

Si tu n'as pas encore fait le chapitre 13 et que ton code contient encore le `4` en dur, le bloc a chercher sera:

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta, 4, stats, limits);
}

let mut moves = generate_legal_move(board, tables);
```

Remplacer par:

```rust
if depth == 0 {
    return quiescence(board, tables, alpha, beta, QUIESCENCE_DEPTH, stats, limits);
}

let in_check = is_king_in_check(board, tables, board.side_to_move);

if depth >= 3 && !in_check && has_non_pawn_material(board, board.side_to_move) {
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
        keys,
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

let mut moves = generate_legal_move(board, tables);
```

Ce bloc doit etre avant `generate_legal_move`, parce que son but est justement d'eviter de generer et chercher tous les coups quand une coupure rapide suffit.

## Verification

Compiler:

```bash
cargo check
```

Puis lancer une recherche et verifier que:

```text
stats.null_cutoffs augmente parfois
le moteur ne plante pas
les tests perft restent OK
```

Les tests perft ne doivent pas changer, car le null move ne modifie pas la generation de coups.

Commande:

```bash
cargo test --release perft
```

---

# 12. Late Move Reductions, LMR

## Objectif

LMR reduit la profondeur pour les coups calmes qui arrivent tard dans l'ordre des coups. Si un coup reduit semble finalement bon, on le re-cherche a profondeur complete.

## Fichier a modifier

```text
src/eval.rs
```

## Etape 12.1 : reutiliser `in_check`

Si tu as ajoute le null move, tu as deja ce bloc avant `generate_legal_move`:

```rust
let in_check = is_king_in_check(board, tables, board.side_to_move);
```

Si tu n'as pas ajoute le null move, ajoute cette ligne juste avant:

```rust
let mut moves = generate_legal_move(board, tables);
```

## Etape 12.2 : changer la boucle des coups

Dans `evaluation_negamax_alpha_beta`, chercher:

```rust
for coups in moves {
```

Remplacer par:

```rust
for (move_index, coups) in moves.into_iter().enumerate() {
```

Ce changement donne l'indice du coup apres tri. Les premiers coups ont `move_index = 0`, `1`, `2`.

## Etape 12.3 : remplacer le calcul du score dans la boucle

Dans la boucle, chercher le bloc actuel:

```rust
let undo = make_move(board, coups);
let score = -evaluation_negamax_alpha_beta(
    board,
    tables,
    depth - 1,
    -beta,
    -alpha,
    stats,
    tt,
    keys,
    limits,
    ply +1,
    heuristics,
);
unmake_move(board, coups, undo);
```

Remplacer par:

```rust
let undo = make_move(board, coups);

let can_reduce =
    depth >= 3
    && move_index >= 4
    && !in_check
    && is_quiet_move(&coups);

let mut score = if can_reduce {
    -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 2,
        -alpha - 1,
        -alpha,
        stats,
        tt,
        keys,
        limits,
        ply + 1,
        heuristics,
    )
} else {
    -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 1,
        -beta,
        -alpha,
        stats,
        tt,
        keys,
        limits,
        ply + 1,
        heuristics,
    )
};

if can_reduce && score > alpha {
    stats.lmr_researches += 1;

    score = -evaluation_negamax_alpha_beta(
        board,
        tables,
        depth - 1,
        -beta,
        -alpha,
        stats,
        tt,
        keys,
        limits,
        ply + 1,
        heuristics,
    );
}

unmake_move(board, coups, undo);
```

Ce que tu ne dois pas retirer:

```text
le bloc qui met a jour meilleure/meilleur_mv
le bloc alpha = alpha.max(score)
le bloc cutoff alpha >= beta
le tt.insert final
```

Tu remplaces seulement la partie qui fait `make_move`, appelle la recherche recursive, puis `unmake_move`.

## Etape 12.4 : verifier le cutoff apres LMR

Le bloc qui suit doit rester comme ca:

```rust
if score > meilleure {
    meilleure = score;
    meilleur_mv = Some(coups);
}

alpha = alpha.max(score);

if alpha >= beta {
    if is_quiet_move(&coups) {
        score_killer_move(heuristics, ply, coups);
    }

    stats.cutoffs += 1;
    break;
}
```

Le chapitre 14 explique comment ajouter aussi `update_history` dans ce bloc.

## Verification

Compiler:

```bash
cargo check
```

Puis surveiller:

```text
stats.lmr_researches
nodes
best_move
```

Si `lmr_researches` explose ou si le meilleur coup devient instable, augmente la prudence:

```rust
move_index >= 6
```

au lieu de:

```rust
move_index >= 4
```

---

# 13. Quiescence efficace

Tu as deja un document dedie:

```text
optimisation_moteur_echecs_quiescence_tutoriel.md
```

Mais voici les changements concrets a faire dans ce fichier-ci pour ton code actuel.

## Fichiers a modifier

```text
src/eval.rs
```

Plus tard seulement:

```text
src/pseudo_legal_move.rs
src/legal_move.rs
```

## Etape 13.1 : ajouter une constante

Placement: dans `src/eval.rs`, pres de `SCORE_MAT`, `INF`, `MAX_PLY`.

Ajouter:

```rust
const QUIESCENCE_DEPTH: u32 = 4;
```

## Etape 13.2 : retirer le nombre magique `4`

Dans `evaluation_negamax_alpha_beta`, chercher:

```rust
return quiescence(board, tables, alpha, beta, 4, stats, limits);
```

Remplacer par:

```rust
return quiescence(board, tables, alpha, beta, QUIESCENCE_DEPTH, stats, limits);
```

Ce que tu retires:

```text
le 4 en dur
```

Ce que tu ajoutes:

```text
QUIESCENCE_DEPTH
```

## Etape 13.3 : corriger le cas ou le roi est en echec

Dans `quiescence`, chercher:

```rust
let mut moves = generate_tactical_legal_move(board, tables);
```

Remplacer par:

```rust
let mut moves = if in_check {
    generate_legal_move(board, tables)
} else {
    generate_tactical_legal_move(board, tables)
};
```

Pourquoi?

Si le roi est en echec, une sortie d'echec peut etre un coup calme. Une quiescence qui ne regarde que les captures peut evaluer a tort une position comme perdue ou mate.

## Etape 13.4 : garder la suite de `quiescence`

Ne retire pas:

```rust
moves.sort_by_key(|mv| Reverse(score_ordre_coup(mv)));
```

Ne retire pas non plus la boucle:

```rust
for mv in moves {
    ...
}
```

Cette etape corrige seulement le type de coups generes quand le roi est en echec.

## Verification

Compiler:

```bash
cargo check
```

Tester:

```bash
cargo test
```

Surveiller ensuite:

```text
qnodes augmente
le moteur ne ralentit pas de facon explosive
les positions d'echec ne donnent pas de scores absurdes
```

Si c'est trop lent, baisse temporairement:

```rust
const QUIESCENCE_DEPTH: u32 = 2;
```

---

# 14. Relier toutes les optimisations dans le score des coups

Ce chapitre corrige un point important dans ton code actuel.

Dans `score_search_move`, tu testes actuellement les coups calmes avant les killer moves. Comme les killer moves sont justement des coups calmes, le code peut retourner le score history avant d'atteindre le test killer.

## Fichier a modifier

```text
src/eval.rs
```

## Etape 14.1 : remplacer `score_search_move`

Chercher toute la fonction:

```rust
fn score_search_move(mv : &Move,tt_best: Option<Move>,heuristics: &SearchHeuristics,ply: usize)->i32{
```

Remplacer toute la fonction par:

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

    if matches!(
        mv.flag,
        MoveFlag::Capture
            | MoveFlag::EnPassant
            | MoveFlag::Promotion
            | MoveFlag::PromotionCapture
    ) {
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

Ordre voulu:

```text
TT best move
captures/promotions
killer moves
history
reste
```

## Etape 14.2 : brancher vraiment `update_history`

Tu as deja:

```rust
fn update_history(heuristics : &mut SearchHeuristics,mv : Move,depth: u32)
fn maybe_decay_history(heuristics: &mut SearchHeuristics)
```

Mais dans ton bloc cutoff, tu ne mets actuellement a jour que les killer moves.

Dans `evaluation_negamax_alpha_beta`, chercher:

```rust
if alpha >= beta {
    if is_quiet_move(&coups){
    score_killer_move(heuristics, ply, coups);
    }

    stats.cutoffs += 1;
    break;
}
```

Remplacer par:

```rust
if alpha >= beta {
    if is_quiet_move(&coups) {
        score_killer_move(heuristics, ply, coups);
        update_history(heuristics, coups, depth);
        maybe_decay_history(heuristics);
    }

    stats.cutoffs += 1;
    break;
}
```

Ce que tu ajoutes:

```text
update_history(...)
maybe_decay_history(...)
```

Pourquoi?

Sinon `history` reste presque toujours a zero et ne sert pas vraiment dans `score_search_move`.

## Verification

Compiler:

```bash
cargo check
```

Puis ajouter temporairement un log dans `maybe_decay_history` ou inspecter en debug si les valeurs history augmentent. Retire le log apres verification.

---

# 15. Tests et validation apres chaque etape

## Commandes minimales

Apres chaque chapitre implemente:

```bash
cargo check
cargo test
cargo test --release perft
```

## Ce qu'il faut noter dans les logs

Dans `meilleur_coup`, tu affiches deja:

```rust
println!("Nodes : {}", stats.nodes);
println!("QNodes : {}", stats.qnodes);
println!("Cutoffs : {}", stats.cutoffs);
println!("QCutoffs : {}", stats.qcutoffs);
```

Ajoute temporairement si necessaire:

```rust
println!("Null cutoffs : {}", stats.null_cutoffs);
println!("LMR researches : {}", stats.lmr_researches);
```

Retire ou commente ces logs quand tu joues via le serveur web, sinon la sortie devient vite illisible.

## Validation par optimisation

Aspiration windows:

```text
le moteur compile
meilleur_coup_iterative retourne toujours Option<Move>
les logs montrent un score par profondeur
les fail low/fail high ne bouclent pas sans fin
```

Null move:

```text
perft inchange
null_cutoffs augmente parfois
pas de null move si le roi est en echec
pas de null move si le camp au trait n'a que des pions
```

LMR:

```text
lmr_researches reste raisonnable
best_move ne change pas de facon chaotique a faible profondeur
nodes baisse ou reste stable
```

Quiescence:

```text
qnodes augmente
qcutoffs augmente parfois
les positions d'echec sont gerees avec tous les coups legaux
```

Move ordering:

```text
killer moves testes avant history
history augmente seulement sur cutoff calme
captures restent prioritaires
```

---

# 16. Roadmap concrete

Voici l'ordre conseille a partir de ton code actuel.

## Phase 1 : rendre l'existant coherent

```text
[ ] remplacer score_search_move pour tester killer avant history
[ ] appeler update_history dans le cutoff alpha >= beta
[ ] remplacer le 4 de quiescence par QUIESCENCE_DEPTH
[ ] corriger quiescence en echec avec generate_legal_move
```

Pourquoi cette phase d'abord?

Parce que aspiration windows, null move et LMR dependent tous d'un score de recherche stable et d'un move ordering correct.

## Phase 2 : aspiration windows

```text
[ ] ajouter ASPIRATION_WINDOW
[ ] garder/ajouter SearchResult
[ ] changer meilleur_coup -> SearchResult
[ ] ajouter root_alpha/root_beta a meilleur_coup
[ ] ajouter la boucle fail low/fail high dans meilleur_coup_iterative
[ ] verifier avec cargo check et cargo test
```

Objectif:

```text
utiliser le score precedent pour reduire la fenetre de recherche
```

## Phase 3 : null move pruning

```text
[ ] ajouter UndoNullMove dans src/make_move.rs
[ ] ajouter make_null_move / unmake_null_move
[ ] importer ces fonctions dans src/eval.rs
[ ] ajouter has_non_pawn_material(board, color)
[ ] inserer le bloc null move avant generate_legal_move
[ ] verifier null_cutoffs et perft
```

Objectif:

```text
couper vite les positions clairement au-dessus de beta
```

## Phase 4 : LMR

```text
[ ] ajouter move_index avec enumerate
[ ] reduire seulement les coups calmes tardifs
[ ] re-chercher a profondeur complete si score > alpha
[ ] surveiller lmr_researches
```

Objectif:

```text
chercher moins profondement les coups probablement faibles
```

---

# 17. Ce qu'il faut eviter

## Ne pas modifier toutes les optimisations en une fois

Mauvais plan:

```text
aspiration windows + null move + LMR + quiescence dans le meme commit
```

Bon plan:

```text
une optimisation
cargo check
cargo test
petit test de recherche
commit ou note de sauvegarde
optimisation suivante
```

## Ne pas ajouter LMR avant de corriger `score_search_move`

Si le tri met un bon coup trop tard, LMR peut reduire ce bon coup. Donc corrige d'abord:

```text
TT move
captures
killer moves
history
```

## Ne pas utiliser null move en finale de pions

Garde toujours:

```rust
has_non_pawn_material(board, board.side_to_move)
```

Sans ca, le moteur peut rater des zugzwangs simples.

## Ne pas laisser les logs permanents dans le serveur web

Les logs sont utiles pendant l'optimisation, mais le serveur appelle l'IA via:

```text
src/web_server.rs -> jouer_coup_ia -> meilleur_coup_iterative
```

Donc trop de `println!` peut rendre le serveur tres bruyant.

---

# 18. Resume ultra-court

Ordre concret:

```text
1. Corriger score_search_move.
2. Brancher update_history.
3. Corriger quiescence en echec.
4. Ajouter SearchResult et aspiration windows.
5. Ajouter null move avec garde-fous.
6. Ajouter LMR prudemment.
```

Pour chaque etape, ne te contente pas de copier le concept. Cherche le bloc indique, remplace exactement le code indique, compile, teste, puis seulement ensuite passe a l'etape suivante.
