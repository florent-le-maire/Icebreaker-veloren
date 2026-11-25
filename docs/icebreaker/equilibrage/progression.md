# Progression : points d'arme, niveaux et passifs

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

Jouer un jeu d'armes en expédition donne de la maîtrise (l'expérience des ennemis tués, partagée entre ceux qui les ont blessés) ; la maîtrise donne des points d'arme, dépensés à la salle d'armes.

## Gagner des points d'arme

| Point d'arme n° | Maîtrise pour ce point | Maîtrise cumulée | Ennemis « guerrier » à tuer seul | Ennemis « archer » à tuer seul | Ennemis « mage » à tuer seul | Ennemis « chef » à tuer seul |
|---|---|---|---|---|---|---|
| 1 | 100 | 100 | 6.7 | 8.3 | 8.3 | 2.2 |
| 2 | 150 | 250 | 16.7 | 20.8 | 20.8 | 5.6 |
| 3 | 200 | 450 | 30.0 | 37.5 | 37.5 | 10.0 |
| 4 | 250 | 700 | 46.7 | 58.3 | 58.3 | 15.6 |
| 5 | 300 | 1000 | 66.7 | 83.3 | 83.3 | 22.2 |
| 6 | 350 | 1350 | 90.0 | 112.5 | 112.5 | 30.0 |
| 7 | 400 | 1750 | 116.7 | 145.8 | 145.8 | 38.9 |
| 8 | 450 | 2200 | 146.7 | 183.3 | 183.3 | 48.9 |
| 9 | 500 | 2700 | 180.0 | 225.0 | 225.0 | 60.0 |
| 10 | 550 | 3250 | 216.7 | 270.8 | 270.8 | 72.2 |
| 11 | 600 | 3850 | 256.7 | 320.8 | 320.8 | 85.6 |
| 12 | 650 | 4500 | 300.0 | 375.0 | 375.0 | 100.0 |

## Monter une compétence

| Étape | Points d'arme | Points d'arme cumulés | Maîtrise cumulée, si on ne monte qu'elle |
|---|---|---|---|
| Débloquer (niveau 1) | 1 | 1 | 100 |
| Niveau 2 | 1 | 2 | 250 |
| Niveau 3 | 1 | 3 | 450 |
| Niveau 4 | 2 | 5 | 1000 |
| Niveau 5 | 2 | 7 | 1750 |
| Un jeu d'armes entier (4 compétences) |  | 28 | 21700 |

## Ce qu'un niveau apporte (du niveau 1 au dernier)

