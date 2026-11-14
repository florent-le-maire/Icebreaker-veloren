# Game design — Âmes (idée de Sayanel)

*Idée de Sayanel, notée le 2026-11-03, règles de principe fixées le même jour ; à concevoir par Sayanel.* Ce fichier
rassemble ce qu'il faut pour la concevoir : l'idée, ses règles, les races et classes d'ennemis possibles, et ce que
Veloren sait déjà des créatures (stats, résistances). Rien n'est codé.

## L'idée

- On a un **build** (guerrier, etc.). En plus, tuer un monstre donne une **faible chance de récupérer son âme**.
- Une âme **inconnue** apparaît en **blanc**. Pour la découvrir, on l'**équipe** : elle devient **liée** au personnage
  (*soul bound*).
- Une fois découverte, cette âme est **connue** : si on la retrouve, elle s'affiche en **couleur** (par exemple vert).
- Un **Codex** au village liste les âmes découvertes.

Lien avec les piliers (`piliers.md`) : l'âme **vient du monde** (prise sur un monstre) et **existe encore en sortant**
(liée au personnage, inscrite au Codex). Le choix d'équiper une âme inconnue est un pari : on la lie sans savoir ce
qu'elle fait.

## Règles (Flo et Sayanel, 2026-11-03)

Les chiffres viendront plus tard ; ce qui suit fixe le principe.

- **Ce que donne une âme** : soit un **bonus** permanent (vitesse d'attaque, vitesse de déplacement, déplacement
  silencieux…), soit un **actif** (une capacité à déclencher).
- **Une âme par type de monstre** : ce que donne l'âme dépend de la race **et** du rôle du monstre. Exemples :
  assassin gnarling → bruit réduit ; assassin humain → coup critique dans le dos ×2 ; guerrier gnarling →
  consommation d'endurance réduite.
- **Combien d'âmes existent** : **3 âmes par classe de chaque race**, soit 3 × 3 classes (guerrier, archer, mage) × 6
  races = 54 au départ. Nombre susceptible d'évoluer.
- **Emplacements d'âme** : leur nombre dépend d'un **niveau d'âme** propre au personnage (niveau 1 : une âme, niveau
  2 : deux âmes…). Comment ce niveau monte : à définir plus tard.
- **Équiper** : une âme est **à usage unique** : l'équiper la place dans un emplacement d'âme du personnage, et elle y
  est **liée pour toujours**.
- **À la mort**, le personnage **perd ses âmes**.
- **Chance d'obtenir une âme** : faible en général, **plus forte sur les monstres forts et les boss**.
- **Découvrir** : équiper une âme inconnue révèle ce qu'elle fait au joueur. La découverte n'est **validée** que si le
  personnage **rentre vivant** de l'expédition : alors **tout le village** connaît cette âme et sa couleur (Codex).
  Mort avant le retour : la découverte est perdue avec l'âme.
- **Retrouver une âme connue** : un autre exemplaire de la même âme peut être équipé par un autre joueur (ou un autre
  personnage) ; la connaissance, elle, ne change pas.
- **Couleurs** : une âme inconnue est **blanche** ; une fois découverte, elle prend **sa propre couleur**, propre à
  cette âme, pour la reconnaître d'un coup d'œil. Les couleurs sont **redistribuées à chaque partie** (une partie se
  termine quand le village est achevé ou le boss final battu) : d'une partie à l'autre, il faut redécouvrir quelle
  couleur correspond à quelle âme (comme les potions non identifiées des rogue-likes). Détails à discuter plus tard.
- **Résistances** : selon les âmes que Sayanel imaginera (voir « Résistances » plus bas).

## Les races

Veloren distingue deux choses : les **espèces humanoïdes** (Humain, Nain, Elfe, Orc, Draugr, Danari ; jouables, même
squelette, n'importe quelle arme) et les **créatures** (gnarlings, gobelins, adlets… ; corps à part, armes et
attaques propres). Une race humanoïde se décline facilement en guerrier, archer ou mage ; une créature dépend des
modèles et attaques que Veloren a déjà.

| Race proposée | Dans Veloren | Ennemis déjà faits | Verdict |
|---|---|---|---|
| **Humain** | espèce `Human` | cultistes (corps humanoïde au hasard), bandits | **Oui**, prêt |
| **Nain** | espèce `Dwarf` + créature `IronDwarf` | carrière naine : mineur, garde de mine, nain de fer, gardien des flammes, maître de forge | **Oui**, prêt (donjon « mine ») |
| **Gobelin** | créatures `Gnarling` (+ chef) et `GoblinThug` / `GoblinChucker` / `GoblinRuffian` | fort gnarling (notre donjon actuel), gobelins sauvages | **Oui : les gnarlings remplacent les gobelins** (décision du 2026-11-03). Le *halfelin* n'existe pas dans Veloren ; les « goblins » de Veloren ne sont pas retenus |
| **Orc** | espèce `Orc` | **aucun** camp ni donjon orc | **Oui**, mais à créer : corps orc + armes de Veloren (peu de travail, pas de nouveau modèle) |
| **Elfe** | espèce `Elf` | **aucun** | Comme l'orc : à créer avec le corps elfe |
| **Mort-vivant** | espèce `Draugr` (jouable) ; créatures `Husk`, `Huskbrute`, `Jiangshi`, `Dullahan` ; vampires `Strigoi`, `Bloodservant` | gouffres des cultistes (husks), château des vampires | **Oui**, riche ; pas de squelette « classique » |

