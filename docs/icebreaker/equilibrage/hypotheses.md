# Hypothèses : les cases « et si »

> Fichier généré par `cargo run -p icebreaker-equilibrage` (ou `.\icebreaker.ps1 equilibrage`) depuis les fichiers du jeu : ne pas le modifier à la main. Mêmes chiffres que `docs/icebreaker/equilibrage.xlsx`.

Changer une valeur de la colonne jaune recalcule tout le classeur. La vraie valeur se règle dans le fichier indiqué ; régénérer le classeur (icebreaker.ps1 equilibrage) remet les valeurs du jeu.

## Dégâts des attaques (multiplicateurs)

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Contre un joueur (toute attaque) | 9 | assets/icebreaker/combat.ron : degats_contre_joueurs |
| Contre une créature : corps à corps | 5 | assets/icebreaker/combat.ron : degats_contre_creatures |
| Contre une créature : projectile (flèche, sort lancé) | 2.4 | assets/icebreaker/combat.ron : creatures_projectile |
| Contre une créature : rayon, à chaque touche | 1.8 | assets/icebreaker/combat.ron : creatures_rayon |
| Contre une créature : explosion | 1.4 | assets/icebreaker/combat.ron : creatures_explosion |
| Contre une créature : onde de choc | 1.5 | assets/icebreaker/combat.ron : creatures_onde |
| Contre une créature : arc électrique, zone persistante | 2 | assets/icebreaker/combat.ron : creatures_autre |

## Classes des joueurs

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| guerrier : PV de base | 100 | PV d'un humanoïde dans Veloren (common/src/comp/body/mod.rs, base_health) |
| guerrier : PV de l'inné | 1.25 | assets/icebreaker/passifs/guerrier.ron : inne, MaxHealthModifier |
| guerrier : part des dégâts retirée par l'armure de départ | 68 % | assets/icebreaker/classes/guerrier.ron (pièces d'armure) ; calcul du jeu : protection / (60 + protection) |
| mage : PV de base | 100 | PV d'un humanoïde dans Veloren (common/src/comp/body/mod.rs, base_health) |
| mage : PV de l'inné | 0.9 | assets/icebreaker/passifs/mage.ron : inne, MaxHealthModifier |
| mage : part des dégâts retirée par l'armure de départ | 17 % | assets/icebreaker/classes/mage.ron (pièces d'armure) ; calcul du jeu : protection / (60 + protection) |
| archer : PV de base | 100 | PV d'un humanoïde dans Veloren (common/src/comp/body/mod.rs, base_health) |
| archer : PV de l'inné | 1 | assets/icebreaker/passifs/archer.ron : inne, MaxHealthModifier |
| archer : part des dégâts retirée par l'armure de départ | 35 % | assets/icebreaker/classes/archer.ron (pièces d'armure) ; calcul du jeu : protection / (60 + protection) |

## Armes de départ des jeux d'armes

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Épée et bouclier : puissance de l'arme | 0.5 | assets/common/items/weapons/sword_1h/starter.ron : power (arme choisie dans assets/icebreaker/jeux_armes/epee_bouclier.ron) |
| Grande épée : puissance de l'arme | 0.6 | assets/common/items/weapons/sword/starter.ron : power (arme choisie dans assets/icebreaker/jeux_armes/grande_epee.ron) |
| Hache à deux mains : puissance de l'arme | 0.6 | assets/common/items/weapons/axe/starter_axe.ron : power (arme choisie dans assets/icebreaker/jeux_armes/hache.ron) |
| Marteau à deux mains : puissance de l'arme | 0.6 | assets/common/items/weapons/hammer/starter_hammer.ron : power (arme choisie dans assets/icebreaker/jeux_armes/marteau.ron) |
| Bâton : puissance de l'arme | 0.6 | assets/common/items/weapons/staff/starter_staff.ron : power (arme choisie dans assets/icebreaker/jeux_armes/baton.ron) |
| Sceptre : puissance de l'arme | 0.6 | assets/common/items/weapons/sceptre/starter_sceptre.ron : power (arme choisie dans assets/icebreaker/jeux_armes/sceptre.ron) |
| Arc : puissance de l'arme | 0.6 | assets/common/items/weapons/bow/starter.ron : power (arme choisie dans assets/icebreaker/jeux_armes/arc.ron) |

## Ennemis

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| guerrier : PV | 50 | assets/icebreaker/ennemis/guerrier.ron : icebreaker_pv |
| guerrier : part des dégâts retirée par son armure | 50 % | assets/icebreaker/ennemis/guerrier.ron : pièces d'armure du loadout |
| guerrier : puissance de son arme | 0.6 | assets/common/items/weapons/sword/starter.ron : power |
| guerrier : expérience donnée à sa mort | 15 | calcul de Veloren : note de combat × 20 (common/src/combat.rs, combat_rating), d'après ses PV, son armure et son arme |
| archer : PV | 40 | assets/icebreaker/ennemis/archer.ron : icebreaker_pv |
| archer : part des dégâts retirée par son armure | 21 % | assets/icebreaker/ennemis/archer.ron : pièces d'armure du loadout |
| archer : puissance de son arme | 0.6 | assets/common/items/weapons/bow/starter.ron : power |
| archer : expérience donnée à sa mort | 12 | calcul de Veloren : note de combat × 20 (common/src/combat.rs, combat_rating), d'après ses PV, son armure et son arme |
| mage : PV | 40 | assets/icebreaker/ennemis/mage.ron : icebreaker_pv |
| mage : part des dégâts retirée par son armure | 10 % | assets/icebreaker/ennemis/mage.ron : pièces d'armure du loadout |
| mage : puissance de son arme | 0.6 | assets/common/items/weapons/staff/starter_staff.ron : power |
| mage : expérience donnée à sa mort | 12 | calcul de Veloren : note de combat × 20 (common/src/combat.rs, combat_rating), d'après ses PV, son armure et son arme |
| chef : PV | 150 | assets/icebreaker/ennemis/chef.ron : icebreaker_pv |
| chef : part des dégâts retirée par son armure | 68 % | assets/icebreaker/ennemis/chef.ron : pièces d'armure du loadout |
| chef : puissance de son arme | 1 | assets/common/items/npc_weapons/biped_small/gnarling/chieftain.ron : power |
| chef : expérience donnée à sa mort | 45 | calcul de Veloren : note de combat × 20 (common/src/combat.rs, combat_rating), d'après ses PV, son armure et son arme |

