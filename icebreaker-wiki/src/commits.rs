//! Les commits du dépôt, lus avec `git log`, pour relier chaque tâche à ce qui a été fait et testé.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Commit {
    pub hash: String,
    /// Les 10 premiers caractères, comme dans les plans.
    pub court: String,
    /// Date `AAAA-MM-JJ`.
    pub date: String,
    pub sujet: String,
    /// Le corps sans le paragraphe « Testé » ni les lignes `Co-Authored-By`.
    pub corps: String,
    /// Le paragraphe qui commence par « Testé », s'il y en a un.
    pub teste: Option<String>,
    /// Vrai pour les commits de la branche courante (faux pour ceux trouvés seulement dans `ancien-master`
    /// ou une autre branche).
    pub principal: bool,
}

fn journal(racine: &Path, reference: &str) -> Option<HashMap<String, Commit>> {
    let sortie = Command::new("git")
        .args(["log", reference, "--format=%H%x1f%cs%x1f%s%x1f%b%x1e"])
        .current_dir(racine)
        .output()
        .ok()?;
    sortie
        .status
        .success()
        .then(|| analyser(&String::from_utf8_lossy(&sortie.stdout)))
}

/// Tous les commits, par hash : ceux de la branche courante (`principal`), puis ceux de toutes les références
/// (l'historique d'avant la réduction du dépôt est gardé sous l'étiquette `ancien-master`). Dépôt sans git ou
/// `git` absent : table vide (le site se génère sans les liens).
pub fn lire(racine: &Path) -> HashMap<String, Commit> {
    let Some(mut commits) = journal(racine, "HEAD") else {
        eprintln!("avertissement : git log impossible, pas de liens vers les commits");
        return HashMap::new();
    };
    for c in commits.values_mut() {
        c.principal = true;
    }
    if let Some(tous) = journal(racine, "--all") {
        for (hash, c) in tous {
            commits.entry(hash).or_insert(c);
        }
    }
    commits
}

/// Nos commits sur la branche courante : sujet « Icebreaker … », sans le commit de base de Veloren.
pub fn notres(commits: &HashMap<String, Commit>) -> Vec<&Commit> {
    commits
        .values()
        .filter(|c| c.principal && c.sujet.starts_with("Icebreaker ") && !c.sujet.contains("(base d'Icebreaker)"))
        .collect()
}

/// L'adresse web du dépôt `origin` (GitHub), sans `.git`, pour faire des liens vers les commits.
pub fn url_github(racine: &Path) -> Option<String> {
    let sortie = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(racine)
        .output()
        .ok()?;
    if !sortie.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&sortie.stdout).trim().to_string();
    let url = url
        .strip_prefix("git@github.com:")
        .map(|r| format!("https://github.com/{r}"))
        .unwrap_or(url);
    url.starts_with("https://")
        .then(|| url.trim_end_matches('/').trim_end_matches(".git").to_string())
}

/// Analyse la sortie de `git log` au format `%H<US>%cs<US>%s<US>%b<RS>`.
pub fn analyser(texte: &str) -> HashMap<String, Commit> {
    texte
        .split('\u{1e}')
        .filter_map(|bloc| {
            let bloc = bloc.trim_start_matches(['\n', '\r']);
            let mut champs = bloc.split('\u{1f}');
            let hash = champs.next()?.trim().to_string();
            if hash.len() < 10 {
                return None;
            }
            let date = champs.next()?.trim().to_string();
            let sujet = champs.next()?.trim().to_string();
            let (corps, teste) = separer_corps(champs.next().unwrap_or(""));
            Some((hash[..10].to_string(), Commit {
                court: hash[..10].to_string(),
                hash,
                date,
                sujet,
                corps,
                teste,
                principal: false,
            }))
        })
        .collect()
}

/// Sépare le corps d'un commit : paragraphes ordinaires d'un côté, paragraphe « Testé … » de l'autre,
/// sans les lignes `Co-Authored-By`.
fn separer_corps(corps: &str) -> (String, Option<String>) {
    let corps = corps.replace("\r\n", "\n");
    let mut ordinaires = Vec::new();
    let mut teste = None;
    for paragraphe in corps.split("\n\n") {
        let lignes: Vec<&str> = paragraphe
            .lines()
            .filter(|l| !l.starts_with("Co-Authored-By:"))
            .collect();
        let texte = lignes.join("\n").trim().to_string();
        if texte.is_empty() {
            continue;
        }
        if texte.starts_with("Testé") || texte.starts_with("Tests ") || texte.starts_with("Test :") {
            teste = Some(texte);
        } else {
            ordinaires.push(texte);
        }
    }
    (ordinaires.join("\n\n"), teste)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyse_la_sortie_de_git_log() {
        let texte = "3e9abc6ef2aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\u{1f}2026-11-04\u{1f}Icebreaker T.5 : le dernier vivant tombe\u{1f}\
`sys/icebreaker_mort.rs` regarde chaque emplacement.\n\nTesté (test de la règle) et au client texte à deux :\nA rentré.\n\n\
Co-Authored-By: Claude <noreply@anthropic.com>\n\u{1e}\n\
1af064890baaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\u{1f}2026-11-04\u{1f}Icebreaker T : hashes\u{1f}\u{1e}\n";
        let commits = analyser(texte);
        assert_eq!(commits.len(), 2);
        let c = &commits["3e9abc6ef2"];
        assert_eq!(c.court, "3e9abc6ef2");
        assert_eq!(c.date, "2026-11-04");
        assert_eq!(c.sujet, "Icebreaker T.5 : le dernier vivant tombe");
        assert_eq!(c.corps, "`sys/icebreaker_mort.rs` regarde chaque emplacement.");
        assert_eq!(c.teste.as_deref(), Some("Testé (test de la règle) et au client texte à deux :\nA rentré."));
        assert!(!c.corps.contains("Co-Authored"));
        let c2 = &commits["1af064890b"];
        assert_eq!(c2.corps, "");
        assert_eq!(c2.teste, None);
    }

    #[test]
    fn lit_le_depot() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let commits = lire(&racine);
        assert!(commits.len() > 100, "{} commits lus", commits.len());
        let n = notres(&commits).len();
        assert!((100..2000).contains(&n), "{n} commits Icebreaker");
        assert!(commits.values().any(|c| !c.principal), "ancien-master non lu");
    }
}
