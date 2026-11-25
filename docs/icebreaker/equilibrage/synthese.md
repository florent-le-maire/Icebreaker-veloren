# Équilibrage d'Icebreaker : vue d'ensemble

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

## Tuer : touches à l'attaque principale

Cible contre un ennemi ordinaire (pas le chef) : 3 à 3 touches. Dans la cible : 6 cases sur 21 ; moyenne 6.5.

| Jeu d'armes | guerrier | archer | mage | chef | Dégâts par seconde |
|---|---|---|---|---|---|
| Épée et bouclier | 7 | 4 | 3 | 31 | 31.6 |
| Grande épée | 6 | 3 | 3 | 26 | 34.8 |
| Hache à deux mains | 6 | 3 | 3 | 26 | 37.3 |
| Marteau à deux mains | 5 | 3 | 2 | 20 | 29.7 |
| Bâton | 10 | 5 | 5 | 43 | 10.2 |
| Sceptre | 24 | 12 | 11 | 108 | 10.4 |
| Arc | 11 | 6 | 5 | 47 | 13.3 |

## Mourir : touches d'un ennemi, attaque principale

Cible pour un ennemi ordinaire : 3 à 4 touches. Dans la cible : 3 cases sur 9 ; moyenne 5.7.

| Ennemi | guerrier | mage | archer |
|---|---|---|---|
| guerrier | 12 | 4 | 5 |
| archer | 11 | 3 | 5 |
| mage | 6 | 2 | 3 |
| chef | 5 | 2 | 2 |

## L'expédition type

Donjon moyen, difficulté et densité par défaut : 44 créatures, 2100 PV à abattre, 789 or, 1.3 minutes de frappe du groupe, 3 points d'arme gagnés seul. Premier point d'arme : 6.7 ennemis « guerrier » à tuer seul.

## Les achats

| Achat | Prix (or) | Expéditions type |
|---|---|---|
| Une maison du village | 300 | 0.38 |
| Un jeu d'armes | 150 | 0.19 |
| Un ultime | 200 | 0.25 |
| Un passif | 120 | 0.15 |
| Un soin | 60 | 0.08 |

## Les six difficultés

Réglages de `assets/icebreaker/difficulte.ron` ; ennemis comptés dans de vrais lieux générés ; les deux dernières colonnes multiplient les effets, par rapport à la difficulté par défaut (l'habileté n'y est pas comptée).

| Difficulté | Nombre | PV | Dégâts | Habileté | Guerrier / archer / mage | Ennemis (donjon moyen, grotte, forêt) | PV à abattre | Dégâts reçus par seconde |
|---|---|---|---|---|---|---|---|---|
| 1 | × 0.70 | × 0.60 | × 0.35 (mage × 0.60) | base (celle des fiches) | 3 / 1 / 0 | 31, 18, 26 | × 0.42 | × 0.24 |
| 2 | × 0.85 | × 0.80 | × 0.65 (mage × 0.80) | base (celle des fiches) | 3 / 2 / 1 | 38, 23, 31 | × 0.68 | × 0.55 |
| 3 (par défaut) | × 1.00 | × 1.00 | × 1.00 | base (celle des fiches) | tirage du lieu | 44, 27, 37 | × 1.00 | × 1.00 |
| 4 | × 1.15 | × 1.20 | × 1.15 | aguerri (rang 3) | tirage du lieu | 50, 29, 43 | × 1.36 | × 1.30 |
| 5 | × 1.30 | × 1.40 | × 1.30 | aguerri (rang 3) | 2 / 2 / 2 | 57, 34, 49 | × 1.83 | × 1.70 |
| 6 | × 1.45 | × 1.60 | × 1.45 | veteran (rang 5) | 2 / 2 / 3 | 63, 39, 56 | × 2.34 | × 2.12 |

## Le détail, un fichier par onglet

- [`combat.md`](combat.md) (163 lignes) — Une ligne par compétence et par niveau, lancée avec l'arme de départ du jeu d'armes, comme le fait le jeu. Un « usage » est un geste entier sur une seule cible (une seconde pour un rayon tenu). « Secondes pour tuer » : la compétence une fois, puis l'attaque principale jusqu'au bout ; armure de l'ennemi comprise.
- [`ennemis.md`](ennemis.md) (11 lignes) — Les ennemis frappent avec les attaques de base de leur arme, répétées sans pause. Dégâts sur un joueur en armure de départ ; l'intelligence de l'ennemi (pauses, esquives, choix de l'attaque) n'est pas comptée.
- [`progression.md`](progression.md) (61 lignes) — Jouer un jeu d'armes en expédition donne de la maîtrise (l'expérience des ennemis tués, partagée entre ceux qui les ont blessés) ; la maîtrise donne des points d'arme, dépensés à la salle d'armes.
- [`expeditions.md`](expeditions.md) (41 lignes) — Moyennes sur 20 donjons générés par cas avec le générateur du jeu (objectif « tuer le chef », faction « races »). L'or compte les coffres et ce que lâchent les créatures, pas les caisses du décor ni les objets à revendre.
- [`economie.md`](economie.md) (14 lignes) — Ce qu'un coffre, une créature ou un boss donne en moyenne (toutes les branches de la table, pondérées), et combien il en faut pour chaque achat. L'or moyen se change dans Hypothèses.
- [`difficulte.md`](difficulte.md) (6 lignes) — Réglages lus dans assets/icebreaker/difficulte.ron. Ennemis comptés dans de vrais lieux générés (donjon moyen : les cas de l'onglet Expéditions ; grotte et forêt : 6 graines). Les colonnes de droite multiplient les effets, par rapport à la difficulté par défaut ; l'habileté n'y est pas comptée.
- [`hypotheses.md`](hypotheses.md) (67 lignes) — Changer une valeur de la colonne jaune recalcule tout le classeur. La vraie valeur se règle dans le fichier indiqué ; régénérer le classeur (icebreaker.ps1 equilibrage) remet les valeurs du jeu.

Ce que ces chiffres supposent, et comment s'en servir pour régler le jeu : skill `equilibrage` (`.claude/skills/equilibrage/SKILL.md`).
