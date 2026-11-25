//! Icebreaker : les onglets « Combat » (ce que valent les compétences des joueurs) et « Ennemis » (ce que
//! valent les ennemis face à chaque classe).

use crate::{
    calcul,
    capacite::{Capacite, net},
    donnees::{Donnees, Ligne, Sorte},
    hyp::Hyp,
    style::{self, Nb, Val, a1, accent, col, col_longue},
};
use anyhow::Result;
use rust_xlsxwriter::{Color, ConditionalFormat3ColorScale, ConditionalFormatDataBar, Worksheet};
use std::collections::HashMap;

pub const ONGLET_COMBAT: &str = "Combat";
pub const ONGLET_ENNEMIS: &str = "Ennemis";

// Colonnes de l'onglet Combat.
const C_NOM: u16 = 2;
const C_TOUCHES: u16 = 6;
const C_BASE: u16 = 7;
const C_PN: u16 = 8;
const C_PA: u16 = 9;
const C_MULT: u16 = 10;
pub const C_DEG: u16 = 11;
const C_ENERGIE: u16 = 12;
const C_GESTE: u16 = 13;
const C_RECHARGE: u16 = 14;
pub const C_DPS: u16 = 15;
pub const C_DPS_SOUTENU: u16 = 16;
/// Première colonne des ennemis : deux colonnes par ennemi (touches, secondes).
pub const C_ENNEMIS: u16 = 18;

/// Où une ligne de compétence est écrite, et ce qu'elle vaut avec les valeurs du jeu.
pub struct Ecrite {
    pub ligne: u32,
    pub classe: String,
    pub jeu: String,
    pub nom: String,
    pub sorte: Sorte,
    pub niveau: u8,
    pub degats: f64,
    pub dps: f64,
    pub dps_soutenu: f64,
    /// Par ennemi : (touches pour tuer, secondes pour tuer).
    pub contre: Vec<(f64, f64)>,
}

fn echelle(feuille: &mut Worksheet, debut: u32, fin: u32, colonne: u16) -> Result<()> {
    let regle = ConditionalFormat3ColorScale::new()
        .set_minimum_color(Color::RGB(0x8FD19E))
        .set_midpoint_color(Color::RGB(0xFFE9A8))
        .set_maximum_color(Color::RGB(0xF2A0A0));
    feuille.add_conditional_format(debut, colonne, fin, colonne, &regle)?;
    Ok(())
}

fn barres(feuille: &mut Worksheet, debut: u32, fin: u32, colonne: u16, couleur: u32) -> Result<()> {
    let regle = ConditionalFormatDataBar::new().set_fill_color(Color::RGB(couleur)).set_solid_fill(true);
    feuille.add_conditional_format(debut, colonne, fin, colonne, &regle)?;
    Ok(())
}

/// Le multiplicateur contre une créature d'une capacité : celui de son type d'attaque, ou la moyenne pesée par
/// les dégâts quand elle en a plusieurs (une explosion et une onde de choc). (formule, valeur)
fn multiplicateur(c: &Capacite, hyp: &Hyp) -> (String, f64) {
    let valeur = c.pondere(|s| hyp.v(s.cle())) / c.degats();
    let formule = match c.composantes.as_slice() {
        [seule] => format!("={}", hyp.r(seule.source.cle())),
        plusieurs => {
            let somme: Vec<String> =
                plusieurs.iter().map(|p| format!("{}*{}", p.degats, hyp.r(p.source.cle()))).collect();
            format!("=({})/{}", somme.join("+"), c.degats())
        },
    };
    (formule, valeur)
}

/// Le geste d'une capacité, pour les calculs de temps.
fn geste_de(c: &Capacite) -> calcul::Geste {
    calcul::Geste { duree: c.geste, avant: c.avant, apres: c.apres, tenu: c.tenu }
}

/// La formule du temps qu'il faut pour placer `t` touches (cases `t`, `n` touches par usage, `g` geste) : dans
/// une seule utilisation si elle suffit, sinon `suite`. Une capacité tenue dure le temps qu'il faut.
fn formule_secondes(c: &Capacite, t: &str, n: &str, g: &str, suite: &str) -> String {
    if c.tenu {
        format!("={}+{g}*{t}/{n}", c.avant)
    } else {
        format!("=IF({t}<={n},{}+({g}-{})*{t}/{n},{suite})", c.avant, ((c.avant + c.apres) * 1e4).round() / 1e4)
    }
}

