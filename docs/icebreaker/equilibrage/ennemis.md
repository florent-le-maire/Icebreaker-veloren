# Ennemis : ce qu'ils valent face à chaque classe

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

Les ennemis frappent avec les attaques de base de leur arme, répétées sans pause. Dégâts sur un joueur en armure de départ ; l'intelligence de l'ennemi (pauses, esquives, choix de l'attaque) n'est pas comptée.

| Ennemi | PV | Armure (dégâts retirés) | Attaque | Capacité (fichier) | Type d'attaque | Touches par usage | Dégâts de base (usage) | Puissance de l'arme | Contre un joueur | Dégâts par usage | Geste | Dégâts par seconde | Contre : guerrier — Touches pour tuer | Contre : guerrier — Secondes pour tuer | Contre : mage — Touches pour tuer | Contre : mage — Secondes pour tuer | Contre : archer — Touches pour tuer | Contre : archer — Secondes pour tuer | Ce que le calcul suppose |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| guerrier | 50 | 50 % | Attaque principale | sword.basic_double_slash (ComboMelee2) | Corps à corps | 2.0 | 12.0 | 0.6 | 9 | 64.8 | 1.0 | 62.7 | 12 | 6.2 | 4 | 2.1 | 5 | 2.6 | Enchaînement entier, touche tenue. |
| guerrier | 50 | 50 % | Attaque secondaire | sword.basic_thrust (ChargedMelee) | Corps à corps | 1.0 | 15.0 | 0.6 | 9 | 81.0 | 1.2 | 68.5 | 5 | 5.9 | 2 | 2.4 | 2 | 2.4 | À pleine charge ; sans charger : 0 de base. |
| archer | 40 | 21 % | Attaque principale | bow.arrow_shot (BasicRanged) | Projectile | 1.0 | 7.0 | 0.6 | 9 | 37.8 | 0.8 | 49.8 | 11 | 8.3 | 3 | 2.3 | 5 | 3.8 |  |
| archer | 40 | 21 % | Attaque secondaire | bow.broadhead (ChargedRanged) | Projectile | 1.0 | 25.0 | 0.6 | 9 | 135.0 | 2.8 | 48.1 | 3 | 8.4 | 1 | 2.3 | 2 | 5.6 | À pleine charge ; sans charger : 0 de base. |
| mage | 40 | 10 % | Attaque principale | staff.firebomb (BasicRanged) | Explosion | 1.0 | 13.0 | 0.6 | 9 | 70.2 | 1.1 | 65.6 | 6 | 6.4 | 2 | 2.1 | 3 | 3.2 | Non compté : Burning (10 % des dégâts en plus sur 5 s, 10 % de chances). |
| mage | 40 | 10 % | Attaque secondaire | staff.flamethrower (BasicBeam) | Rayon | 2.4 | 12.0 | 0.6 | 9 | 64.8 | 1.0 | 64.8 | 15 | 6.4 | 5 | 2.2 | 6 | 2.6 | Tenu : compté par seconde de rayon. Non compté : Burning (100 % des dégâts en plus sur 10 s, 25 % de chances). |
| chef | 150 | 68 % | Attaque principale | gnarling.chieftain.flamestrike (BasicMelee) | Corps à corps | 1.0 | 9.0 | 1 | 9 | 81.0 | 2.3 | 34.7 | 5 | 11.7 | 2 | 4.7 | 2 | 4.7 | Non compté : Burning (50 % des dégâts en plus sur 4 s, 50 % de chances). |
| chef | 150 | 68 % | Attaque secondaire | gnarling.chieftain.firebarrage (BasicRanged) | Explosion | 1.0 | 5.5 | 1 | 9 | 49.5 | 1.7 | 28.6 | 8 | 13.9 | 3 | 5.2 | 4 | 6.9 | 5 projectiles en éventail ; un seul compté sur la cible. Non compté : Burning (30 % des dégâts en plus sur 3 s, 10 % de chances). |

## Les classes en face

| Classe | PV de base | Inné (PV) | PV | Armure de départ (dégâts retirés) |
|---|---|---|---|---|
| guerrier | 100 | 1.25 | 125 | 68 % |
| mage | 100 | 0.9 | 90 | 17 % |
| archer | 100 | 1 | 100 | 35 % |
