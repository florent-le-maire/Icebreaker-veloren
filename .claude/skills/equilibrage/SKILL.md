---
name: equilibrage
description: Équilibrer Icebreaker avec des chiffres — ennemis trop durs ou trop mous, compétence ou jeu d'armes trop fort ou trop faible, classe qui meurt trop vite, or, prix, butin ou progression à régler, taille d'une expédition. Mesure avant et après avec le classeur d'équilibrage (crate icebreaker-equilibrage, docs/icebreaker/equilibrage.xlsx), dit quel fichier de assets/ changer, et vérifie. À utiliser dès que l'équipe parle d'équilibrage, de difficulté, de dégâts, de PV, de « temps pour tuer », de prix, ou demande de mettre à jour ou d'étendre le classeur.
---

# Équilibrer Icebreaker avec le classeur

Le classeur (`docs/icebreaker/equilibrage.xlsx`, jalon EQ, `docs/icebreaker/PLAN-PLAYTEST.md` § 8) relit les
chiffres du jeu avec les structures du jeu et calcule ce qu'ils donnent : touches et secondes pour tuer, pour mourir,
or d'une expédition, coût des achats, progression. Il ne change rien au jeu : **les fichiers de `assets/` restent la
source**. La boucle est toujours la même : mesurer, changer un fichier, remesurer, comparer à la cible, faire
vérifier en jeu.

Travailler dans le worktree `Veloren-equilibrage` (le créer s'il manque : `CLAUDE.md`, « Travailler à plusieurs
agents »). Cargo n'est pas dans le PATH des outils : `export PATH="$HOME/.cargo/bin:$PATH"`.

## 1. Mesurer

```
cargo run -q -p icebreaker-equilibrage -- --resume            # matrices, expédition type, achats
cargo run -q -p icebreaker-equilibrage -- --resume fournaise  # + les lignes de Combat dont le nom contient « fournaise »
cargo run -q -p icebreaker-equilibrage -- --resume baton      # + toutes les lignes d'un jeu d'armes (son id)
```

Chaque lancement réécrit aussi le classeur. Le résumé donne :
- **Tuer** : touches à l'attaque principale de chaque jeu d'armes contre chaque ennemi ;
- **Mourir** : touches de chaque ennemi (attaque principale) pour tuer chaque classe ;
- **l'expédition type** (donjon moyen, menace et densité par défaut) et les **achats** comptés en expéditions.

Une ligne « lecture partielle » en sortie veut dire qu'un chiffre n'a pas pu être lu ou que la formule du classeur ne
redonne pas les dégâts du jeu : la traiter avant de conclure quoi que ce soit (§ 5).

Pour un détail que le résumé ne montre pas (énergie, geste, recharge, niveaux, butin par table, cas de donjon), lire
le classeur : `py -c "import openpyxl; f = openpyxl.load_workbook('docs/icebreaker/equilibrage.xlsx',
data_only=True)['Combat']; ..."`. Les résultats des formules sont écrits dans le fichier, pas besoin de tableur (il
n'y a ni Excel ni LibreOffice sur le poste de Flo).

## 2. Ce que les chiffres supposent

À dire à l'équipe chaque fois qu'on avance un chiffre, parce qu'un joueur ne verra pas exactement ça :
- arme et armure **de départ**, sans passif ; tous les coups portent, une seule cible, immobile ;
- **pas d'altérations** : la brûlure et le saignement ne sont pas comptés (la colonne Notes de Combat dit ce que la
  compétence en porte, et ça peut doubler ses dégâts) ;
- charge et élan à fond, plongeon frappé depuis le sol, riposte réussie ; un seul projectile d'un éventail ;
- les ennemis répètent l'attaque de base de leur arme sans pause : leur IA n'est pas comptée ;
- « Secondes pour tuer » d'une compétence : la compétence une fois, puis l'attaque principale jusqu'au bout ;
- expéditions : moyennes de 20 donjons générés par cas, objectif « tuer le chef », faction par défaut.