pub fn ecrire_combat(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp) -> Result<Vec<Ecrite>> {
    feuille.set_name(ONGLET_COMBAT)?;
    let mut colonnes = vec![
        col("Classe", 10.0, Nb::Texte),
        col("Jeu d'armes", 16.0, Nb::Texte),
        col("Compétence", 22.0, Nb::Texte),
        col("Sorte", 15.0, Nb::Texte),
        col("Niveau", 8.0, Nb::Entier),
        col("Type d'attaque", 15.0, Nb::Texte),
        col("Touches par usage", 9.0, Nb::Dec1),
        col("Dégâts de base (usage)", 10.0, Nb::Dec1),
        col("Puissance du niveau", 10.0, Nb::Fois),
        col("Puissance de l'arme", 10.0, Nb::Fois),
        col("Contre une créature", 10.0, Nb::Fois),
        col("Dégâts par usage", 10.0, Nb::Dec1),
        col("Énergie", 9.0, Nb::Dec1),
        col("Geste", 9.0, Nb::Secondes),
        col("Recharge", 9.0, Nb::Secondes),
        col("Dégâts par seconde (pendant le geste)", 12.0, Nb::Dec1),
        col("Dégâts par seconde (avec la recharge)", 12.0, Nb::Dec1),
        col("Dégâts par point d'énergie", 11.0, Nb::Dec1),
    ];
    let mut groupes = vec![
        (0, 5, "La compétence".to_string()),
        (6, 11, "Ce qu'elle frappe".to_string()),
        (12, 17, "Ce qu'elle coûte et rend".to_string()),
    ];
    for (i, ennemi) in d.ennemis.iter().enumerate() {
        let c = C_ENNEMIS + 2 * i as u16;
        colonnes.push(col("Touches pour tuer", 9.0, Nb::Entier));
        colonnes.push(col("Secondes pour tuer", 9.0, Nb::Secondes));
        groupes.push((c, c + 1, format!("Contre : {}", ennemi.role)));
    }
    let c_notes = C_ENNEMIS + 2 * d.ennemis.len() as u16;
    colonnes.push(col_longue("Ce que le niveau apporte, et ce que le calcul suppose", 70.0));
    colonnes.push(col("Capacité (fichier)", 44.0, Nb::Texte));
    groupes.push((c_notes, c_notes + 1, "Notes".to_string()));

    let depart = style::bandeau(
        feuille,
        "Combat : ce que valent les compétences",
        "Une ligne par compétence et par niveau, lancée avec l'arme de départ du jeu d'armes, comme le fait le jeu. \
         Un « usage » est un geste entier sur une seule cible (une seconde pour un rayon tenu). « Secondes pour \
         tuer » : la compétence une fois, puis l'attaque principale jusqu'au bout ; armure de l'ennemi comprise.",
        accent::COMBAT,
        colonnes.len() as u16,
    )?;
    // Les données commencent deux lignes plus bas : titres de groupes, puis en-tête.
    let premiere = depart + 2;
    let mut lignes = Vec::new();
    let mut ecrites: Vec<Ecrite> = Vec::new();
    // L'attaque principale de chaque jeu d'armes : sa ligne et ses dégâts par seconde.
    let mut principales: HashMap<String, (u32, f64)> = HashMap::new();
    for (i, l) in d.lignes.iter().enumerate() {
        let ligne = premiere + i as u32;
        let (valeurs, ecrite) = ligne_combat(l, ligne, d, hyp, principales.get(&l.jeu).copied());
        if l.sorte == Sorte::Base && l.nom == "Attaque principale" {
            principales.insert(l.jeu.clone(), (ligne, ecrite.dps));
        }
        lignes.push(valeurs);
        ecrites.push(ecrite);
    }
    let (entete, debut, fin) = style::tableau(feuille, depart, 0, accent::COMBAT, &groupes, &colonnes, &lignes, |i| {
        format!("{}/{}", d.lignes[i].jeu, d.lignes[i].nom)
    })?;
    debug_assert_eq!(debut, premiere);
    feuille.autofilter(entete, 0, fin, colonnes.len() as u16 - 1)?;
    feuille.set_freeze_panes(debut, C_NOM + 1)?;
    barres(feuille, debut, fin, C_DEG, 0xF2A7A7)?;
    barres(feuille, debut, fin, C_DPS, 0xF6C28B)?;
    barres(feuille, debut, fin, C_DPS_SOUTENU, 0xF6C28B)?;
    for i in 0..d.ennemis.len() as u16 {
        echelle(feuille, debut, fin, C_ENNEMIS + 2 * i)?;
        echelle(feuille, debut, fin, C_ENNEMIS + 2 * i + 1)?;
    }
    Ok(ecrites)
}

