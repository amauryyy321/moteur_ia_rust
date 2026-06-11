# moteur_ia - moteur d'echecs en Rust

Projet personnel d'apprentissage: construire progressivement un moteur d'echecs en Rust avec une representation par bitboards, puis ajouter une IA capable de jouer.

Ce README sert a donner assez de contexte pour reprendre le projet sans repartir de zero. Il doit rester proche de l'etat reel du code.

## Consignes

- Repondre en francais.
- Garder les noms actuels en francais autant que possible.
- Ne pas presenter la generation des coups comme le probleme principal actuel.
- Priorite actuelle: construire la couche "partie" avant de faire une IA plus avancee.
- Ne pas oublier que mat, pat, regle des 50 coups et repetition trois fois ne sont pas encore geres comme etats de partie complets.

## Etat actuel

Le projet est un crate Rust nomme `moteur_ia`.

Le point d'entree actuel charge la position initiale depuis une FEN, initialise les tables d'attaque, puis lance un `perft` profondeur 4.

Le moteur sait deja faire:

- representation du plateau avec bitboards;
- initialisation de la position de depart;
- parser une FEN complete avec pieces, trait, droits de roque, case en passant, halfmove et fullmove;
- generer les coups pseudo-legaux;
- filtrer les coups legaux;
- verifier si un roi est en echec;
- jouer un coup avec `make_move`;
- gerer le roque;
- gerer la promotion;
- gerer la prise en passant;
- convertir des cases et coups en notation lisible;
- lancer des tests `perft`.

Tests importants deja presents:

- position initiale: `perft` profondeurs 1 a 5;
- Kiwipete: `perft` profondeurs 1 a 4;
- parser FEN position initiale;
- FEN avec roque;
- FEN avec prise en passant;
- FEN avec promotion;
- conversions de notation.

## Ce qui manque avant une IA propre

Le moteur sait produire et jouer des coups, mais il ne sait pas encore gerer une partie complete.

Il manque surtout:

- detection d'echec et mat;
- detection de pat;
- detection de nulle par regle des 50 coups;
- detection de nulle par repetition trois fois;
- historique de partie;
- evaluation de position;
- recherche IA.

Ces points sont importants parce qu'une IA doit savoir quand une branche de recherche est gagnee, perdue, nulle ou encore en cours.

## Architecture des fichiers

- `src/main.rs`: point d'entree actuel. Charge une FEN de depart, initialise les tables d'attaque et lance `perft`.
- `src/lib.rs`: expose les modules du moteur.
- `src/board.rs`: contient `CBoard`, `Color`, `Pieces`, les droits de roque et l'initialisation du plateau.
- `src/chess_move.rs`: contient `Move` et `MoveFlag`.
- `src/attack_tables.rs`: prepare les attaques des cavaliers, rois et pions, et calcule les attaques glissantes.
- `src/pseudo_legal_move.rs`: genere les coups selon le mouvement des pieces, sans verifier si le roi reste en securite.
- `src/legal_move.rs`: filtre les coups pseudo-legaux pour garder seulement les coups legaux.
- `src/make_move.rs`: applique un coup sur le plateau.
- `src/legality.rs`: verifie si une case est attaquee et si un roi est en echec.
- `src/perft.rs`: compte les positions legales jusqu'a une profondeur donnee et contient des tests de reference.
- `src/fen.rs`: charge une position depuis une chaine FEN.
- `src/notation.rs`: convertit cases et coups en notation lisible.
- `src/affichage.rs`: contient les fonctions d'affichage et de debug.

## Types centraux

`CBoard` represente une position.

Champs importants:

- `piece_bb`: bitboards des pieces;
- `vide_bb`: cases vides;
- `occupe_bb`: cases occupees;
- `side_to_move`: couleur qui doit jouer;
- `castling_rights`: droits de roque;
- `en_passant_square`: case de prise en passant possible;
- `halfmove_clock`: compteur utile pour la regle des 50 coups;
- `fullmove_number`: numero du coup complet;
- `white_king_square`: case du roi blanc;
- `black_king_square`: case du roi noir.

`Move` represente un coup.

Champs importants:

- `from`: case de depart;
- `to`: case d'arrivee;
- `piece`: piece qui bouge;
- `captured`: piece capturee si capture;
- `promotion`: piece de promotion si promotion;
- `flag`: type de coup.

Flags actuels:

- `Quiet`;
- `Capture`;
- `DoublePawnPush`;
- `EnPassant`;
- `Castling`;
- `Promotion`;
- `PromotionCapture`.

## Mapping des cases