Le classeur donne des **ordres de grandeur et des comparaisons** (ce jeu d'armes vaut la moitié de celui-là), pas
des mesures. Un écart avec ce que l'équipe voit en jeu est une information : chercher d'abord l'erreur dans le
classeur (§ 5), pas dans le joueur.

## 3. Trouver le bon levier

Un réglage **global** si toutes les lignes sont décalées dans le même sens, un réglage **local** si une seule
dépasse. L'onglet Hypothèses donne, pour chaque chiffre, le fichier où il se règle.

| Ce qu'on veut changer | Fichier | Ce que ça entraîne |
|---|---|---|
| Tous les dégâts des joueurs sur les créatures | `assets/icebreaker/combat.ron` (`degats_contre_creatures`, puis un par type d'attaque) | tout le jeu d'un coup, proportions gardées |
| Tous les dégâts reçus par les joueurs | `combat.ron` (`degats_contre_joueurs`) | idem, côté ennemis |
| Un ennemi : ses PV | `assets/icebreaker/ennemis/<role>.ron` (`icebreaker_pv`) | touches pour le tuer, PV d'une expédition, expérience |
| Un ennemi : son armure | pièces d'armure de son `loadout`, même fichier | le jeu retire protection / (60 + protection) : le plastron et les jambières de cuir cloué retirent déjà 50 % |
| Un ennemi : ses dégâts | son arme (`active_hands`) | attention : l'arme de départ est la même que celle des joueurs |
| Une classe : ses PV | `assets/icebreaker/passifs/<classe>.ron` (`inne`, `MaxHealthModifier`) | touches pour la tuer |
| Une classe : son armure | `assets/icebreaker/classes/<classe>.ron` | le guerrier retire 68 %, le mage 17 % : l'écart entre classes vient surtout de là |
| Une compétence : dégâts et recharge par niveau | `assets/icebreaker/jeux_armes/<jeu>.ron`, `ultimes/<classe>.ron` (`puissance`, `recharge`) | la ligne de la compétence |
| Une compétence : dégâts de base, durées, énergie | sa capacité : `assets/icebreaker/abilities/…` | idem |
| Une compétence au niveau 1 qui pointe vers Veloren (`common.abilities.…`) | la **copier** dans `assets/icebreaker/abilities/` et pointer dessus | on ne retouche pas les fichiers de Veloren (nos changements restent regroupés) |
| La puissance d'une arme de départ | fichier de l'objet (`assets/common/items/weapons/…`) | change aussi les ennemis qui la portent : préférer `combat.ron` ou la compétence |
| L'or et les objets d'un butin | `assets/common/loot_tables/icebreaker/butin/*.ron` (quelle table pour quoi : `assets/icebreaker/butin.ron`) | or d'une expédition, achats en expéditions |
| Les prix | `assets/icebreaker/salle_armes.ron`, `village.ron` ; le soin : `server/src/settings/mod.rs` (`heal_price`) | achats en expéditions |
| La progression | `assets/icebreaker/salle_armes.ron` (`premier_point`, `pas_point`, `cout_niveaux`) | onglet Progression |
| Le nombre d'ennemis et de coffres d'un donjon | `assets/icebreaker/donjons/generation.ron` (`population`, `parameters`) | onglet Expéditions |
| Les cibles (3 touches pour tuer, 3 à 4 pour mourir) | `icebreaker-equilibrage/src/combat.rs`, fin de `hypotheses` ; règle écrite dans `docs/icebreaker/PLAN.md`, L.1 | couleurs de la Synthèse |

Changer une cible, c'est changer une règle de jeu : demander à l'équipe d'abord (`CLAUDE.md`, « Quand demander à
l'équipe »). Même chose pour un choix entre deux leviers qui ne donnent pas le même jeu (baisser l'armure des
ennemis ou monter les dégâts des joueurs).

## 4. Changer, remesurer, livrer

1. Noter la mesure d'avant (copier les lignes utiles du résumé).
2. Changer **un** levier dans son fichier. Calculer d'avance ce qu'il faut : les dégâts sont un produit (base ×
   niveau × arme × multiplicateur de `combat.ron`), l'armure retire sa part ensuite ; touches = PV / (dégâts d'une
   touche × (1 − armure)), arrondi au-dessus. Viser le milieu de la marche, pas le bord : 49,9 PV et 50,1 PV ne
   donnent pas le même nombre de touches.
