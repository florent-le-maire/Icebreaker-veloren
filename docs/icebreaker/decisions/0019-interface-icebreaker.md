# 0019 — Interface Icebreaker : un panneau typé, proche de Veloren

- **Date** : 2026-11-11
- **Statut** : proposé ; style choisi par Flo le 2026-11-11 ; le reste est à trancher après lecture des maquettes
  (`QUESTIONS.md`, Q39)
- **Règles** : `PLAN-INTERFACE.md` (charte, règles, tâches du jalon N) ; maquettes :
  <https://claude.ai/artifact/KXR2tEFGXNNdTk8QJMUcdW>

## Problème

L'interface ajoutée par Icebreaker (3 éléments au menu, 10 en jeu) marche, mais :

- au village, un clic n'a pas de réponse immédiate : le bouton envoie une commande de chat, la réponse arrive dans le
  chat, et le panneau n'est mis à jour qu'à la demande suivante (une par seconde) ; sa hauteur vient de l'image
  précédente, donc le cadre saute ;
- le panneau est une pile de phrases et une colonne de boutons, sans défilement ni bouton grisé ; il est collé à
  gauche, sur la fenêtre de groupe de Veloren ; l'encadré d'expédition et le panneau de balise ont la même place ;
- ce qui compte passe par le chat : accueil du fantôme, refus, annonces, bilan de retour, victoire ;
- en donjon, la recharge de la balise cache la quantité, la mort tient en une ligne, on ne sait pas quel allié on
  regarde ; au menu, 4 parties au plus et un écran vide pendant le lancement.

La décision `0018` (option 1) avait réparé les défauts un par un en gardant les cadres de Veloren ; son option 2
(« une charte à nous ») restait possible plus tard.

## Options

1. **Garder `0018`** : le panneau générique actuel (titre, lignes, boutons) et des correctifs au cas par cas.
2. **Un panneau typé, piloté par le serveur** : le serveur envoie des blocs (chiffres, jauge, lignes avec état,
   actions permises ou non, message), le client les range toujours au même endroit, dans une seule fenêtre.
3. **Des fenêtres dédiées dans le client**, une par bâtiment, chacune avec sa mise en page et ses messages réseau.

## Décision proposée

- **Option 2.** Elle donne la mise en page voulue (chiffres en grand, boutons grisés avec leur raison, listes qui
  défilent) avec un seul gabarit à écrire côté client ; ajouter un bâtiment reste un changement côté serveur.
- **Style proche de Veloren** (choix de Flo) : polices `Alkhemikal` et `EnterCommand`, panneaux sombres, bordures
  acier, boutons rouille ; un seul accent à nous, l'or de l'anneau de verrouillage. Aucune nouvelle texture.
- **Le serveur pousse** : l'état d'un panneau part quand il change et juste après une action ; le client ne le
  redemande plus chaque seconde.
- **Trois zones fixes** : bandeaux en haut au centre, fenêtre au centre, messages à droite sous la mini-carte.
- **Hors du chat** : fantôme, expédition, refus, annonces et bilan ont leur bandeau, leur message ou leur fenêtre ;
  le chat garde une ligne de trace.

## Conséquences

- Le protocole client-serveur change (comme pour `0018`) : client et serveur se recompilent ensemble.
- Piste mod (`PLAN-MOD.md`, Q6) : le contenu des panneaux reste piloté par le serveur ; le code client ajouté est un
  seul moteur de rendu, plus les bandeaux et les messages.
- Les textes des panneaux restent écrits en français côté serveur ; les passer en clés de traduction est une tâche
  à part.
- `0018` reste la base (états propres du plan, projection, enseignes, recrues en PNJ) ; cette note remplace sa
  phrase « le visuel reste celui de Veloren » par la charte de `PLAN-INTERFACE.md`.
- L'option 3 reste possible pour un écran qui ne rentrerait pas dans le gabarit (aucun aujourd'hui).