Autres familles déjà prêtes, si Sayanel en veut plus : **Danari** (petite race jouable), **Adlet** (peuple des glaces),
**Sahagin** (hommes-poissons), **Myrmidon** (hommes-fourmis), **Haniwa / Terracotta** (statues animées),
**Saurok** (hommes-lézards), **Gnoll**, et des grands (ogre, trolls, cyclope, minotaure).

## Les trois classes d'ennemis

| Race | Guerrier | Archer | Mage |
|---|---|---|---|
| Humain | cultiste (épée), bandit | archer du village (`bowman`) en ennemi | cultiste (bâton) |
| Nain | garde de mine, mineur (pioche-épée), nain de fer | *à faire* (arbalète ou arc) | gardien des flammes (bâton) |
| Gobelin | gnarling bûcheron (hache), détrousseur (dague) | gnarling traqueur (sarbacane) | chef gnarling (bâton), totems |
| Orc | *à faire* : corps orc + épée ou hache | *à faire* : + arc | *à faire* : + bâton ou sceptre |
| Elfe | *à faire* | *à faire* | *à faire* |
| Mort-vivant | husk, draugr (épée) | *à faire* : draugr + arc | *à faire* : draugr + bâton ; seigneur de guerre cultiste |

Pour les humanoïdes (humain, nain, orc, elfe, draugr), « à faire » veut dire une fiche d'entité (`assets/common/entity`)
avec une arme existante : quelques minutes chacune.

**Dans le prototype (jalon H, 2026-11-13, Flo)** : donjons générés et grottes sont peuplés d'**une seule race**
humanoïde (humain, orc, nain, elfe, draugr, danari ; tirée par la graine du lieu), en guerrier, archer et mage avec
l'équipement des classes des joueurs (`assets/icebreaker/ennemis/`) ; le chef (et le boss) est un membre de la race qui
a les capacités du chef gnarling. Les **monstres solitaires** (araignées, mandragores…) n'ont pas de race : nids du
donjon (`solitaires` de `generation.ron`), poches et culs-de-sac de la grotte (`solitaires` des ambiances). Le fort
gnarling ne sert plus en campagne.

