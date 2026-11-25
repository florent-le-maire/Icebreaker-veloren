//! Icebreaker : le résumé en texte du classeur (`--resume [filtre]`), pour lire l'équilibrage sans tableur :
//! les deux matrices de la Synthèse, l'expédition type, les achats, et les lignes de Combat dont le nom
//! contient le filtre.

use crate::{
    combat::{Ecrite, EnnemiEcrit},
    donnees::{Donnees, Sorte},
    economie, expeditions,
    hyp::Hyp,
};

fn nombre(x: f64) -> String { if x.is_finite() { format!("{x:.0}") } else { "—".into() } }

pub fn afficher(
    d: &Donnees,
    hyp: &Hyp,
    competences: &[Ecrite],
    ennemis: &[EnnemiEcrit],
    expedition: &expeditions::Ecrit,
    filtre: Option<&str>,
) {
    let roles: Vec<&str> = d.ennemis.iter().map(|e| e.role.as_str()).collect();
    println!(
        "\nTuer : touches à l'attaque principale (cible : {} à {} contre un ennemi ordinaire)",
        hyp.v("cible.tuer_min"),
        hyp.v("cible.tuer_max")
    );
    println!("  {:<22}{}", "jeu d'armes", roles.iter().map(|r| format!("{r:>10}")).collect::<String>());
    for e in competences.iter().filter(|e| e.sorte == Sorte::Base && e.nom == "Attaque principale") {
        let nom = d.jeux.iter().find(|j| j.id == e.jeu).map_or(e.jeu.as_str(), |j| j.nom.as_str());
        println!("  {nom:<22}{}", e.contre.iter().map(|c| format!("{:>10}", nombre(c.0))).collect::<String>());
    }
    println!(
        "\nMourir : touches d'un ennemi, attaque principale (cible : {} à {})",
        hyp.v("cible.mourir_min"),
        hyp.v("cible.mourir_max")
    );
    println!("  {:<22}{}", "ennemi", d.classes.iter().map(|c| format!("{:>10}", c.nom)).collect::<String>());
    for e in ennemis.iter().filter(|e| e.principale) {
        println!("  {:<22}{}", e.role, e.contre.iter().map(|c| format!("{:>10}", nombre(c.0))).collect::<String>());
    }
    let (creatures, pv, or, minutes, points) = expedition.valeurs;
    println!(
        "\nExpédition type (donjon moyen) : {creatures:.0} créatures, {pv:.0} PV à abattre, {or:.0} or, {minutes:.1} \
         min de frappe, {points:.0} points d'arme gagnés seul"
    );
    println!("Achats, en expéditions type :");
    for (cle, nom) in economie::ACHATS {
        println!("  {nom:<24}{:>6.0} or{:>8.2}", hyp.v(cle), hyp.v(cle) / or);
    }
    let Some(filtre) = filtre.map(str::to_lowercase) else { return };
    println!("\nCombat, lignes contenant « {filtre} » (touches / secondes pour tuer : {}) :", roles.join(", "));
    for e in competences.iter().filter(|e| format!("{} {}", e.jeu, e.nom).to_lowercase().contains(&filtre)) {
        let niveau = if e.niveau == 0 { "base".to_string() } else { format!("niv {}", e.niveau) };
        let contre: String = e.contre.iter().map(|c| format!("{:>5} /{:>6.1} s", nombre(c.0), c.1)).collect();
        println!("  {:<14}{:<22}{niveau:<7} dégâts {:>6.1}  par seconde {:>6.1} |{contre}", e.jeu, e.nom, e.degats, e.dps);
    }
}
