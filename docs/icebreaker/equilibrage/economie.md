# Économie : le butin face aux prix

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

Ce qu'un coffre, une créature ou un boss donne en moyenne (toutes les branches de la table, pondérées), et combien il en faut pour chaque achat. L'or moyen se change dans Hypothèses.

## Ce que donne chaque butin, en moyenne

| Butin | Or au moins | Or moyen | Or au plus | Potions et bombes | Nourriture | Armes | Armures | Minerais et matériaux | Autres | Objets en tout |
|---|---|---|---|---|---|---|---|---|---|---|
| Coffre, palier 1 | 10 | 20.0 | 30 | 0.41 | 0.27 | 0.09 | 0.09 | 1.64 |  | 2.5 |
| Coffre, palier 2 | 30 | 55.0 | 80 | 0.56 |  | 0.1 | 0.1 | 1.13 |  | 1.89 |
| Coffre, palier 3 | 80 | 140.0 | 200 | 0.69 |  | 0.15 | 0.15 | 0.58 |  | 1.57 |
| Créature, palier 1 | 0 | 1.2 | 10 | 0.09 |  | 0.03 | 0.03 | 0.19 |  | 0.34 |
| Créature, palier 2 | 0 | 2.5 | 20 | 0.14 |  | 0.03 | 0.03 | 0.27 |  | 0.47 |
| Créature, palier 3 | 0 | 6.5 | 40 | 0.16 |  | 0.04 | 0.04 | 0.14 |  | 0.38 |
| Chef | 310 | 505.0 | 700 | 1.38 |  | 0.29 | 1.29 | 1.17 |  | 4.13 |
| Boss | 440 | 720.0 | 1000 | 2.06 |  | 0.94 | 0.94 | 1.75 |  | 5.69 |
| Caisse du décor | 0 | 1.9 | 10 | 0.1 | 0.2 |  |  | 1.2 |  | 1.5 |

## Combien de butins pour s'offrir…

| Achat | Prix (or) | Expéditions type | Coffre, palier 1 | Coffre, palier 2 | Coffre, palier 3 | Créature, palier 1 | Créature, palier 2 | Créature, palier 3 | Chef | Boss | Caisse du décor |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Une maison du village | 300 | 0.4 | 15.0 | 5.5 | 2.1 | 243.9 | 119.0 | 45.8 | 0.6 | 0.4 | 153.8 |
| Un jeu d'armes | 150 | 0.2 | 7.5 | 2.7 | 1.1 | 122.0 | 59.5 | 22.9 | 0.3 | 0.2 | 76.9 |
| Un ultime | 200 | 0.3 | 10.0 | 3.6 | 1.4 | 162.6 | 79.4 | 30.5 | 0.4 | 0.3 | 102.6 |
| Un passif | 120 | 0.2 | 6.0 | 2.2 | 0.9 | 97.6 | 47.6 | 18.3 | 0.2 | 0.2 | 61.5 |
| Un soin | 60 | 0.1 | 3.0 | 1.1 | 0.4 | 48.8 | 23.8 | 9.2 | 0.1 | 0.1 | 30.8 |
