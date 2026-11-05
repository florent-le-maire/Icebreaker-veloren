//! Rendu Markdown → HTML, avec les liens `.md` réécrits vers les pages du site.

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

pub struct Rendu {
    /// Texte du premier titre de niveau 1, s'il y en a un.
    pub titre: Option<String>,
    pub html: String,
}

/// Sépare un en-tête TOML (`+++` … `+++` en tête de fichier) du corps. Sans en-tête, renvoie tout le texte.
pub fn separer_entete(texte: &str) -> (Option<&str>, &str) {
    let texte_sans_bom = texte.trim_start_matches('\u{feff}');
    let Some(reste) = texte_sans_bom.strip_prefix("+++") else {
        return (None, texte_sans_bom);
    };
    let reste = reste.trim_start_matches(['\r', '\n']);
    match reste.find("\n+++") {
        Some(fin) => {
            let entete = reste[..fin].trim_end_matches('\r');
            let corps = reste[fin + 4..].trim_start_matches(['\r', '\n']);
            (Some(entete), corps)
        },
        None => (None, texte_sans_bom),
    }
}

/// Réécrit la cible d'un lien : `fichier.md` (relatif, éventuellement avec `#ancre`) → `fichier.html` ;
/// `README.md` → `index.html`. Les liens absolus et les autres fichiers ne changent pas.
pub fn reecrire_lien(cible: &str) -> String {
    if cible.contains("://") || cible.starts_with('#') || cible.starts_with('/') {
        return cible.to_string();
    }
    let (chemin, ancre) = match cible.find('#') {
        Some(i) => (&cible[..i], &cible[i..]),
        None => (cible, ""),
    };
    let Some(sans_md) = chemin.strip_suffix(".md") else {
        return cible.to_string();
    };
    let (dossier, nom) = match sans_md.rfind('/') {
        Some(i) => (&sans_md[..=i], &sans_md[i + 1..]),
        None => ("", sans_md),
    };
    if nom == "README" {
        format!("{dossier}index.html{ancre}")
    } else {
        format!("{dossier}{nom}.html{ancre}")
    }
}

pub fn rendre(corps: &str, _source: &str) -> Rendu {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let mut titre: Option<String> = None;
    let mut dans_h1 = false;
    let mut evenements = Vec::new();
    for evenement in Parser::new_ext(corps, options) {
        match &evenement {
            Event::Start(Tag::Heading { level: HeadingLevel::H1, .. }) if titre.is_none() => {
                dans_h1 = true;
                titre = Some(String::new());
            },
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if dans_h1 => dans_h1 = false,
            Event::Text(t) | Event::Code(t) if dans_h1 => {
                if let Some(titre) = titre.as_mut() {
                    titre.push_str(t);
                }
            },
            _ => {},
        }
        let evenement = match evenement {
            Event::Start(Tag::Link { link_type, dest_url, title, id }) => Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::from(reecrire_lien(&dest_url)),
                title,
                id,
            }),
            autre => autre,
        };
        evenements.push(evenement);
    }
    let mut sortie = String::new();
    html::push_html(&mut sortie, evenements.into_iter());
    Rendu { titre: titre.map(|t| t.trim().to_string()), html: sortie }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titre_et_tableau() {
        let rendu = rendre("# Mon `titre`\n\n| a | b |\n|---|---|\n| 1 | 2 |\n", "x.md");
        assert_eq!(rendu.titre.as_deref(), Some("Mon titre"));
        assert!(rendu.html.contains("<table>"));
        assert!(rendu.html.contains("<td>2</td>"));
    }

    #[test]
    fn liens_md_vers_html() {
        assert_eq!(reecrire_lien("PLAN.md"), "PLAN.html");
        assert_eq!(reecrire_lien("design/combat.md#degats"), "design/combat.html#degats");
        assert_eq!(reecrire_lien("../README.md"), "../index.html");
        assert_eq!(reecrire_lien("https://exemple.org/a.md"), "https://exemple.org/a.md");
        assert_eq!(reecrire_lien("ames-idees.xlsx"), "ames-idees.xlsx");
        let rendu = rendre("[plan](PLAN.md)", "x.md");
        assert!(rendu.html.contains("href=\"PLAN.html\""));
    }

    #[test]
    fn entete_toml() {
        let (entete, corps) = separer_entete("+++\ndesign = \"combat.md\"\n+++\n\n# Titre\n");
        assert_eq!(entete, Some("design = \"combat.md\""));
        assert_eq!(corps, "# Titre\n");
        let (entete, corps) = separer_entete("# Sans en-tête\n");
        assert!(entete.is_none());
        assert_eq!(corps, "# Sans en-tête\n");
        let (entete, _) = separer_entete("+++\r\na = 1\r\n+++\r\ncorps");
        assert_eq!(entete, Some("a = 1"));
    }

    #[test]
    fn cases_a_cocher() {
        let rendu = rendre("- [x] **T.1 Fait** — ok 63f57ce830\n- [ ] **T.2 Reste**\n", "x.md");
        assert!(rendu.html.contains("type=\"checkbox\""));
        assert!(rendu.html.contains("checked"));
    }
}