fn ligne_combat(l: &Ligne, ligne: u32, d: &Donnees, hyp: &Hyp, principale: Option<(u32, f64)>) -> (Vec<Val>, Ecrite) {
    let c = &l.capacite;
    let blesse = c.blesse();
    let cle_arme = format!("jeu.{}.puissance", l.jeu);
    let puissance_arme = hyp.v(&cle_arme);
    let (f_multiplicateur, v_multiplicateur) =
        if blesse { multiplicateur(c, hyp) } else { (String::new(), f64::NAN) };
    let (touches_usage, base) = (c.touches(), c.degats());
    let degats = calcul::degats(base, l.puissance, puissance_arme, v_multiplicateur);
    let dps = degats / c.geste;
    let dps_soutenu = degats / c.geste.max(l.recharge);
    let r = |colonne: u16| a1(ligne, colonne);
    let mut valeurs = vec![
        Val::t(&l.classe),
        Val::t(&l.jeu_nom),
        Val::t(&l.nom),
        Val::t(l.sorte.nom()),
        if l.niveau == 0 { Val::t("base") } else { Val::N(f64::from(l.niveau)) },
        Val::T(c.types()),
        if blesse { Val::N(touches_usage) } else { Val::Vide },
        if blesse { Val::N(base) } else { Val::Vide },
        Val::N(l.puissance),
        Val::F(format!("={}", hyp.r(&cle_arme)), puissance_arme),
    ];
    if blesse {
        valeurs.push(Val::F(f_multiplicateur, v_multiplicateur));
        valeurs.push(Val::F(format!("={}*{}*{}*{}", r(C_BASE), r(C_PN), r(C_PA), r(C_MULT)), degats));
    } else {
        valeurs.extend([Val::Vide, Val::Vide]);
    }
    valeurs.push(Val::N(c.energie));
    valeurs.push(Val::N(c.geste));
    valeurs.push(if l.sorte == Sorte::Base { Val::Vide } else { Val::N(l.recharge) });
    if blesse {
        valeurs.push(Val::F(format!("={}/{}", r(C_DEG), r(C_GESTE)), dps));
        valeurs.push(Val::F(format!("={}/MAX({},{})", r(C_DEG), r(C_GESTE), r(C_RECHARGE)), dps_soutenu));
        valeurs.push(if c.energie > 0.0 {
            Val::F(format!("=IF({e}>0,{d}/{e},\"—\")", e = r(C_ENERGIE), d = r(C_DEG)), degats / c.energie)
        } else {
            Val::FVide(format!("=IF({e}>0,{d}/{e},\"—\")", e = r(C_ENERGIE), d = r(C_DEG)))
        });
    } else {
        valeurs.extend([Val::Vide, Val::Vide, Val::Vide]);
    }
    let mut contre = Vec::new();
    for (i, ennemi) in d.ennemis.iter().enumerate() {
        let (pv, armure) = (format!("ennemi.{}.pv", ennemi.role), format!("ennemi.{}.armure", ennemi.role));
        if !blesse {
            valeurs.extend([Val::Vide, Val::Vide]);
            contre.push((f64::NAN, f64::NAN));
            continue;
        }
        let (r_pv, r_armure) = (hyp.r(&pv), hyp.r(&armure));
        let c_touches = C_ENNEMIS + 2 * i as u16;
        let touches = calcul::touches_pour_tuer(hyp.v(&pv), degats / touches_usage, hyp.v(&armure));
        valeurs.push(Val::F(
            format!("=ROUNDUP(ROUND({r_pv}/(({d}/{t})*(1-{r_armure})),6),0)", d = r(C_DEG), t = r(C_TOUCHES)),
            touches,
        ));
        // Une attaque de base se répète ; une compétence se lance une fois, puis on finit à l'attaque principale.
        let (t, n, g) = (r(c_touches), r(C_TOUCHES), r(C_GESTE));
        let (formule, secondes) = match principale.filter(|_| l.sorte != Sorte::Base) {
            Some((ligne_principale, dps_principale)) => (
                formule_secondes(c, &t, &n, &g, &format!(
                    "{g}+({r_pv}/(1-{r_armure})-{deg})/{p}",
                    deg = r(C_DEG),
                    p = a1(ligne_principale, C_DPS),
                )),
                calcul::secondes_avec_competence(
                    touches,
                    touches_usage,
                    geste_de(c),
                    hyp.v(&pv),
                    hyp.v(&armure),
                    degats,
                    dps_principale,
                ),
            ),
            None => (
                formule_secondes(c, &t, &n, &g, &format!("{g}*{t}/{n}")),
                calcul::secondes_en_repetant(touches, touches_usage, geste_de(c)),
            ),
        };
        valeurs.push(Val::F(formule, secondes));
        contre.push((touches, secondes));
    }
    let notes = [l.texte.as_str(), c.remarque.as_str()]
        .iter()
        .filter(|t| !t.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" — ");
    valeurs.push(Val::T(notes));
    valeurs.push(Val::t(format!("{} ({})", c.court(), c.variante)));
    let ecrite = Ecrite {
        ligne,
        classe: l.classe.clone(),
        jeu: l.jeu.clone(),
        nom: l.nom.clone(),
        sorte: l.sorte,
        niveau: l.niveau,
        degats,
        dps,
        dps_soutenu,
        contre,
    };
    (valeurs, ecrite)
}

