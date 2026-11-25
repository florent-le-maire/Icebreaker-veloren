# Équilibrage d'Icebreaker : vue d'ensemble

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

## Tuer : touches à l'attaque principale

Cible contre un ennemi ordinaire (pas le chef) : 3 à 3 touches. Dans la cible : 6 cases sur 21 ; moyenne 6.5.

| Jeu d'armes | guerrier | archer | mage | chef | Dégâts par seconde |
|---|---|---|---|---|---|
| Épée et bouclier | 7 | 4 | 3 | 31 | 45.0 |
| Grande épée | 6 | 3 | 3 | 26 | 48.0 |
| Hache à deux mains | 6 | 3 | 3 | 26 | 50.8 |
| Marteau à deux mains | 5 | 3 | 2 | 20 | 38.4 |
| Bâton | 10 | 5 | 5 | 43 | 11.6 |
| Sceptre | 24 | 12 | 11 | 108 | 10.4 |
| Arc | 11 | 6 | 5 | 47 | 16.1 |

## Mourir : touches d'un ennemi, attaque principale

Cible pour un ennemi ordinaire : 3 à 4 touches. Dans la cible : 3 cases sur 9 ; moyenne 5.7.

| Ennemi | guerrier | mage | archer |
|---|---|---|---|
| guerrier | 12 | 4 | 5 |
| archer | 11 | 3 | 5 |
| mage | 6 | 2 | 3 |
| chef | 5 | 2 | 2 |

## L'expédition type

Donjon moyen, menace et densité par défaut : 44 créatures, 2370 PV à abattre, 789 or, 1.1 minutes de frappe du groupe, 3 points d'arme gagnés seul. Premier point d'arme : 6.7 ennemis « guerrier » à tuer seul.

## Les achats

| Achat | Prix (or) | Expéditions type |
|---|---|---|
| Une maison du village | 300 | 0.38 |
| Un jeu d'armes | 150 | 0.19 |
| Un ultime | 200 | 0.25 |
| Un passif | 120 | 0.15 |
| Un soin | 60 | 0.08 |

## Le détail, un fichier par onglet

- [`combat.md`](combat.md) (163 lignes) — Une ligne par compétence et par niveau, lancée avec l'arme de départ du jeu d'armes, comme le fait le jeu. Un « usage » est un geste entier sur une seule cible (une seconde pour un rayon tenu). « Secondes pour tuer » : la compétence une fois, puis l'attaque principale jusqu'au bout ; armure de l'ennemi comprise.
- [`ennemis.md`](ennemis.md) (11 lignes) — Les ennemis frappent avec les attaques de base de leur arme, répétées sans pause. Dégâts sur un joueur en armure de départ ; l'intelligence de l'ennemi (pauses, esquives, choix de l'attaque) n'est pas comptée.
- [`progression.md`](progression.md) (61 lignes) — Jouer un jeu d'armes en expédition donne de la maîtrise (l'expérience des ennemis tués, partagée entre ceux qui les ont blessés) ; la maîtrise donne des points d'arme, dépensés à la salle d'armes.
- [`expeditions.md`](expeditions.md) (38 lignes) — Moyennes sur 20 donjons générés par cas avec le générateur du jeu (objectif « tuer le chef », faction « races »). L'or compte les coffres et ce que lâchent les créatures, pas les caisses du décor ni les objets à revendre.
- [`economie.md`](economie.md) (14 lignes) — Ce qu'un coffre, une créature ou un boss donne en moyenne (toutes les branches de la table, pondérées), et combien il en faut pour chaque achat. L'or moyen se change dans Hypothèses.
- [`hypotheses.md`](hypotheses.md) (67 lignes) — Changer une valeur de la colonne jaune recalcule tout le classeur. La vraie valeur se règle dans le fichier indiqué ; régénérer le classeur (icebreaker.ps1 equilibrage) remet les valeurs du jeu.

Ce que ces chiffres supposent, et comment s'en servir pour régler le jeu : skill `equilibrage` (`.claude/skills/equilibrage/SKILL.md`).
