# Interface web

## Fonctionnement

Le front est une application React + Vite dans `web/`.
Le point d'entree est `web/src/main.tsx`.
L'etat principal est gere dans `web/src/app.tsx`.
Le front ne valide pas les regles d'echecs.
Le moteur Rust reste la source de verite.
Les appels HTTP sont regroupes dans `web/src/api.ts`.
La liaison Rust/web est dans `src/web_server.rs` et `src/api_type.rs`.
Cette liaison expose l'etat, les coups joueur, les coups IA et la nouvelle partie.
Au demarrage, l'utilisateur choisit blanc ou noir.
Si le joueur choisit blanc, il joue le premier coup.
Si le joueur choisit noir, l'IA joue automatiquement le premier coup.
Apres chaque coup joueur valide, le front demande le coup IA.
Un verrou `isAiThinking` evite les doubles appels.
Le plateau est affiche par `web/src/components/chessBoard.tsx`.
Chaque case est affichee par `web/src/components/square.tsx`.
L'orientation depend du camp choisi.
Blanc en bas affiche `a1` en bas a gauche.
Noir en bas affiche `h8` en bas a gauche.
La couleur des cases est calculee depuis les coordonnees.
La case en bas a gauche est toujours foncee.
Les pieces PNG sont mappees dans `web/src/pieceImages.ts`.
Les images sont servies depuis `/pieces/standard/{piece}.png`.
Le panneau lateral affiche l'etat, les controles et l'historique.
Le message de tour indique si le joueur doit jouer ou si l'IA reflechit.

## Utilisation

Installer les dependances front:
`cd web && npm install`

Lancer le front Vite:
`npm run dev`

Lancer le serveur Rust depuis la racine:
`cargo run`

Arreter un serveur:
`Ctrl+C` dans le terminal concerne.

Les PNG doivent etre dans:
`web/public/pieces/standard/`

Noms attendus:
`wp.png wn.png wb.png wr.png wq.png wk.png`
`bp.png bn.png bb.png br.png bq.png bk.png`

Pour changer de set, remplace ces fichiers en gardant les memes noms.
Pour changer le chemin, modifie `web/src/pieceImages.ts`.
Pour changer les couleurs, modifie `.square-light` et `.square-dark`.
Si l'IA ne joue pas, verifie que le serveur Rust tourne sur `127.0.0.1:3001`.
Verifie aussi que la partie est `EnCours` et que le trait est celui de l'IA.
Si les pieces ne s'affichent pas, cherche une erreur 404 dans le navigateur.
Une 404 indique souvent un mauvais nom ou un mauvais dossier.
Apres modification des PNG, relance `npm run build` si tu sers `web/dist`.
