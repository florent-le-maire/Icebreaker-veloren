//! Icebreaker : l'onglet « Expéditions » — ce qu'un donjon généré contient (compté sur de vrais donjons), ce
//! qu'il rapporte et ce qu'il demande ; les minerais de la grotte.

use crate::{
    donjon::{SORTES, densite_en_clair, taille_en_clair},
    donnees::{Donnees, Ennemi, net},
    hyp::Hyp,
    style::{self, Nb, Val, a1, accent, col},
};
use anyhow::Result;
use common::icebreaker_minerais::minerais;
use rust_xlsxwriter::{Color, ConditionalFormatDataBar, Format, Worksheet};
use world::icebreaker::grotte;

pub const ONGLET: &str = "Expéditions";

pub fn hypotheses(hyp: &mut Hyp, d: &Donnees) {
    let g = "Repères d'une expédition";
    hyp.ajoute(g, "ref.joueurs", "Joueurs dans une expédition", 2.0, Nb::Entier, "hypothèse : la campagne se joue à deux (docs/icebreaker/PLAN-PLAYTEST.md, § 8)");
    // La moyenne des attaques principales des jeux d'armes, en rafale, sans l'armure des ennemis.
    let principales: Vec<f64> = d
        .lignes
        .iter()
        .filter(|l| l.niveau == 0 && l.nom == "Attaque principale")
        .filter_map(|l| {
            let arme = &d.jeux.iter().find(|j| j.id == l.jeu)?.arme;
            let c = &l.capacite;
            let multiplicateur = hyp.v(c.source.cle());
            (c.degats > 0.0).then(|| {
                crate::calcul::degats(c.degats, 1.0, arme.puissance, multiplicateur)
                    / crate::calcul::geste(c.duree, arme.vitesse, c.duree_fixe)
            })
        })
        .collect();
    let moyenne = principales.iter().sum::<f64>() / principales.len().max(1) as f64;
    hyp.ajoute(g, "ref.dps", "Dégâts par seconde d'un joueur qui frappe", (moyenne * 10.0).round() / 10.0, Nb::Dec1, "hypothèse : moyenne des attaques principales des jeux d'armes (onglet Combat), sans l'armure des ennemis (comptée à part dans l'onglet Expéditions)");
}

/// Ce que l'onglet donne aux autres.
pub struct Ecrit {
    /// La ligne de l'expédition type, et la colonne de son or.
    pub reference: u32,
    pub c_or: u16,
    pub c_creatures: u16,
    pub c_pv: u16,
    pub c_minutes: u16,
    pub c_points: u16,
    /// Valeurs de l'expédition type : (créatures, PV, or, minutes, points d'arme).
    pub valeurs: (f64, f64, f64, f64, f64),
}

/// Les PV et l'expérience d'une fiche : une case d'hypothèse si c'est un de nos ennemis, sinon ses nombres.
type Case = (String, f64);

fn fiche(id: &str, d: &Donnees, hyp: &Hyp) -> Option<[Case; 3]> {
    if let Some(ennemi) = d.ennemis.iter().find(|e| id == format!("icebreaker.ennemis.{}", e.role)) {
        let case = |quoi: &str| {
            let cle = format!("ennemi.{}.{quoi}", ennemi.role);
            (hyp.r(&cle), hyp.v(&cle))
        };
        return Some([case("pv"), case("experience"), case("armure")]);
    }
    let autre = Ennemi::fiche(id, id).ok()?;
    let nombre = |x: f64| (x.to_string(), x);
    Some([nombre(autre.pv), nombre(autre.experience), nombre(autre.armure)])
}

