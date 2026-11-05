//! Les icônes de Veloren dans les tableaux (W.11) : quand la première cellule d'une ligne (ou la clé et la valeur
//! d'un réglage) parle d'une arme, d'un lieu, d'un état ou d'un objet connu, on met l'icône en pixel art devant.

use std::sync::OnceLock;

use regex::Regex;

/// Les règles, dans l'ordre : la première qui correspond gagne. (motif insensible à la casse, chemin sous `assets/`).
const REGLES: [(&str, &str); 24] = [
    (r"\bgnarling", "voxygen/element/ui/map/buttons/gnarling.png"),
    (r"\bgrotte|\bcave\b", "voxygen/element/ui/map/buttons/cave.png"),
    (r"\bgenere\b|donjon généré|\bdungeon\b|\bdonjon", "voxygen/element/ui/map/buttons/dungeon.png"),
    (r"\bvillage|\btown\b|\bcampagne\b", "voxygen/element/ui/map/buttons/town.png"),
    (r"\bguerrier|\bsword\b|\bépée", "voxygen/element/weapons/sword.png"),
    (r"\barcher|\bbow\b|\barc\b|\bflèche", "voxygen/element/weapons/bow.png"),
    (r"\bmage\b|\bstaff\b|\bbâton", "voxygen/element/weapons/staff.png"),
    (r"\bpickaxe|\bpioche|\bfilon|\bminerai", "voxygen/element/weapons/pickaxe.png"),
    (r"\bsceptre|\bclerc|\bdruide", "voxygen/element/weapons/sceptre.png"),
    (r"\bdagger|\bdague|\bassassin", "voxygen/element/weapons/daggers.png"),
    (r"\baxe\b|\bhache|\bberserker", "voxygen/element/weapons/axe.png"),
    (r"\bhammer\b|\bmarteau", "voxygen/element/weapons/hammer.png"),
    (r"\bmort\b|\bskull\b|\bboss\b|\bchef\b", "voxygen/element/ui/generic/icons/skull_2.png"),
    (r"\bsac\b|\bbag\b|\binventaire", "voxygen/element/items/item_bag_leather_large.png"),
    (r"\bpotion", "voxygen/element/de_buffs/buff_potion_0.png"),
    (r"\bfeu\b|\bbrûl|\bburning|\bflamme", "voxygen/element/de_buffs/debuff_burning_0.png"),
    (r"\bpoison|\bempoisonn", "voxygen/element/de_buffs/debuff_poisoned_0.png"),
    (r"\bsaign|\bbleed", "voxygen/element/de_buffs/debuff_bleed_0.png"),
    (r"\bglace\b|\bgel|\bfrozen|\bgivre", "voxygen/element/de_buffs/debuff_frozen_0.png"),
    (r"\beau\b|\bmouill|\bwet\b", "voxygen/element/de_buffs/debuff_wet_0.png"),
    (r"\bralenti|\bchilled", "voxygen/element/de_buffs/debuff_chilled.png"),
    (r"\bétourdi|\bstun|\bparried", "voxygen/element/de_buffs/debuff_offbalance_0.png"),
    (r"\bsoin|\bheal|\blumière|\bsoigne", "voxygen/element/de_buffs/buff_campfire_heal_0.png"),
    (r"\bvitesse|\bhaste|\bhâte", "voxygen/element/de_buffs/buff_haste_0.png"),
];

fn regles() -> &'static [(Regex, &'static str)] {
    static REGEX: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    REGEX.get_or_init(|| {
        REGLES
            .iter()
            .map(|(m, c)| (Regex::new(&format!("(?i){m}")).expect("motif d'icône"), *c))
            .collect()
    })
}

/// L'icône (chemin sous `assets/`) qui va avec un texte, s'il y en a une.
pub fn pour_texte(texte: &str) -> Option<&'static str> {
    regles().iter().find(|(m, _)| m.is_match(texte)).map(|(_, c)| *c)
}

/// La balise `<img>` d'une icône, `racine` étant le préfixe vers la racine du site.
pub fn balise(chemin: &str, racine: &str) -> String {
    format!("<img class=\"icone-table\" src=\"{racine}assets/{chemin}\" alt=\"\">")
}

/// Met l'icône devant la première cellule de chaque ligne des tableaux d'un HTML rendu ; renvoie le HTML et les
/// icônes utilisées (à copier dans le site).
pub fn annoter_tableaux(html: &str, racine: &str) -> (String, Vec<String>) {
    static CELLULE: OnceLock<Regex> = OnceLock::new();
    let cellule = CELLULE.get_or_init(|| Regex::new(r"(?s)<tr>\s*<td([^>]*)>(.*?)</td>").expect("regex de cellule"));
    let mut icones: Vec<String> = Vec::new();
    let sortie = cellule.replace_all(html, |c: &regex::Captures| {
        let contenu = &c[2];
        let texte = texte_brut(contenu);
        match pour_texte(&texte) {
            Some(chemin) => {
                if !icones.iter().any(|i| i == chemin) {
                    icones.push(chemin.to_string());
                }
                format!("<tr><td{}>{}{}</td>", &c[1], balise(chemin, racine), contenu)
            },
            None => c[0].to_string(),
        }
    });
    (sortie.into_owned(), icones)
}

fn texte_brut(html: &str) -> String {
    let mut texte = String::new();
    let mut dans_balise = false;
    for ch in html.chars() {
        match ch {
            '<' => dans_balise = true,
            '>' if dans_balise => dans_balise = false,
            _ if !dans_balise => texte.push(ch),
            _ => {},
        }
    }
    texte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icone_selon_le_texte() {
        assert_eq!(pour_texte("guerrier"), Some("voxygen/element/weapons/sword.png"));
        assert_eq!(pour_texte("Feu"), Some("voxygen/element/de_buffs/debuff_burning_0.png"));
        assert_eq!(pour_texte("(nom: \"Donjon généré\""), Some("voxygen/element/ui/map/buttons/dungeon.png"));
        assert_eq!(pour_texte("delai_a_terre: 30.0"), None);
        assert_eq!(pour_texte("mortier"), None);
        assert_eq!(pour_texte("Mort"), Some("voxygen/element/ui/generic/icons/skull_2.png"));
    }

    #[test]
    fn tableaux_annotes() {
        let html = "<table><tbody><tr><td>Feu</td><td>4 s</td></tr>\n<tr><td>Rien</td><td>brûle</td></tr></tbody></table>";
        let (sortie, icones) = annoter_tableaux(html, "../");
        assert!(sortie.contains("<tr><td><img class=\"icone-table\" src=\"../assets/voxygen/element/de_buffs/debuff_burning_0.png\" alt=\"\">Feu</td>"));
        assert!(sortie.contains("<tr><td>Rien</td><td>brûle</td>"));
        assert_eq!(icones, vec!["voxygen/element/de_buffs/debuff_burning_0.png"]);
    }
}
