# 0017 — La charrette et le registre commun

- **Date** : 2026-11-03
- **Statut** : choisi par Flo le 2026-11-03 ; remplace la Décision 1 du plan (« chaque compte garde ses personnages »)
- **Règles** : `design/village.md` (relais, registre) ; tâches : jalon R de `PLAN.md`

## Problème

À la connexion, Veloren montre son écran de personnages : chacun crée et garde les siens. Le village d'Icebreaker
veut l'inverse (façon diligence de Darkest Dungeon) : on prend une recrue, et les personnages appartiennent au village.

## Options

1. **Écran de personnages remplacé** : la charrette est un menu avant d'entrer. Simple, mais hors du village.
2. **Choix dans le village** : on arrive en fantôme (sans corps), on choisit à la charrette sur la place.
3. Registre **par compte** (Veloren) ou **commun** (tout le monde joue tout le monde).

## Décision

- **Option 2**, avec le mode spectateur de Veloren comme fantôme (ouvert à tous quand le registre est actif).
- Recrues **tirées au hasard** parmi 3 classes (guerrier, mage, archer) ; la charrette est renouvelée chaque jour de
  carte et **complétée au minimum à chaque retour de donjon**, pour que personne ne reste sans personnage.
- **Registre commun** : tout personnage libre se joue par n'importe qui (un joueur à la fois) ; chacun peut en
  **réserver jusqu'à 5** pour lui.
- Classe = **équipement de départ** pour commencer (`assets/icebreaker/classes.ron`).

## Conséquences

- Les personnages du registre sont rangés sous un compte fixe « registre » de la base de Veloren ; qui joue ou
  réserve quoi vit dans `campagne/registre.ron`.
- Un donjon lancé seul garde l'écran de personnages de Veloren (mode essai).
- Restent à trancher : prix d'engagement, places du registre, mort d'un personnage du registre (`PLAN.md`).