| Classe | Jeu d'armes | Compétence | Dernier niveau | Puissance au dernier niveau | Recharge au niveau 1 | Recharge au dernier niveau | Dégâts par usage au niveau 1 | Dégâts par usage au dernier niveau | Gain de dégâts | Dégâts par seconde (soutenu) au niveau 1 | Dégâts par seconde (soutenu) au dernier niveau | Gain en dégâts par seconde |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| guerrier | Épée et bouclier | Riposte | 5 | 1.25 | 8.0 | 6.8 | 32.5 | 40.6 | 25 % | 4.1 | 6.0 | 47 % |
| guerrier | Épée et bouclier | Coup de pommeau | 5 | 1.25 | 7.0 | 6.0 | 30.0 | 37.5 | 25 % | 4.3 | 6.3 | 47 % |
| guerrier | Épée et bouclier | Estoc | 5 | 1.3 | 6.0 | 5.1 | 35.0 | 45.5 | 30 % | 5.8 | 8.9 | 53 % |
| guerrier | Épée et bouclier | Dégagement | 5 | 1 | 9.0 | 6.8 |  |  |  |  |  |  |
| guerrier | Grande épée | Balayage lourd | 5 | 1.15 | 6.0 | 5.4 | 60.0 | 86.2 | 44 % | 10.0 | 16.0 | 60 % |
| guerrier | Grande épée | Tourbillon | 5 | 1.1 | 10.0 | 9.0 | 48.0 | 69.3 | 44 % | 4.8 | 7.7 | 60 % |
| guerrier | Grande épée | Fendeur de terre | 5 | 1.25 | 12.0 | 10.8 | 27.0 | 33.8 | 25 % | 2.2 | 3.1 | 39 % |
| guerrier | Grande épée | Frappe cruelle | 5 | 1.25 | 5.0 | 4.2 | 57.0 | 71.2 | 25 % | 11.4 | 16.8 | 47 % |
| guerrier | Hache à deux mains | Coup brutal | 5 | 1.15 | 6.0 | 5.4 | 54.0 | 82.8 | 53 % | 9.0 | 15.3 | 70 % |
| guerrier | Hache à deux mains | Ratissage | 5 | 1.15 | 8.0 | 7.2 | 48.0 | 55.2 | 15 % | 6.0 | 7.7 | 28 % |
| guerrier | Hache à deux mains | Fracasse-crâne | 5 | 1.2 | 10.0 | 9.0 | 42.0 | 50.4 | 20 % | 4.2 | 5.6 | 33 % |
| guerrier | Hache à deux mains | Ravage | 5 | 1.1 | 9.0 | 8.1 | 81.0 | 118.8 | 47 % | 9.0 | 14.7 | 63 % |
| guerrier | Marteau à deux mains | Revers méprisant | 5 | 1.3 | 8.0 | 6.8 | 69.0 | 89.7 | 30 % | 8.6 | 13.2 | 53 % |
| guerrier | Marteau à deux mains | Moulinet | 5 | 1.3 | 8.0 | 6.8 | 45.0 | 58.5 | 30 % | 5.6 | 8.6 | 53 % |
| guerrier | Marteau à deux mains | Interception | 5 | 1.3 | 10.0 | 8.5 | 60.0 | 78.0 | 30 % | 6.0 | 9.2 | 53 % |
| guerrier | Marteau à deux mains | Tempête de fer | 5 | 1.3 | 14.0 | 11.9 | 120.0 | 234.0 | 95 % | 8.6 | 19.7 | 129 % |
| mage | Bâton | Onde de feu | 5 | 1.5 | 8.0 | 7.2 | 18.0 | 27.0 | 50 % | 2.2 | 3.8 | 67 % |
| mage | Bâton | Napalm | 5 | 1.5 | 14.0 | 12.6 | 9.6 | 21.6 | 125 % | 0.7 | 1.7 | 150 % |
| mage | Bâton | Anneau de feu | 5 | 1.5 | 9.0 | 8.1 | 33.6 | 50.4 | 50 % | 3.7 | 6.2 | 67 % |
| mage | Bâton | Manteau de flammes | 5 | 1.5 | 12.0 | 10.8 |  |  |  |  |  |  |
| mage | Sceptre | Aura de soin | 5 | 1.5 | 12.0 | 10.8 |  |  |  |  |  |  |
| mage | Sceptre | Protection | 5 | 1.5 | 14.0 | 12.6 |  |  |  |  |  |  |
| mage | Sceptre | Rayon de vie | 5 | 1.5 | 8.0 | 7.2 | 10.4 | 15.6 | 50 % | 1.3 | 2.2 | 67 % |
| mage | Sceptre | Cadence | 5 | 1.5 | 15.0 | 13.5 |  |  |  |  |  |  |
| archer | Arc | Trait lourd | 5 | 1.5 | 6.0 | 5.4 | 21.6 | 32.4 | 50 % | 3.6 | 6.0 | 67 % |
| archer | Arc | Vent perçant | 5 | 1.5 | 8.0 | 7.2 | 12.2 | 18.4 | 50 % | 1.5 | 2.5 | 67 % |
| archer | Arc | Pieu d'épines | 5 | 1.5 | 10.0 | 9.0 | 21.6 | 32.4 | 50 % | 2.2 | 3.6 | 67 % |
| archer | Arc | Bond arrière | 5 | 1.5 | 10.0 | 9.0 | 26.3 | 39.4 | 50 % | 2.6 | 4.4 | 67 % |

## Passifs et inné des classes

| Classe | Passif | Effets | Prix (or) |
|---|---|---|---|
| guerrier | Inné (toujours actif) | PV max +25 % ; Déséquilibre reçu -20 % |  |
| guerrier | Robuste | PV max +15 % | 120 |
| guerrier | Peau dure | Dégâts reçus -10 % | 120 |
| guerrier | Sang chaud | Dégâts +10 % | 120 |
| guerrier | Infatigable | Coût des compétences -20 % | 120 |
| mage | Inné (toujours actif) | Endurance max +30 % ; PV max -10 % |  |
| mage | Puissance arcanique | Dégâts +15 % | 120 |
| mage | Flux maîtrisé | Coût des compétences -17 % | 120 |
| mage | Voile de cendre | Dégâts reçus -10 % | 120 |
| mage | Pas de brume | Vitesse +10 % | 120 |
| archer | Inné (toujours actif) | Vitesse +10 % ; Endurance max +15 % |  |
| archer | Tir puissant | Dégâts +10 % | 120 |
| archer | Œil de lynx | Dégâts de précision +20 % | 120 |
| archer | Souffle long | Coût des compétences -17 % | 120 |
| archer | Cuir souple | Dégâts reçus -8 % | 120 |
