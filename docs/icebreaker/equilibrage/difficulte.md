# Difficulté : les six difficultés d'une expédition

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

Réglages lus dans assets/icebreaker/difficulte.ron. Ennemis comptés dans de vrais lieux générés (donjon moyen : les cas de l'onglet Expéditions ; grotte et forêt : 6 graines). Les colonnes de droite multiplient les effets, par rapport à la difficulté par défaut ; l'habileté n'y est pas comptée.

## Les six difficultés

| Difficulté | Repère | Nombre d'ennemis | PV des ennemis | Dégâts des ennemis | Habileté | Poids guerrier / archer / mage | Ennemis, donjon moyen | Ennemis, grotte | Ennemis, forêt | Ennemis à affronter | PV à abattre | Dégâts reçus par seconde | Coups pour tuer un ennemi | Coups encaissés avant de mourir |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 |  | 0.7 | 0.6 | 0.7 | base (celle des fiches) | 3 / 1 / 0 | 30.9 | 18.3 | 25.5 | 0.7 | 0.42 | 0.49 | 0.6 | 1.43 |
| 2 |  | 0.85 | 0.8 | 0.85 | base (celle des fiches) | 3 / 2 / 1 | 37.8 | 23.0 | 30.7 | 0.85 | 0.68 | 0.72 | 0.8 | 1.18 |
| 3 | par défaut | 1 | 1 | 1 | base (celle des fiches) | tirage du lieu | 43.5 | 26.7 | 37.2 | 1 | 1 | 1 | 1 | 1 |
| 4 |  | 1.15 | 1.2 | 1.15 | aguerri (rang 3) | tirage du lieu | 49.7 | 29.0 | 42.7 | 1.13 | 1.36 | 1.3 | 1.2 | 0.87 |
| 5 |  | 1.3 | 1.4 | 1.3 | aguerri (rang 3) | 2 / 2 / 2 | 56.7 | 34.2 | 49.2 | 1.3 | 1.83 | 1.7 | 1.4 | 0.77 |
| 6 |  | 1.45 | 1.6 | 1.45 | veteran (rang 5) | 2 / 2 / 3 | 62.9 | 38.7 | 55.7 | 1.46 | 2.34 | 2.12 | 1.6 | 0.69 |