pub fn ecrire(feuille: &mut Worksheet, d: &Donnees, hyp: &Hyp) -> Result<Ecrit> {
    feuille.set_name(ONGLET)?;
    let donjon = &d.donjon;
    let paliers = donjon.paliers as u16;
    let graines = donjon.cas.iter().map(|c| c.graines).max().unwrap_or(0);
    // Colonnes du tableau des cas.
    let c_sortes: u16 = 5;
    let c_total = c_sortes + SORTES.len() as u16;
    let c_creatures_palier = c_total + 1;
    let c_coffres = c_creatures_palier + paliers;
    let c_pv = c_coffres + paliers;
    let c_dur = c_pv + 1;
    let c_or = c_dur + 1;
    let c_or_joueur = c_or + 1;
    let c_xp = c_or_joueur + 1;
    let c_points = c_xp + 1;
    let c_minutes = c_points + 1;
    let colonnes_en_tout = c_minutes + 1;

    let mut ligne = style::bandeau(
        feuille,
        "Expéditions : ce qu'un donjon contient, rapporte et demande",
        &format!(
            "Moyennes sur {graines} donjons générés par cas avec le générateur du jeu (objectif « tuer le chef », \
             faction « {} »). L'or compte les coffres et ce que lâchent les créatures, pas les caisses du décor ni \
             les objets à revendre.",
            donjon.faction
        ),
        accent::EXPEDITIONS,
        colonnes_en_tout,
    )?;
    let titre = style::intertitre(accent::EXPEDITIONS);

    // 1. Qui habite le donjon.
    feuille.merge_range(ligne, 0, ligne, 9, "Qui habite le donjon", &titre)?;
    ligne += 1;
    let colonnes = [
        col("Sorte", 13.0, Nb::Texte),
        col("PV moyens", 9.0, Nb::Dec1),
        col("Expérience moyenne", 11.0, Nb::Dec1),
        col("PV, armure comprise", 14.0, Nb::Dec1),
        col("Fiches (une au hasard)", 8.0, Nb::Texte),
    ];
    let mut lignes = Vec::new();
    // Par sorte : les cases des PV, de l'expérience et des PV armure comprise, avec leurs valeurs.
    let mut habitants: Vec<[Case; 3]> = Vec::new();
    for (i, (_, nom)) in SORTES.iter().enumerate() {
        let l = ligne + 1 + i as u32;
        let fiches: Vec<_> = donjon.habitants[i].iter().filter_map(|id| fiche(id, d, hyp)).collect();
        let n = fiches.len().max(1) as f64;
        let moyenne = |choix: &dyn Fn(&[Case; 3]) -> Case| {
            if fiches.is_empty() {
                return Val::N(0.0);
            }
            let (refs, valeurs): (Vec<String>, Vec<f64>) = fiches.iter().map(choix).unzip();
            Val::F(format!("=({})/{n}", refs.join("+")), valeurs.iter().sum::<f64>() / n)
        };
        let pv = moyenne(&|f| f[0].clone());
        let xp = moyenne(&|f| f[1].clone());
        // Ce qu'il faut de dégâts bruts pour en venir à bout : ses PV, divisés par ce que l'armure laisse passer.
        let dur = moyenne(&|f| (format!("{}/(1-{})", f[0].0, f[2].0), f[0].1 / (1.0 - f[2].1)));
        let valeur = |v: &Val| if let Val::F(_, x) | Val::N(x) = v { *x } else { 0.0 };
        let case = |colonne: char, v: &Val| (format!("${colonne}${}", l + 1), valeur(v));
        habitants.push([case('B', &pv), case('C', &xp), case('D', &dur)]);
        let noms: Vec<&str> = donjon.habitants[i].iter().map(|id| id.rsplit('.').next().unwrap_or(id)).collect();
        lignes.push(vec![Val::t(*nom), pv, xp, dur, Val::T(if noms.is_empty() { "aucune".into() } else { noms.join(", ") })]);
    }
    let (_, _, fin) = style::tableau(feuille, ligne, 0, accent::EXPEDITIONS, &[], &colonnes, &lignes, |i| i.to_string())?;

    // 2. Les cas.
    ligne = fin + 3;
    feuille.merge_range(ligne, 0, ligne, 9, "Ce qu'un donjon contient, rapporte et demande", &titre)?;
    ligne += 1;
    let mut colonnes = vec![
        col("Taille", 13.0, Nb::Texte),
        col("Menace", 9.0, Nb::Entier),
        col("Densité d'ennemis", 11.0, Nb::Texte),
        col("Repère", 11.0, Nb::Texte),
        col("Salles", 8.0, Nb::Dec1),
    ];
    colonnes.extend(SORTES.iter().map(|(_, nom)| col(nom, 9.0, Nb::Dec1)));
    colonnes.push(col("Créatures en tout", 10.0, Nb::Dec1));
    colonnes.extend((1..=paliers).map(|p| col(&format!("Créatures, palier {p}"), 10.0, Nb::Dec1)));
    colonnes.extend((1..=paliers).map(|p| col(&format!("Coffres, palier {p}"), 9.0, Nb::Dec1)));
    colonnes.extend([
        col("PV à abattre", 10.0, Nb::Entier),
        col("PV, armure comprise", 10.0, Nb::Entier),
        col("Or attendu", 10.0, Nb::Entier),
        col("Or par joueur", 10.0, Nb::Entier),
        col("Expérience en tout", 11.0, Nb::Entier),
        col("Points d'arme gagnés, seul", 11.0, Nb::Entier),
        col("Minutes de frappe du groupe", 11.0, Nb::Dec1),
    ]);
    let groupes = [
        (0, 4, "Le donjon".to_string()),
        (c_sortes, c_total, "Créatures".to_string()),
        (c_creatures_palier, c_coffres + paliers - 1, "Par palier de butin".to_string()),
        (c_pv, c_minutes, "Ce qu'il demande et rapporte".to_string()),
    ];
    let premiere = ligne + 2;
    let (joueurs, dps) = (hyp.v("ref.joueurs"), hyp.v("ref.dps"));
    let (premier, pas, par_xp) =
        (hyp.v("salle.premier_point"), hyp.v("salle.pas_point"), hyp.v("salle.maitrise_par_experience"));
    let mut reference = premiere;
    let mut valeurs_reference = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut lignes = Vec::new();
    for (i, cas) in donjon.cas.iter().enumerate() {
        let l = premiere + i as u32;
        let r = |colonne: u16| a1(l, colonne);
        let mut valeurs = vec![
            Val::t(taille_en_clair(&cas.taille)),
            Val::N(f64::from(cas.menace)),
            Val::t(densite_en_clair(&cas.densite)),
            if cas.reference { Val::t("expédition type") } else { Val::Vide },
            Val::N(cas.salles),
        ];
        valeurs.extend(cas.par_sorte.iter().map(|n| Val::N(*n)));
        let total: f64 = cas.par_sorte.iter().sum();
        valeurs.push(Val::F(format!("=SUM({}:{})", r(c_sortes), r(c_total - 1)), total));
        valeurs.extend(cas.creatures_palier.iter().map(|n| Val::N(*n)));
        valeurs.extend(cas.coffres_palier.iter().map(|n| Val::N(*n)));
        // PV et expérience : chaque sorte × ce qu'elle vaut.
        let somme = |quoi: usize| {
            let (formule, valeur): (Vec<String>, Vec<f64>) = habitants
                .iter()
                .enumerate()
                .map(|(k, h)| {
                    let (case, v) = h[quoi].clone();
                    (format!("{}*{case}", r(c_sortes + k as u16)), cas.par_sorte[k] * v)
                })
                .unzip();
            (format!("={}", formule.join("+")), valeur.iter().sum::<f64>())
        };
        let (f_pv, pv) = somme(0);
        let (f_xp, xp) = somme(1);
        let (f_dur, dur) = somme(2);
        // Or : coffres et créatures par palier, chef, boss.
        let mut f_or = Vec::new();
        let mut or = 0.0;
        for p in 0..paliers {
            for (colonne, cle, n) in [
                (c_coffres + p, format!("butin.coffre_{}.or", p + 1), cas.coffres_palier[p as usize]),
                (c_creatures_palier + p, format!("butin.creature_{}.or", p + 1), cas.creatures_palier[p as usize]),
            ] {
                f_or.push(format!("{}*{}", r(colonne), hyp.r(&cle)));
                or += n * hyp.v(&cle);
            }
        }
        for (k, cle) in [(4, "butin.chef.or"), (5, "butin.boss.or")] {
            f_or.push(format!("{}*{}", r(c_sortes + k), hyp.r(cle)));
            or += cas.par_sorte[k as usize] * hyp.v(cle);
        }
        let maitrise = xp * par_xp;
        let racine = 2.0 * premier - pas;
        let points = ((-racine + (racine * racine + 8.0 * pas * maitrise).sqrt()) / (2.0 * pas)).floor();
        let minutes = dur / (dps * joueurs) / 60.0;
        valeurs.extend([
            Val::F(f_pv, pv),
            Val::F(f_dur, dur),
            Val::F(format!("={}", f_or.join("+")), or),
            Val::F(format!("={}/{}", r(c_or), hyp.r("ref.joueurs")), or / joueurs),
            Val::F(f_xp, xp),
            Val::F(
                format!(
                    "=FLOOR((-(2*{p}-{s})+SQRT((2*{p}-{s})^2+8*{s}*{x}*{m}))/(2*{s}),1)",
                    p = hyp.r("salle.premier_point"),
                    s = hyp.r("salle.pas_point"),
                    x = r(c_xp),
                    m = hyp.r("salle.maitrise_par_experience")
                ),
                points,
            ),
            Val::F(format!("={}/({}*{})/60", r(c_dur), hyp.r("ref.dps"), hyp.r("ref.joueurs")), minutes),
        ]);
        if cas.reference {
            reference = l;
            valeurs_reference = (total, pv, or, minutes, points);
        }
        lignes.push(valeurs);
    }
    let (entete, debut, fin) = style::tableau(feuille, ligne, 0, accent::EXPEDITIONS, &groupes, &colonnes, &lignes, |i| {
        donjon.cas[i].taille.clone()
    })?;
    debug_assert_eq!(debut, premiere);
    feuille.autofilter(entete, 0, fin, colonnes.len() as u16 - 1)?;
    for (colonne, couleur) in [(c_total, 0x9FD8B0), (c_pv, 0xF2A7A7), (c_or, 0xEBCB6B), (c_minutes, 0xA9C4F5)] {
        let barres = ConditionalFormatDataBar::new().set_fill_color(Color::RGB(couleur)).set_solid_fill(true);
        feuille.add_conditional_format(debut, colonne, fin, colonne, &barres)?;
    }
    // L'expédition type ressort.
    let gras = Format::new().set_bold();
    let type_ = rust_xlsxwriter::ConditionalFormatFormula::new()
        .set_rule(format!("=${}=\"expédition type\"", a1(debut, 3)).as_str())
        .set_format(&gras);
    feuille.add_conditional_format(debut, 0, fin, colonnes.len() as u16 - 1, &type_)?;

    // 3. La grotte : ses habitants et ses minerais.
    ligne = fin + 3;
    feuille.merge_range(ligne, 0, ligne, 9, "La grotte : minerais par niveau", &titre)?;
    ligne += 1;
    let liste = minerais().liste();
    let niveaux = liste.iter().map(|(_, m)| m.poids.len()).max().unwrap_or(1);
    let mut colonnes = vec![
        col("Minerai", 13.0, Nb::Texte),
        col("Coups de pioche", 9.0, Nb::Entier),
        col("Énergie par coup", 11.0, Nb::Dec1),
        col("Quantité au moins", 11.0, Nb::Entier),
        col("Quantité au plus", 8.0, Nb::Entier),
    ];
    colonnes.extend((1..=niveaux).map(|n| col(&format!("Chance d'un filon, niveau {n}"), 9.0, Nb::Pct)));
    colonnes.push(col("Chance dans la salle au minerai", 9.0, Nb::Pct));
    let poids = |m: &common::icebreaker_minerais::Minerai, n: usize| {
        f64::from(*m.poids.get(n).or(m.poids.last()).unwrap_or(&0.0))
    };
    let totaux: Vec<f64> = (0..niveaux).map(|n| liste.iter().map(|(_, m)| poids(m, n)).sum()).collect();
    let total_salle: f64 = liste.iter().map(|(_, m)| f64::from(m.salle)).sum();
    let part = |x: f64, total: f64| if total > 0.0 { Val::N((x / total * 1000.0).round() / 1000.0) } else { Val::Vide };
    let lignes: Vec<Vec<Val>> = liste
        .iter()
        .map(|(nom, m)| {
            let mut valeurs = vec![
                Val::t(nom),
                Val::N(f64::from(m.coups)),
                Val::N(net(m.energie)),
                Val::N(f64::from(m.quantite.0)),
                Val::N(f64::from(m.quantite.1)),
            ];
            valeurs.extend((0..niveaux).map(|n| part(poids(m, n), totaux[n])));
            valeurs.push(part(f64::from(m.salle), total_salle));
            valeurs
        })
        .collect();
    let (_, debut, fin) = style::tableau(feuille, ligne, 0, accent::EXPEDITIONS, &[], &colonnes, &lignes, |i| i.to_string())?;
    let echelle = rust_xlsxwriter::ConditionalFormat2ColorScale::new()
        .set_minimum_color(Color::White)
        .set_maximum_color(Color::RGB(0x8FD19E));
    feuille.add_conditional_format(debut, 5, fin, 5 + niveaux as u16, &echelle)?;
    ligne = fin + 2;
    if let Ok(reglages) = grotte::Settings::load() {
        let p = &reglages.population;
        let phrase = format!(
            "Habitants de la grotte (assets/icebreaker/grottes/grotte.ron) : {} à {} créatures par poche à l'entrée, \
             {} de plus par niveau ; dans une niche, {:.0} % de chances d'une créature et {:.0} % d'un coffre ; {:.0} % \
             de monstres solitaires ; {} niveaux de minerai.",
            p.par_poche.0,
            p.par_poche.1,
            p.par_niveau,
            p.trous * 100.0,
            p.coffres * 100.0,
            p.part_solitaires * 100.0,
            reglages.filons.niveaux,
        );
        feuille.merge_range(ligne, 0, ligne + 1, c_total, &phrase, &style::note())?;
    }
    // Les largeurs du tableau des cas priment.
    for (colonne, largeur) in [(0, 13.0), (1, 9.0), (2, 11.0), (3, 15.0), (4, 8.0)] {
        feuille.set_column_width(colonne, largeur)?;
    }
    feuille.set_freeze_panes(3, 1)?;

    Ok(Ecrit {
        reference,
        c_or,
        c_creatures: c_total,
        c_pv,
        c_minutes,
        c_points,
        valeurs: valeurs_reference,
    })
}