Le moteur utilise le mapping bitboard suivant:

- `0 = a1`;
- `1 = b1`;
- `4 = e1`;
- `7 = h1`;
- `8 = a2`;
- `12 = e2`;
- `28 = e4`;
- `56 = a8`;
- `60 = e8`;
- `63 = h8`.

Les blancs avancent avec `+8`.
Les noirs avancent avec `-8`.

## Commandes utiles

Verifier la compilation:

```bash
cargo check
```

Lancer les tests:

```bash
cargo test
```

Lancer le programme:

```bash
cargo run
```

## References perft

Position initiale:

- profondeur 1 = 20;
- profondeur 2 = 400;
- profondeur 3 = 8902;
- profondeur 4 = 197281;
- profondeur 5 = 4865609;
- profondeur 6 = 119060324.

Kiwipete:

- profondeur 1 = 48;
- profondeur 2 = 2039;
- profondeur 3 = 97862;
- profondeur 4 = 4085603.

Les tests actuels couvrent la position initiale jusqu'a 5 et Kiwipete jusqu'a 4. La profondeur 6 de la position initiale est une reference utile, mais elle est plus lourde.

## Prochaine etape importante

La prochaine grosse etape n'est pas d'ameliorer directement la generation de coups. La prochaine etape est de creer une logique d'etat de partie.

Il faut pouvoir repondre a cette question apres chaque coup:

- la partie continue-t-elle ?
- le joueur au trait est-il mat ?
- le joueur au trait est-il pat ?
- la partie est-elle nulle par 50 coups ?
- la partie est-elle nulle par repetition trois fois ?

Ordre conseille:

1. detecter `EnCours`, `Mat`, `Pat`;
2. ajouter des tests FEN pour mat et pat;
3. mettre a jour correctement `halfmove_clock` dans les coups joues;
4. detecter la regle des 50 coups;
5. creer une structure de partie avec historique;
6. detecter la repetition trois fois;
7. seulement ensuite commencer une IA minimax propre.

## Mat et pat

Mat:

- le joueur au trait n'a aucun coup legal;
- son roi est en echec.

Pat:

- le joueur au trait n'a aucun coup legal;
- son roi n'est pas en echec.

Fonctions deja utiles:

- `generate_legal_move`;
- `is_king_in_check`;
- `side_to_move`.

Ces deux etats doivent etre faits avant une IA serieuse, parce que le score d'une position matee doit etre tres different d'une simple mauvaise position.

## Regle des 50 coups

Le champ `halfmove_clock` existe deja et le parser FEN le charge.

Il faut verifier que ce compteur est mis a jour correctement quand un coup est joue:

- mouvement de pion: retour a 0;
- capture: retour a 0;
- autre coup: +1;
- a partir de 100 demi-coups, la nulle par 50 coups est possible.

Cette regle demande moins d'historique que la repetition, mais elle demande que `make_move` ou la future couche "partie" mette a jour le compteur proprement.

## Repetition trois fois

La repetition demande un historique.

Une position identique doit tenir compte de:

- la position des pieces;
- la couleur au trait;
- les droits de roque;
- la case en passant possible.

Elle ne doit pas tenir compte de:

- `halfmove_clock`;
- `fullmove_number`.

Premiere approche conseillee:

- creer une cle de position simple;
- compter combien de fois cette cle apparait dans l'historique;
- declarer la nulle quand elle apparait 3 fois.

Approche plus avancee:

- utiliser un hash Zobrist;
- reutiliser ce hash pour une future table de transposition de l'IA.

## Route vers l'IA

Quand la couche "partie" est claire, commencer l'IA dans cet ordre:

1. evaluation materielle simple;
2. minimax profondeur 1;
3. minimax profondeur 2 puis 3;
4. prise en compte de `Mat`, `Pat` et nulles dans le score;
5. alpha-beta;
6. ordre des coups;
7. table de transposition;
8. evaluation plus fine.

Valeurs simples pour debuter l'evaluation:

- pion = 100;
- cavalier = 320;
- fou = 330;
- tour = 500;
- dame = 900;
- roi = 0.

## Questions utiles a poser ensuite

- "Aide-moi a detecter mat et pat avec mes fonctions actuelles."
- "Aide-moi a choisir une structure pour representer l'etat de partie."
- "Aide-moi a mettre a jour la regle des 50 coups sans casser perft."
- "Aide-moi a creer une cle de repetition de position."
- "Explique-moi comment commencer une evaluation materielle."
- "Explique-moi minimax avec mon type `Move` et mon `make_move`."
- "Explique-moi alpha-beta apres minimax."
