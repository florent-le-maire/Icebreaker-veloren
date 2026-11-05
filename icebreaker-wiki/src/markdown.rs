//! Rendu Markdown → HTML, avec les liens `.md` réécrits vers les pages du site et les images en figures.

use std::path::Path;

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

pub struct Rendu {
    /// Texte du premier titre de niveau 1, s'il y en a un.
    pub titre: Option<String>,
    pub html: String,
    /// Images relatives trouvées sur le disque (chemin tel qu'écrit dans le Markdown), à copier dans le site.
    pub images: Vec<String>,
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

fn echapper(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Une image en figure avec sa légende (le texte alternatif), cliquable pour l'agrandir.
fn figure(cible: &str, legende: &str) -> String {
    format!(
        "<figure class=\"capture\"><a class=\"zoom\" href=\"{0}\"><img src=\"{0}\" alt=\"{1}\" loading=\"lazy\"></a>\
<figcaption>{1}</figcaption></figure>",
        echapper(cible),
        echapper(legende)
    )
}

/// L'encadré qui remplace une image absente : ce qu'il faut photographier en jeu et où déposer le fichier.
fn capture_a_prendre(cible: &str, legende: &str, dossier_doc: &str) -> String {
    format!(
        "<div class=\"capture-a-prendre\"><span class=\"etiquette etiquette-chaud\">Capture à prendre</span> \
<span class=\"capture-legende\">{}</span><span class=\"capture-ou\">à déposer dans <code>{}{}</code></span></div>",
        echapper(legende),
        echapper(dossier_doc),
        echapper(cible)
    )
}

/// Rend un corps Markdown ; `source` est le chemin du fichier relatif à `docs/icebreaker/`, `dossier` le dossier
/// absolu qui le contient (pour vérifier que les images existent ; `None` : on ne vérifie pas).
pub fn rendre_images(corps: &str, source: &str, dossier: Option<&Path>) -> Rendu {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    let dossier_doc = match source.rfind('/') {
        Some(i) => format!("docs/icebreaker/{}", &source[..=i]),
        None => "docs/icebreaker/".to_string(),
    };

    let mut titre: Option<String> = None;
    let mut dans_h1 = false;
    let mut image: Option<(String, String)> = None;
    let mut images = Vec::new();
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
            Event::Start(Tag::Image { dest_url, .. }) => {
                image = Some((dest_url.to_string(), String::new()));
                continue;
            },
            Event::Text(t) | Event::Code(t) if image.is_some() => {
                if let Some((_, legende)) = image.as_mut() {
                    legende.push_str(&t);
                }
                continue;
            },
            Event::End(TagEnd::Image) => {
                let Some((cible, legende)) = image.take() else { continue };
                let relative = !cible.contains("://") && !cible.starts_with('/');
                let html = match dossier {
                    Some(d) if relative && !d.join(&cible).is_file() => capture_a_prendre(&cible, &legende, &dossier_doc),
                    _ => {
                        if relative && dossier.is_some() {
                            images.push(cible.clone());
                        }
                        figure(&cible, &legende)
                    },
                };
                Event::Html(CowStr::from(html))
            },
            autre => autre,
        };
        evenements.push(evenement);
    }
    let mut sortie = String::new();
    html::push_html(&mut sortie, evenements.into_iter());
    Rendu { titre: titre.map(|t| t.trim().to_string()), html: sortie, images }
}

/// Rendu sans vérification des images (elles sont laissées telles quelles).
pub fn rendre(corps: &str, source: &str) -> Rendu {
    rendre_images(corps, source, None)
}

/// Rendu d'un bout de texte en ligne : comme `rendre`, sans le `<p>` autour quand il n'y a qu'un paragraphe.
pub fn rendre_inline(texte: &str) -> String {
    let html = rendre(texte, "").html;
    let html = html.trim();
    match html.strip_prefix("<p>").and_then(|h| h.strip_suffix("</p>")) {
        Some(interieur) if !interieur.contains("<p>") => interieur.to_string(),
        _ => html.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendu_en_ligne() {
        assert_eq!(rendre_inline("du `code` et *gras*"), "du <code>code</code> et <em>gras</em>");
        assert!(rendre_inline("un\n\ndeux").starts_with("<p>un</p>"));
    }

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

    #[test]
    fn images_presentes_et_absentes() {
        let dossier = std::env::temp_dir().join(format!("icebreaker-wiki-images-{}", std::process::id()));
        std::fs::create_dir_all(dossier.join("images")).unwrap();
        std::fs::write(dossier.join("images/la.png"), b"png").unwrap();
        let md = "![Un allié à terre](images/la.png)\n\n![La charrette & ses recrues](images/pas-la.png)\n";
        let rendu = rendre_images(md, "wiki/mort.md", Some(&dossier));
        assert_eq!(rendu.images, vec!["images/la.png"]);
        assert!(rendu.html.contains("<figure class=\"capture\"><a class=\"zoom\" href=\"images/la.png\">"));
        assert!(rendu.html.contains("<figcaption>Un allié à terre</figcaption>"));
        assert!(rendu.html.contains("Capture à prendre"));
        assert!(rendu.html.contains("La charrette &amp; ses recrues"));
        assert!(rendu.html.contains("<code>docs/icebreaker/wiki/images/pas-la.png</code>"));
        // Sans dossier, l'image est laissée telle quelle et n'est pas à copier.
        let rendu = rendre(md, "wiki/mort.md");
        assert!(rendu.images.is_empty());
        assert_eq!(rendu.html.matches("<figure").count(), 2);
        std::fs::remove_dir_all(&dossier).unwrap();
    }
}