## Cibles d'équilibrage

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Coups de base pour tuer un ennemi ordinaire : au moins | 3 | docs/icebreaker/PLAN.md, L.1 (cible : 3 coups pour tuer, 3 à 4 pour mourir) |
| Coups de base pour tuer un ennemi ordinaire : au plus | 3 | docs/icebreaker/PLAN.md, L.1 (cible : 3 coups pour tuer, 3 à 4 pour mourir) |
| Coups d'un ennemi ordinaire pour tuer un joueur : au moins | 3 | docs/icebreaker/PLAN.md, L.1 (cible : 3 coups pour tuer, 3 à 4 pour mourir) |
| Coups d'un ennemi ordinaire pour tuer un joueur : au plus | 4 | docs/icebreaker/PLAN.md, L.1 (cible : 3 coups pour tuer, 3 à 4 pour mourir) |

## Progression (salle d'armes)

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Maîtrise pour le premier point d'arme | 100 | assets/icebreaker/salle_armes.ron : premier_point |
| Maîtrise en plus pour chaque point suivant | 50 | assets/icebreaker/salle_armes.ron : pas_point |
| Maîtrise gagnée par point d'expérience | 1 | assets/icebreaker/salle_armes.ron : maitrise_par_experience |
| Points d'arme pour débloquer une compétence | 1 | assets/icebreaker/salle_armes.ron : cout_debloquer |
| Points d'arme pour passer au niveau 2 | 1 | assets/icebreaker/salle_armes.ron : cout_niveaux |
| Points d'arme pour passer au niveau 3 | 1 | assets/icebreaker/salle_armes.ron : cout_niveaux |
| Points d'arme pour passer au niveau 4 | 2 | assets/icebreaker/salle_armes.ron : cout_niveaux |
| Points d'arme pour passer au niveau 5 | 2 | assets/icebreaker/salle_armes.ron : cout_niveaux |

## Prix (or)

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Une maison du village | 300 | assets/icebreaker/village.ron : prix_maison |
| Apprendre un jeu d'armes | 150 | assets/icebreaker/salle_armes.ron : prix_jeu_armes |
| Apprendre un ultime | 200 | assets/icebreaker/salle_armes.ron : prix_ultime |
| Apprendre un passif | 120 | assets/icebreaker/salle_armes.ron : prix_passif |
| Un soin au village | 60 | server/src/settings/mod.rs : heal_price (réglage du serveur, pas encore dans assets/) |

## Or moyen par butin

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Coffre, palier 1 : or moyen | 20.0 | assets/common/loot_tables/icebreaker/butin/coffre_1.ron (nommée dans assets/icebreaker/butin.ron) |
| Coffre, palier 2 : or moyen | 55.0 | assets/common/loot_tables/icebreaker/butin/coffre_2.ron (nommée dans assets/icebreaker/butin.ron) |
| Coffre, palier 3 : or moyen | 140.0 | assets/common/loot_tables/icebreaker/butin/coffre_3.ron (nommée dans assets/icebreaker/butin.ron) |
| Créature, palier 1 : or moyen | 1.2 | assets/common/loot_tables/icebreaker/butin/creature_1.ron (nommée dans assets/icebreaker/butin.ron) |
| Créature, palier 2 : or moyen | 2.5 | assets/common/loot_tables/icebreaker/butin/creature_2.ron (nommée dans assets/icebreaker/butin.ron) |
| Créature, palier 3 : or moyen | 6.5 | assets/common/loot_tables/icebreaker/butin/creature_3.ron (nommée dans assets/icebreaker/butin.ron) |
| Chef : or moyen | 505.0 | assets/common/loot_tables/icebreaker/butin/chef.ron (nommée dans assets/icebreaker/butin.ron) |
| Boss : or moyen | 720.0 | assets/common/loot_tables/icebreaker/butin/boss.ron (nommée dans assets/icebreaker/butin.ron) |
| Caisse du décor : or moyen | 1.9 | assets/common/loot_tables/icebreaker/butin/caisse.ron (nommée dans assets/icebreaker/butin.ron) |

## Repères d'une expédition

| Réglage | Valeur du jeu | Où la régler |
|---|---|---|
| Joueurs dans une expédition | 2 | hypothèse : la campagne se joue à deux (docs/icebreaker/PLAN-PLAYTEST.md, § 8) |
| Dégâts par seconde d'un joueur qui frappe | 23.9 | hypothèse : moyenne des attaques principales des jeux d'armes (onglet Combat), sans l'armure des ennemis (comptée à part dans l'onglet Expéditions) |