**Comportement par race (jalon Y, 2026-11-14, Flo ; à valider, `QUESTIONS.md` Q50)** : réglages dans
`assets/icebreaker/ennemis/races.ron`. Faits : **tous** choisissent leur cible parmi les joueurs qu'ils ont
repérés (Y.9) : un joueur debout passe avant un joueur à terre ; si tous sont à terre, ils sont achevés ; entre joueurs
debout ils se répartissent (un joueur déjà visé par un allié compte comme 8 blocs plus loin) ; ils rechoisissent toutes
les 3 s environ, et tout de suite quand leur cible tombe. Les **danari** sont fourbes : à deux ou plus (à 20 blocs), un
joueur à terre à moins de 20 blocs passe en tête (ils l'achèvent), sauf si quelqu'un vient de les frapper (3 s) ; un
danari seul fait comme les autres ; ils fuient sous 30 % de leurs PV. Les **humains** font des rondes : jusqu'à deux groupes de trois
guerriers vont de salle en salle (trois salles, sans clé ni chute, jamais celle du chef), s'arrêtent 5 s, puis
reviennent. Prévus : orcs en rage, nains qui tiennent leur salle, elfes tireurs, draugr endormis.

## Les stats dans Veloren

**À retenir d'abord** : Veloren n'a **pas de stats par race** (toutes les espèces humanoïdes ont 100 PV, 100
d'endurance, 100 d'équilibre) ni de **résistance chiffrée** au poison, au feu, etc. Il a :
- des **immunités** tout ou rien, par corps (tableau plus bas) ;
- une **réduction des dégâts** générale, donnée par l'armure ;
- ce que le personnage porte et les **effets** (buffs) qu'il subit.

Les âmes devront donc apporter **nos propres résistances** (fichier de données), en s'appuyant sur ces briques.

### Stats d'un personnage ou d'une créature (`common/src/comp/stats.rs`)

Ce qu'un effet (et donc une âme) peut modifier :

| Stat | Effet |
|---|---|
| PV max | multiplicateur et ajout |
| Endurance max | multiplicateur et ajout |
| Réduction des dégâts | part des dégâts reçus retirée |
| Réduction d'équilibre | résistance aux mises à terre (*poise*) |
| Dégâts d'attaque | multiplicateur |
| Dégâts d'équilibre | ce qu'on fait vaciller chez l'autre |
| Vitesse d'attaque, de charge, d'armement, de récupération | multiplicateurs |
| Vitesse de déplacement, de nage, saut | multiplicateurs |
| Gain et économie d'endurance | endurance rendue par les coups, coût des capacités |
| Coups précis (critiques) | puissance des coups précis |
| Pénétration | part de l'armure adverse ignorée |
| Résistance au contrôle | réduit étourdissements, ralentissements |
| Recul | force du recul infligé |
| Effets à l'attaque, quand on est blessé, à sa mort | ajouter un effet (poison, saignement…) |

### Stats d'une armure

| Stat | Effet |
|---|---|
| **Protection** | réduction des dégâts = prot / (60 + prot), toutes pièces additionnées |
| **Résilience** | même formule pour l'équilibre (moins de mises à terre) |
| Endurance max, gain d'endurance | ajoutés |
| Précision | puissance des coups précis |
| **Discrétion** | les créatures perçoivent moins loin |

### Stats d'une arme

Puissance (dégâts), puissance des effets (poison, feu…), vitesse, portée, économie d'endurance, force des buffs,
temps pour la sortir. Les armes des monstres sont à 1,0 partout (leur force vient du corps et des attaques) ; la
sarbacane du traqueur gnarling : effets 0,8, vitesse 0,6.

### Types de dégâts

| Type | Particularité |
|---|---|
| Perforant (flèches, estocs) | ignore une partie de l'armure |
| Tranchant (épées, haches) | vide l'endurance ; dégâts en plus si elle est à zéro |
| Contondant (marteaux) | plus de dégâts d'équilibre contre les cibles très armurées |
| Énergie (magie) | le reste |

### Effets négatifs (les « poisons »)

| Effet | Ce qu'il fait dans Veloren |
|---|---|
| **Empoisonné** | **vide l'endurance** (pas les PV !) |
| Saignement | perte de PV dans le temps |
| Brûlure | perte de PV ; empêche le gel |
| Estropié | ralentit et fait perdre des PV |
| Gelé | ralentit déplacement et attaques, plus fragile à l'équilibre |
| Refroidi | ralentit, plus fragile à l'équilibre |
| Mouillé | sol glissant ; empêche de brûler |
| Entravé | ralentit |
| Enraciné | immobile |
| Maudit | PV max qui fondent |
| Essoufflé | ralentit, moins d'endurance rendue |
| Déséquilibré | plus fragile à l'équilibre |
| Amnésie | capacités secondaires bloquées |
| Coup de chaleur | ralentit, moins d'endurance rendue |

### Stats de base des races candidates (`common/src/comp/body/mod.rs`)

PV, endurance et équilibre de base, avant arme, armure et multiplicateurs d'Icebreaker (`assets/icebreaker/combat.ron`,
L.1). « Défaut » : 100 pour les petites créatures, 300 pour les grandes.

| Race | Corps | PV | Endurance | Équilibre | Immunités |
|---|---|---|---|---|---|
| Humain, Nain, Elfe, Orc, Draugr, Danari | humanoïde | 100 | 100 | 100 | aucune |
| Gobelin | gnarling | 50 | 100 | défaut | aucune |
| Gobelin | chef gnarling | 150 | 100 | 130 | aucune |
| Nain | nain de fer | 250 | 100 | 300 | saignement, brûlure |
| Nain | gardien des flammes | 2 000 | 100 | 300 | brûlure |
| Mort-vivant | husk | 50 | 100 | défaut | saignement |
| Mort-vivant | husk colosse | 800 | 300 | défaut | saignement |
| Mort-vivant | jiangshi | 250 | 100 | défaut | saignement |
| Mort-vivant | dullahan | 600 | 400 | défaut | saignement, estropié |
| Mort-vivant | strigoi | 800 | 300 | défaut | aucune |
| Mort-vivant | serviteur de sang | 300 | 100 | défaut | aucune |
| Humain (boss) | seigneur de guerre / sorcier cultiste | 200 | 240 | défaut | aucune |

**Personne n'est immunisé au poison** dans Veloren. Les immunités existantes : saignement (morts-vivants, statues,
golems), estropié, brûlure (créatures de feu), gel (créatures des glaces), entravé (insectes).

## Résistances

Veloren n'a que des immunités tout ou rien ; on peut créer **nos résistances chiffrées** (vérifié dans le code,
2026-11-03), à concevoir selon les âmes :
- tout effet appliqué passe par `BuffKind::modify_data` (`common/src/comp/buff.rs`), qui connaît déjà les stats de la
  cible et y réduit la durée des contrôles selon la « résistance au contrôle ». Une résistance par effet s'y ajoute de
  la même façon : force et/ou durée × (1 − résistance) ; 1 = immunité, valeur négative = vulnérabilité ;
- valeurs de base **par race** dans un fichier de données ; un effet « résistance » que donnent âmes, armures ou
  potions s'y ajoute ;
- résistances **par type de dégâts** (perforant, tranchant, contondant, énergie) possibles aussi, dans le calcul des
  dégâts (`common/src/combat.rs`).

Une tâche de quelques heures pour les effets, une autre pour les types de dégâts ; l'affichage (fiche, Codex) à part.
