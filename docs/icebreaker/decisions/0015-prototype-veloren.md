# 0015 — Prototype sur Veloren, en projet séparé

- **Date** : 2026-10-28
- **Statut** : prototype accepté par Flo ; le choix du moteur pour la suite reste à valider avec Sayanel

## Problème

La version Godot demande de produire nous-mêmes modèles, animations et décors : c'est ce qui coûte le plus et
ralentit le plus. L'équipe veut savoir si un jeu existant, gratuit et modifiable, peut porter le concept avec ses
modèles, animations, créatures et donjons déjà faits.

## Options étudiées

1. **Continuer en Godot** (2D prioritaire, `0011`) : liberté totale, tout le code et les données sont à nous, mais
   tous les visuels sont à produire.
2. **Un mod d'un jeu payant** (Vintage Story, Valheim, Bannerlord) : beaux assets, mais chaque joueur doit acheter
   le jeu ; Bannerlord n'a pas de campagne en coop.
3. **Veloren** (RPG voxel, gratuit, open source, Rust, licence GPL v3) : donjons avec boss, villages, monde généré,
   simulation de PNJ qui vit sans les joueurs (`rtsim`), serveur dédié Linux, animations écrites en code.

## Ce que le prototype a montré (2026-10-28)

Clone de Veloren dans `../Veloren-test`, branche locale `icebreaker-traces` :

- **Traces de donjon** : monstres tués et coffres ouverts ne reviennent plus (au rechargement et au redémarrage du
  serveur), sauvegardés avec les données `rtsim`. Environ 80 lignes ; une recompilation prend environ 1 min.
- **Instances** : un serveur génère un petit monde avec un seul donjon choisi, en **~4 s pour 2 km** (~9 s pour
  4 km, build de développement) ; les joueurs apparaissent devant. Six types de donjon se placent ; terracotta,
  myrmidons et sahagins demandent un biome à imposer.
- Touches AZERTY par défaut, titre « Icebreaker » au menu.
- Faisable sans refonte : nos générateurs de donjon (à partir des briques de Veloren), zone fermée avec horizon,
  IA des ennemis (`server/agent`, `rtsim`), classes (capacités en fichiers de données). Gros chantiers : les
  classes et la carte stratégique façon Mount & Blade (le monde Veloren entier peut servir de carte).

## Conséquences si on continue sur Veloren

- **Projet séparé** : une copie modifiée de Veloren dans son propre dépôt ; ce dépôt-ci garde la vision et le design
  (`docs/VISION.md`, `docs/design/`), copiés dans le projet Veloren sous `docs/icebreaker/`.
- **Licence GPL v3** : distribuer le jeu (même entre amis) oblige à fournir le code source modifié ; pas de jeu
  fermé ou vendu.
- **Langage Rust** : `src/` (C# pur, dont `WorldSim`) n'est pas réutilisable tel quel ; les règles se réécrivent.
- **Architecture** : un processus serveur par instance, comme prévu par `0005`.
- Suivre les mises à jour de Veloren demandera de résoudre des conflits : garder nos changements regroupés.

## Recommandation

Continuer le prototype Veloren (village, pillage sur la route) jusqu'à une démo à l'équipe, sans arrêter la version
Godot ; trancher après cette démo.
