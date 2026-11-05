+++
ordre = 2
titre = "Le village"
icone = "voxygen/element/ui/map/buttons/town.png"
schema = "plan-village"
resume = "La base commune : on y range, on y paie, on y soigne, on y part. Il grandit avec l'or que l'équipe y met."
design = ["village.md"]
reglages = ["village.ron", "vue_village.ron"]
taches = ["V", "C", "X"]
systemes = [
  "server/src/icebreaker_village.rs",
  "server/src/icebreaker_buildings.rs",
  "server/src/icebreaker_warehouse.rs",
  "server/src/icebreaker_treasury.rs",
  "server/src/icebreaker_services.rs",
  "voxygen/src/session/icebreaker_vue.rs",
]

[[chiffre]]
cle = "maisons_au_depart"
nom = "Maisons bâties au départ"

[[chiffre]]
cle = "terrains_ouverts"
nom = "Terrains à bâtir proposés"

[[chiffre]]
cle = "prix_maison"
nom = "Prix d'une maison"
unite = "or"

[[chiffre]]
cle = "heures_chantier"
nom = "Durée d'un chantier"
unite = "h de carte"

[[chiffre]]
cle = "habitants_par_maison"
nom = "Habitants par maison"
+++

## La règle

Le village est une ville de Veloren dont on a gardé la place et les maisons les plus proches. Les six premières
sont les **bâtiments** de l'équipe, chacun tenu par un personnage devant sa porte :

- **Entrepôt** : un inventaire commun à tous les joueurs ; on y dépose et on y reprend par la fenêtre d'échange.
- **Caisse** : l'or commun. Chacun garde son or, mais peut en donner à la caisse ; c'est elle qui paie les chantiers.
- **Intendance** : un marchand de Veloren (potions, vivres, rachat du butin).
- **Infirmerie** : soin complet contre de l'or.
- **Départ** : le chef choisit la destination, les autres rejoignent, et on part groupés par téléportation.
- **Charrette** : les recrues du jour (voir la fiche Charrette et registre).

La touche **U** ouvre la **vue de dessus** : la caméra monte, chaque bâtiment a son cadre coloré et son icône, un
clic ouvre son panneau. C'est de là qu'on commande et qu'on part.

Le village **grandit** : les autres maisons sont des terrains à bâtir, les plus proches de la place d'abord. Un
chantier se paie avec l'or de la caisse et la maison sort de terre au fil du temps de carte ; une maison finie amène
des habitants. Ce qui est bâti est gardé dans la partie, d'une session à l'autre.

## Ce qui reste à faire

Le chantier visible qui pousse sans relancer (C.4), les nouveaux habitants (C.5), les paliers d'or qui ouvrent des
terrains plus loin (C.6). `PLAN-BOUCLES.md` propose de remplacer les maisons anonymes par des bâtiments nommés à
niveaux (forge, salle d'armes, taverne…) : à trancher avant C.5.
