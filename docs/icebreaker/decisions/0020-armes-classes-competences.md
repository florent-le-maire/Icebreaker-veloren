# 0020 — Armes, classes et compétences : une couche à nous sur les capacités de Veloren

- **Date** : 2026-11-11
- **Statut** : choisi par Flo le 2026-11-11 (règles de jeu) ; architecture proposée par l'agent après lecture du code
- **Règles** : `design/classes.md` (section « Veloren ») ; tâches : `PLAN-CLASSES.md` (jalon A)

## Problème

Une classe du prototype n'est qu'un équipement de départ et des compétences Veloren débloquées (R.1, R.6 à R.8). Il
n'y a ni recharge, ni niveau, ni passif ; n'importe qui manie n'importe quelle arme ; deux recrues de même classe sont
identiques. Améliorer un personnage n'est pas intéressant et les classes n'ont pas d'identité.

Flo veut : des compétences qui dépendent des armes tenues (façon Guild Wars 2), 2 compétences d'arme + 1 ultime +
1 passif, des niveaux, des recharges, et des recrues tirées avec un lot différent.

## Options

1. **Rester dans le système de Veloren** : manifeste des capacités par type d'outil, déblocage par l'enum `Skill`,
   emplacements `ActiveAbilities` sauvés en base. Un ultime indépendant de l'arme ne s'y exprime pas ; un niveau ou
   une recharge demandent de toute façon un composant à nous ; chaque compétence nouvelle touche une enum Rust ; c'est
   la zone que Veloren annonce réécrire.
2. **Une couche Icebreaker à part** : nos données décrivent jeux d'armes, compétences, ultimes et passifs ; la fiche
   du registre porte ce que le personnage a acquis ; un composant synchronisé le projette en jeu ; un crochet dans
   `handle_ability` résout les touches 1 à 3 depuis nos données et laisse Veloren exécuter la capacité (gestes,
   dégâts, animations).
3. **Tout réécrire** (états de combat à nous) : hors de portée du prototype.

## Décision

**Option 2.** Règles de jeu (Flo) :

- le **jeu d'armes** en mains donne les attaques de base et un pool de compétences d'arme, dont 2 placées ;
- la **classe** donne un pool d'ultimes (1 placé, longue recharge) et de passifs (1 actif), sans niveau ;
- jouer un jeu d'armes en expédition donne de la **maîtrise** et des **points d'arme**, dépensés à la **salle
  d'armes** pour débloquer ou monter une compétence de cette arme ; niveaux 1 à 5, les niveaux 3 et 5 transforment la
  compétence ;
- une recrue arrive avec un **lot tiré au hasard** ; ensuite tout s'apprend à la salle d'armes (or) ;
- en expédition, on change d'arme librement par l'inventaire ; trois classes pour commencer (guerrier, mage, archer).

Technique (agent) : capacités de Veloren réutilisées par leur id de fichier ; une variante `Ability::Icebreaker` pour
garder les animations ; postures de l'épée neutralisées pour ces personnages ; recharge armée par le serveur seul ;
passif appliqué par un composant visible de tous ; maîtrise comptée à l'attribution d'expérience ; arbre de
compétences de Veloren coupé pour les personnages du registre.

## Conséquences

- Le protocole client-serveur change (deux composants synchronisés) : client et serveur se recompilent ensemble, le
  VPS est à redéployer.
- Nouveaux crochets dans `common/` (lancement des capacités, système de buffs) : courts et commentés, à surveiller
  lors des mises à jour de Veloren. `CharacterAbility` refuse les champs inconnus : un test charge toutes nos
  capacités et signale une copie cassée.
- `classes.ron` perd `competences` et `touches` (A.18) ; R.6 à R.8 sont remplacés. Les personnages créés avant
  reçoivent le lot par défaut de leur classe.
- La salle d'armes devient un bâtiment du village (remplace B.12 de `PLAN-BOUCLES.md` ; tranche Q13).
- `decisions/0010` (neuf classes et combos de zones) reste une référence : le nombre de classes et les combos sont
  toujours ouverts (Q36).
- Piste mod (`PLAN-MOD.md`) : ces crochets sont dans le moteur, pas dans le plugin ; à compter dans l'écart.