// Colonnes de l'onglet Ennemis.
const E_TOUCHES: u16 = 6;
const E_BASE: u16 = 7;
const E_PA: u16 = 8;
const E_MULT: u16 = 9;
const E_DEG: u16 = 10;
const E_GESTE: u16 = 11;
const E_DPS: u16 = 12;
/// Première colonne des classes : deux colonnes par classe (touches, secondes).
pub const E_CLASSES: u16 = 13;

/// Où une attaque d'ennemi est écrite, et ce qu'elle vaut.
pub struct EnnemiEcrit {
    pub ligne: u32,
    pub role: String,
    pub principale: bool,
    /// Par classe : (touches pour tuer, secondes pour tuer).
    pub contre: Vec<(f64, f64)>,
}

pub fn ecrire_ennemis(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp) -> Result<Vec<EnnemiEcrit>> {
    feuille.set_name(ONGLET_ENNEMIS)?;
    let mut colonnes = vec![
        col("Ennemi", 11.0, Nb::Texte),
        col("PV", 8.0, Nb::Entier),
        col("Armure (dégâts retirés)", 11.0, Nb::Pct),
        col("Attaque", 20.0, Nb::Texte),
        col("Capacité (fichier)", 40.0, Nb::Texte),
        col("Type d'attaque", 15.0, Nb::Texte),
        col("Touches par usage", 9.0, Nb::Dec1),
        col("Dégâts de base (usage)", 10.0, Nb::Dec1),
        col("Puissance de l'arme", 10.0, Nb::Fois),
        col("Contre un joueur", 10.0, Nb::Fois),
        col("Dégâts par usage", 10.0, Nb::Dec1),
        col("Geste", 9.0, Nb::Secondes),
        col("Dégâts par seconde", 11.0, Nb::Dec1),
    ];
    let mut groupes = vec![(0, 2, "L'ennemi".to_string()), (3, 12, "Son attaque".to_string())];
    for (i, classe) in d.classes.iter().enumerate() {
        let c = E_CLASSES + 2 * i as u16;
        colonnes.push(col("Touches pour tuer", 9.0, Nb::Entier));
        colonnes.push(col("Secondes pour tuer", 9.0, Nb::Secondes));
        groupes.push((c, c + 1, format!("Contre : {}", classe.nom)));
    }
    let c_notes = E_CLASSES + 2 * d.classes.len() as u16;
    colonnes.push(col_longue("Ce que le calcul suppose", 60.0));
    groupes.push((c_notes, c_notes, "Notes".to_string()));
    let depart = style::bandeau(
        feuille,
        "Ennemis : ce qu'ils valent face à chaque classe",
        "Les ennemis frappent avec les attaques de base de leur arme, répétées sans pause. Dégâts sur un joueur en \
         armure de départ ; l'intelligence de l'ennemi (pauses, esquives, choix de l'attaque) n'est pas comptée.",
        accent::ENNEMIS,
        colonnes.len() as u16,
    )?;
    let premiere = depart + 2;
    let mut lignes = Vec::new();
    let mut ecrits = Vec::new();
    let mut roles = Vec::new();
    for ennemi in &d.ennemis {
        for (nom, id, principale) in [
            ("Attaque principale", &ennemi.arme.primaire, true),
            ("Attaque secondaire", &ennemi.arme.secondaire, false),
        ] {
            let Some(capacite) = id.as_deref().and_then(|id| Capacite::lire(id, ennemi.arme.stats).ok()) else {
                continue;
            };
            let ligne = premiere + lignes.len() as u32;
            let r = |colonne: u16| a1(ligne, colonne);
            let cle = format!("ennemi.{}", ennemi.role);
            let blesse = capacite.blesse();
            let puissance = hyp.v(&format!("{cle}.puissance"));
            let multiplicateur = hyp.v("combat.joueur");
            let (touches_usage, base) = (capacite.touches(), capacite.degats());
            let degats = calcul::degats(base, 1.0, puissance, multiplicateur);
            let mut valeurs = vec![
                Val::t(&ennemi.role),
                Val::F(format!("={}", hyp.r(&format!("{cle}.pv"))), hyp.v(&format!("{cle}.pv"))),
                Val::F(format!("={}", hyp.r(&format!("{cle}.armure"))), hyp.v(&format!("{cle}.armure"))),
                Val::t(nom),
                Val::t(format!("{} ({})", capacite.court(), capacite.variante)),
                Val::T(capacite.types()),
                Val::N(touches_usage),
                Val::N(base),
                Val::F(format!("={}", hyp.r(&format!("{cle}.puissance"))), puissance),
                Val::F(format!("={}", hyp.r("combat.joueur")), multiplicateur),
                Val::F(format!("={}*{}*{}", r(E_BASE), r(E_PA), r(E_MULT)), degats),
                Val::N(capacite.geste),
                Val::F(format!("={}/{}", r(E_DEG), r(E_GESTE)), degats / capacite.geste),
            ];
            let mut contre = Vec::new();
            for (i, classe) in d.classes.iter().enumerate() {
                let k = format!("classe.{}", classe.id);
                if !blesse {
                    valeurs.extend([Val::Vide, Val::Vide]);
                    contre.push((f64::NAN, f64::NAN));
                    continue;
                }
                let pv = hyp.v(&format!("{k}.pv_base")) * hyp.v(&format!("{k}.inne_pv"));
                let armure = hyp.v(&format!("{k}.armure"));
                let pv_ref = format!("({}*{})", hyp.r(&format!("{k}.pv_base")), hyp.r(&format!("{k}.inne_pv")));
                let armure_ref = hyp.r(&format!("{k}.armure"));
                let c_touches = E_CLASSES + 2 * i as u16;
                let touches = calcul::touches_pour_tuer(pv, degats / touches_usage, armure);
                valeurs.push(Val::F(
                    format!(
                        "=ROUNDUP(ROUND({pv_ref}/(({d}/{t})*(1-{armure_ref})),6),0)",
                        d = r(E_DEG),
                        t = r(E_TOUCHES)
                    ),
                    touches,
                ));
                let (t, n, g) = (r(c_touches), r(E_TOUCHES), r(E_GESTE));
                let secondes = calcul::secondes_en_repetant(touches, touches_usage, geste_de(&capacite));
                valeurs.push(Val::F(formule_secondes(&capacite, &t, &n, &g, &format!("{g}*{t}/{n}")), secondes));
                contre.push((touches, secondes));
            }
            valeurs.push(Val::t(&capacite.remarque));
            lignes.push(valeurs);
            roles.push(ennemi.role.clone());
            ecrits.push(EnnemiEcrit {
                ligne,
                role: ennemi.role.clone(),
                principale,
                contre,
            });
        }
    }
    let (entete, debut, fin) =
        style::tableau(feuille, depart, 0, accent::ENNEMIS, &groupes, &colonnes, &lignes, |i| roles[i].clone())?;
    feuille.autofilter(entete, 0, fin, colonnes.len() as u16 - 1)?;
    feuille.set_freeze_panes(debut, 1)?;
    barres(feuille, debut, fin, E_DEG, 0xCDB7F2)?;
    barres(feuille, debut, fin, E_DPS, 0xCDB7F2)?;
    for i in 0..d.classes.len() as u16 {
        // Ici le vert est du côté des grands nombres : le joueur tient longtemps.
        let regle = ConditionalFormat3ColorScale::new()
            .set_minimum_color(Color::RGB(0xF2A0A0))
            .set_midpoint_color(Color::RGB(0xFFE9A8))
            .set_maximum_color(Color::RGB(0x8FD19E));
        feuille.add_conditional_format(debut, E_CLASSES + 2 * i, fin, E_CLASSES + 2 * i, &regle)?;
    }

    // Les classes en face : leurs PV et leur armure, d'après les hypothèses.
    let mut ligne = fin + 3;
    feuille.merge_range(ligne, 0, ligne, 4, "Les classes en face", &style::intertitre(accent::ENNEMIS))?;
    ligne += 1;
    let colonnes_classes = [
        col("Classe", 11.0, Nb::Texte),
        col("PV de base", 8.0, Nb::Entier),
        col("Inné (PV)", 11.0, Nb::Fois),
        col("PV", 20.0, Nb::Entier),
        col("Armure de départ (dégâts retirés)", 40.0, Nb::Pct),
    ];
    let lignes_classes: Vec<Vec<Val>> = d
        .classes
        .iter()
        .enumerate()
        .map(|(i, classe)| {
            let k = format!("classe.{}", classe.id);
            let l = ligne + 1 + i as u32;
            vec![
                Val::t(&classe.nom),
                Val::F(format!("={}", hyp.r(&format!("{k}.pv_base"))), classe.pv_base),
                Val::F(format!("={}", hyp.r(&format!("{k}.inne_pv"))), classe.inne_pv),
                Val::F(format!("={}*{}", a1(l, 1), a1(l, 2)), classe.pv_base * classe.inne_pv),
                Val::F(format!("={}", hyp.r(&format!("{k}.armure"))), classe.armure),
            ]
        })
        .collect();
    // Les largeurs des colonnes restent celles du tableau du haut.
    let largeurs: Vec<f64> = colonnes.iter().take(5).map(|c| c.largeur).collect();
    style::tableau(feuille, ligne, 0, accent::ENNEMIS, &[], &colonnes_classes, &lignes_classes, |i| i.to_string())?;
    for (i, largeur) in largeurs.iter().enumerate() {
        feuille.set_column_width(i as u16, *largeur)?;
    }
    Ok(ecrits)
}