3. Relancer avec `--resume` : la mesure d'après. Regarder aussi ce qui n'était **pas** visé (un multiplicateur
   global déplace toutes les lignes ; les PV d'un ennemi déplacent l'expérience et les minutes d'une expédition).
4. Présenter à l'équipe un tableau avant / après / cible, avec ce que le calcul suppose (§ 2).
5. `cargo test -p icebreaker-equilibrage`, puis un commit par réglage, en français, avec le fichier de données **et**
   le classeur régénéré (il est reproductible : mêmes données, même fichier). Le changement n'est pas vérifié en
   jeu : l'ajouter à « À vérifier en jeu » de `docs/icebreaker/PLAN.md`.
6. Proposer l'essai : `.\icebreaker.ps1 entrainement` (port 14150), puis `/entrainement classe`, `arme`, `ennemi`…
   pour compter les coups sur le vrai ennemi. Si le serveur tournait déjà, il est périmé : le relancer.

## 5. Quand le classeur a tort

Le cas vécu (EQ.6) : la Fournaise affichait 227 s pour tuer un guerrier, contre quelques secondes en jeu. La lecture
des capacités devinait d'après le nom des champs (`beam_duration` pris pour la durée du rayon). Depuis, chaque
capacité est lancée comme le jeu (`adjusted_by_stats`, stats de l'arme × niveau) puis lue **sorte par sorte**
dans `icebreaker-equilibrage/src/capacite.rs`.

Quand un chiffre surprend, ou quand le jeu gagne une nouvelle sorte de capacité ou un champ `icebreaker_*` :
1. Ouvrir l'état de combat de la sorte : `common/src/states/<sorte>.rs`. Y lire combien de fois elle frappe, ce qui
   la fait durer (touche tenue, durée fixe, charge), ce qu'elle coûte. Ne pas déduire du nom d'un champ.
2. Voir ce que l'arme change : `adjusted_by_stats` dans `common/src/comp/ability.rs` (la vitesse divise les
   durées, multiplie le rythme d'un rayon ; la puissance multiplie tous les dégâts).
3. Corriger l'arme du `match` dans `Capacite::lancee` : composantes (type d'attaque, touches, dégâts), énergie,
   geste, `avant` (temps avant la première touche), `apres`, `tenu`. Mettre dans `remarques` ce qu'on suppose.
4. Écrire un test dans `capacite.rs` sur la capacité réelle, avec les nombres attendus calculés à la main depuis
   son fichier.
5. Le type d'attaque décide du multiplicateur de `combat.ron` : un projectile explosif frappe comme un projectile
   ou comme une explosion selon sa cible (`ProjectileExplosionTarget`), à lire dans `common/src/comp/projectile.rs`.

Le témoin de `Donnees::lire` compare pour chaque ligne la formule du classeur (base × niveau × arme) aux dégâts que
le jeu lance ; un écart sort en « lecture partielle ».

## 6. Changer le classeur lui-même

- Un module par onglet dans `icebreaker-equilibrage/src/` ; `style.rs` pour l'allure (`tableau`, `bandeau`, `Val`).
- Tout chiffre du jeu qui entre dans un calcul est une case de l'onglet Hypothèses (`hyp.ajoute`, avec le fichier
  où il se règle) ; les formules pointent dessus (`hyp.r`), et portent leur résultat calculé en Rust (`Val::F`).
  Le calcul vit dans `calcul.rs`, testé, et la formule Excel doit dire la même chose.
- Après un changement de formule : recalculer le classeur avec un moteur à part et comparer aux résultats écrits
  (Python `formulas` dans un venv jetable : `ExcelModel().loads(x).finish().calculate()`, puis comparer cellule par
  cellule à `openpyxl.load_workbook(x, data_only=True)`). Zéro erreur, zéro écart avant de committer : ce contrôle
  a déjà attrapé un décalage d'une ligne.
- Les graphiques et les couleurs conditionnelles ne se voient pas sans tableur : le dire à l'équipe, qui regarde.
- `rust_xlsxwriter` : un résultat vide (`set_result("")`) s'écrit 0, utiliser un tiret ; un chemin Windows ou une
  barre inverse ne s'écrit pas par heredoc dans l'outil Bash (elle disparaît), passer par l'outil d'écriture.