/// Les cases « et si » du combat.
pub fn hypotheses(hyp: &mut Hyp, d: &Donnees) {
    let g = "Dégâts des attaques (multiplicateurs)";
    let source = "assets/icebreaker/combat.ron";
    let c = &d.combat;
    let ou = |v: Option<f32>| net(v.unwrap_or(c.degats_contre_creatures));
    hyp.ajoute(g, "combat.joueur", "Contre un joueur (toute attaque)", net(c.degats_contre_joueurs), Nb::Fois, &format!("{source} : degats_contre_joueurs"));
    hyp.ajoute(g, "combat.melee", "Contre une créature : corps à corps", net(c.degats_contre_creatures), Nb::Fois, &format!("{source} : degats_contre_creatures"));
    hyp.ajoute(g, "combat.projectile", "Contre une créature : projectile (flèche, sort lancé)", ou(c.creatures_projectile), Nb::Fois, &format!("{source} : creatures_projectile"));
    hyp.ajoute(g, "combat.rayon", "Contre une créature : rayon, à chaque touche", ou(c.creatures_rayon), Nb::Fois, &format!("{source} : creatures_rayon"));
    hyp.ajoute(g, "combat.explosion", "Contre une créature : explosion", ou(c.creatures_explosion), Nb::Fois, &format!("{source} : creatures_explosion"));
    hyp.ajoute(g, "combat.onde", "Contre une créature : onde de choc", ou(c.creatures_onde), Nb::Fois, &format!("{source} : creatures_onde"));
    hyp.ajoute(g, "combat.autre", "Contre une créature : arc électrique, zone persistante", ou(c.creatures_autre), Nb::Fois, &format!("{source} : creatures_autre"));

    let g = "Classes des joueurs";
    for classe in &d.classes {
        let k = format!("classe.{}", classe.id);
        hyp.ajoute(g, &format!("{k}.pv_base"), &format!("{} : PV de base", classe.nom), classe.pv_base, Nb::Entier, "PV d'un humanoïde dans Veloren (common/src/comp/body/mod.rs, base_health)");
        hyp.ajoute(g, &format!("{k}.inne_pv"), &format!("{} : PV de l'inné", classe.nom), classe.inne_pv, Nb::Fois, &format!("assets/icebreaker/passifs/{}.ron : inne, MaxHealthModifier", classe.id));
        hyp.ajoute(g, &format!("{k}.armure"), &format!("{} : part des dégâts retirée par l'armure de départ", classe.nom), classe.armure, Nb::Pct, &format!("assets/icebreaker/classes/{}.ron (pièces d'armure) ; calcul du jeu : protection / (60 + protection)", classe.id));
    }

    // La vitesse de l'arme n'est pas une case : elle change les gestes et le rythme des rayons sorte par sorte,
    // c'est le jeu qui l'applique (colonne Geste).
    let g = "Armes de départ des jeux d'armes";
    for jeu in &d.jeux {
        let fichier = format!("assets/{}.ron", jeu.arme.id.replace('.', "/"));
        hyp.ajoute(g, &format!("jeu.{}.puissance", jeu.id), &format!("{} : puissance de l'arme", jeu.nom), jeu.arme.puissance, Nb::Fois, &format!("{fichier} : power (arme choisie dans assets/icebreaker/jeux_armes/{}.ron)", jeu.id));
    }

    let g = "Ennemis";
    for ennemi in &d.ennemis {
        let k = format!("ennemi.{}", ennemi.role);
        let fichier = format!("assets/icebreaker/{}", ennemi.fichier);
        let arme = format!("assets/{}.ron", ennemi.arme.id.replace('.', "/"));
        hyp.ajoute(g, &format!("{k}.pv"), &format!("{} : PV", ennemi.role), ennemi.pv, Nb::Entier, &format!("{fichier} : icebreaker_pv"));
        hyp.ajoute(g, &format!("{k}.armure"), &format!("{} : part des dégâts retirée par son armure", ennemi.role), ennemi.armure, Nb::Pct, &format!("{fichier} : pièces d'armure du loadout"));
        hyp.ajoute(g, &format!("{k}.puissance"), &format!("{} : puissance de son arme", ennemi.role), ennemi.arme.puissance, Nb::Fois, &format!("{arme} : power"));
        hyp.ajoute(g, &format!("{k}.experience"), &format!("{} : expérience donnée à sa mort", ennemi.role), ennemi.experience, Nb::Entier, "calcul de Veloren : note de combat × 20 (common/src/combat.rs, combat_rating), d'après ses PV, son armure et son arme");
    }

    let g = "Cibles d'équilibrage";
    let source = "docs/icebreaker/PLAN.md, L.1 (cible : 3 coups pour tuer, 3 à 4 pour mourir)";
    hyp.ajoute(g, "cible.tuer_min", "Coups de base pour tuer un ennemi ordinaire : au moins", 3.0, Nb::Entier, source);
    hyp.ajoute(g, "cible.tuer_max", "Coups de base pour tuer un ennemi ordinaire : au plus", 3.0, Nb::Entier, source);
    hyp.ajoute(g, "cible.mourir_min", "Coups d'un ennemi ordinaire pour tuer un joueur : au moins", 3.0, Nb::Entier, source);
    hyp.ajoute(g, "cible.mourir_max", "Coups d'un ennemi ordinaire pour tuer un joueur : au plus", 4.0, Nb::Entier, source);
}
