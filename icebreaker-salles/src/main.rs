//! Icebreaker : écrit `docs/icebreaker/salles.xlsx`, le catalogue des salles de
//! `assets/icebreaker/donjons/salles/*.txt` à relire par l'équipe : chaque salle est dessinée en cases de
//! couleur (une case du tableur = une case de la salle), avec sa fiche à gauche et une zone « Retours » à droite.
//! Le décor se lit dans `generation.ron` (`decor.symbols`).
//! Lancer depuis la racine du dépôt : `cargo run -p icebreaker-salles [sortie.xlsx]`.

use anyhow::{Context, Result};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};
use std::collections::BTreeMap;
use std::path::Path;

/// Un symbole de décor : son sprite et son libellé (le commentaire de la ligne de `generation.ron`).
struct Symbol {
    sprite: String,
    label: String,
    at: String,
}

struct Room {
    id: String,
    description: String,
    roles: Vec<String>,
    themes: Vec<String>,
    bosses: Vec<String>,
    /// Les étages (numéro, lignes du plan), du plus bas au plus haut.
    levels: Vec<(i32, Vec<String>)>,
    width: usize,
    depth: usize,
    doors: usize,
    pits: usize,
    chests: usize,
    creatures: BTreeMap<String, usize>,
    decor: BTreeMap<char, usize>,
}

/// Ce qu'on écrit dans une case du dessin : texte court, couleur de fond, couleur du texte.
struct Look {
    text: String,
    fill: u32,
    ink: u32,
}

const WALL: u32 = 0x4A4A4A;
const FLOOR: u32 = 0xEFE6D2;
const WHITE: u32 = 0xFFFFFF;
const BLACK: u32 = 0x222222;

/// Abréviation et couleur d'un symbole de décor (lumières en jaune, restes en gris-vert, meubles en brun).
fn decor_look(symbol: char) -> (&'static str, u32) {
    match symbol {
        '=' => ("Ta", 0xB98A5A),
        '&' => ("Li", 0x9DB7D5),
        '!' => ("Bo", 0xFFD966),
        '?' => ("Ét", 0xB98A5A),
        '|' => ("To", 0xC9A27A),
        '"' => ("Ca", 0xC9A27A),
        '\'' => ("Sa", 0xC9A27A),
        '-' => ("Os", 0xD9D9C3),
        ':' => ("Pa", 0xE8D98A),
        '1' => ("Fe", 0xF4A742),
        '2' => ("Ma", 0xD98E73),
        '3' => ("Ch", 0xD98E73),
        '4' => ("Tr", 0xA9C4A0),
        '5' => ("He", 0xB7C3CC),
        '6' => ("Ba", 0x9AA5AD),
        '7' => ("Cn", 0x9AA5AD),
        '8' => ("Sc", 0xC7A6E0),
        '9' => ("Tb", 0xBFC7BD),
        '0' => ("Or", 0xC7A6E0),
        '`' => ("Tl", 0xD9D9C3),
        ',' => ("Cp", 0xA8D5A2),
        '/' => ("En", 0x9AA5AD),
        '\\' => ("Ea", 0x8FC1E3),
        '(' => ("Ap", 0xFFD966),
        ')' => ("Cl", 0xE0C36A),
        '{' => ("Ou", 0xB98A5A),
        '}' => ("Te", 0xC98F8F),
        _ => ("?", 0xFF00FF),
    }
}

fn look(symbol: char, symbols: &BTreeMap<char, Symbol>) -> Option<Look> {
    let make = |text: &str, fill: u32, ink: u32| Some(Look { text: text.to_string(), fill, ink });
    match symbol {
        ' ' => None,
        '#' => make("", WALL, WHITE),
        '.' => make("", FLOOR, BLACK),
        'O' => make("Pil", 0x8C8C8C, WHITE),
        'D' => make("Grille", 0xE69138, BLACK),
        '+' => make("Porte", 0xE69138, BLACK),
        '*' => make("Fosse", 0x1F3A5F, WHITE),
        '_' => make("vide", 0xBFD3E6, 0x5A6B7A),
        'H' => make("Éch", 0x7FB3D5, BLACK),
        '^' => make("Esc ↑", 0x7FB3D5, BLACK),
        '>' => make("Esc →", 0x7FB3D5, BLACK),
        'v' => make("Esc ↓", 0x7FB3D5, BLACK),
        '<' => make("Esc ←", 0x7FB3D5, BLACK),
        '$' => make("Coffre", 0xF1C232, BLACK),
        'S' => make("Dép", 0x93C47D, BLACK),
        s if symbols.contains_key(&s) => {
            let (text, fill) = decor_look(s);
            make(text, fill, BLACK)
        },
        s if is_creature(s) => {
            let text = match s.to_ascii_lowercase() {
                'b' => "Brute",
                'a' => "Arch",
                'g' => "Guet",
                'c' => "CHEF",
                'x' => "BOSS",
                _ => "Bête",
            };
            // Endormie : rouge pâle.
            make(text, if s.is_ascii_uppercase() { 0xEA9999 } else { 0xCC0000 }, if s.is_ascii_uppercase() { BLACK } else { WHITE })
        },
        _ => make("?", 0xFF00FF, BLACK),
    }
}

fn is_creature(symbol: char) -> bool { "bagcxefijklnpqrtuwyz".contains(symbol.to_ascii_lowercase()) }

fn role_name(role: &str) -> &str {
    match role {
        "entrance" => "Entrée",
        "hall" => "Salle de passage",
        "guardRoom" => "Salle de garde",
        "barracks" => "Caserne",
        "storage" => "Réserve",
        "kitchen" => "Cuisine",
        "nest" => "Nid de bêtes",
        "treasure" => "Salle au trésor",
        "chiefLair" => "Antre du chef",
        "prison" => "Prison",
        "vault" => "Chambre forte",
        "sanctum" => "Sanctuaire",
        "bossLair" => "Antre de boss",
        other => other,
    }
}

fn level_name(level: i32) -> String {
    match level {
        0 => "Rez-de-chaussée".into(),
        n if n > 0 => format!("Étage {n} (au-dessus)"),
        n => format!("Sous-sol {}", -n),
    }
}

fn main() -> Result<()> {
    let out = std::env::args().nth(1).unwrap_or_else(|| "docs/icebreaker/salles.xlsx".into());
    let symbols = read_symbols(Path::new("assets/icebreaker/donjons/generation.ron"))?;
    let mut rooms = Vec::new();
    let dir = Path::new("assets/icebreaker/donjons/salles");
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("{} (lancer depuis la racine du dépôt)", dir.display()))?
    {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "txt") {
            let id = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            rooms.push(read_room(&id, &std::fs::read_to_string(&path)?, &symbols));
        }
    }
    rooms.sort_by(|a, b| a.id.cmp(&b.id));
    write_workbook(&rooms, &symbols, &out)?;
    println!("{} salles dans {out}", rooms.len());
    Ok(())
}

/// Les lignes `'=': (sprite: "Nom", at: center, …), // libellé` du bloc `symbols` de `generation.ron`.
fn read_symbols(path: &Path) -> Result<BTreeMap<char, Symbol>> {
    let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
    let mut symbols = BTreeMap::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("symbols:") {
            inside = true;
            continue;
        }
        if inside && line.starts_with('}') {
            break;
        }
        if !inside || !line.starts_with('\'') {
            continue;
        }
        let mut chars = line[1..].chars();
        let mut symbol = chars.next().unwrap_or(' ');
        if symbol == '\\' {
            symbol = chars.next().unwrap_or(' ');
        }
        let sprite = between(line, "sprite: \"", "\"").unwrap_or_default();
        let at = between(line, "at: ", ",").or_else(|| between(line, "at: ", ")")).unwrap_or_default();
        let label = line.split_once("//").map(|(_, l)| l.trim().to_string()).unwrap_or_else(|| sprite.clone());
        symbols.insert(symbol, Symbol { sprite, label, at });
    }
    Ok(symbols)
}

fn between(text: &str, start: &str, end: &str) -> Option<String> {
    let from = text.find(start)? + start.len();
    let to = text[from..].find(end)? + from;
    Some(text[from..to].to_string())
}

fn creature_name(symbol: char) -> String {
    let name = match symbol.to_ascii_lowercase() {
        'b' => "brute",
        'a' => "archer",
        'g' => "guetteur",
        'c' => "chef",
        'x' => "boss",
        _ => "bête",
    };
    if symbol.is_ascii_uppercase() { format!("{name} endormi(e)") } else { name.to_string() }
}

fn read_room(id: &str, text: &str, symbols: &BTreeMap<char, Symbol>) -> Room {
    let mut room = Room {
        id: id.to_string(),
        description: String::new(),
        roles: Vec::new(),
        themes: Vec::new(),
        bosses: Vec::new(),
        levels: Vec::new(),
        width: 0,
        depth: 0,
        doors: 0,
        pits: 0,
        chests: 0,
        creatures: BTreeMap::new(),
        decor: BTreeMap::new(),
    };
    let list = |value: &str| {
        value.split(',').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect::<Vec<_>>()
    };
    for line in text.lines() {
        let line = line.trim_end();
        if let Some(comment) = line.strip_prefix(';') {
            let comment = comment.trim();
            if !comment.starts_with('+') && !comment.starts_with('*') {
                room.description.push_str(&format!("{comment} "));
            }
        } else if let Some(setting) = line.strip_prefix('@') {
            let (key, value) = setting.split_once(':').unwrap_or((setting, ""));
            match key.trim() {
                "roles" => room.roles = list(value),
                "theme" => room.themes = list(value),
                "boss" => room.bosses = list(value),
                _ => {},
            }
        } else if let Some(header) = line.strip_prefix('[') {
            let number = header.trim_end_matches(']').trim().trim_start_matches("étage").trim().parse().unwrap_or(0);
            room.levels.push((number, Vec::new()));
        } else {
            if room.levels.is_empty() {
                room.levels.push((0, Vec::new()));
            }
            room.levels.last_mut().unwrap().1.push(line.to_string());
        }
    }
    room.description = room.description.trim().to_string();
    // Les lignes vides de fin d'étage ne comptent pas ; celles du début calent l'étage sur les autres.
    for (_, lines) in &mut room.levels {
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
    }
    room.levels.sort_by_key(|(n, _)| *n);
    for (_, lines) in &room.levels {
        room.depth = room.depth.max(lines.len());
        for line in lines {
            room.width = room.width.max(line.chars().count());
            for symbol in line.chars() {
                match symbol {
                    '+' => room.doors += 1,
                    '*' => room.pits += 1,
                    '$' => room.chests += 1,
                    s if symbols.contains_key(&s) => *room.decor.entry(s).or_default() += 1,
                    s if is_creature(s) => *room.creatures.entry(creature_name(s)).or_default() += 1,
                    _ => {},
                }
            }
        }
    }
    room
}

fn title(id: &str) -> String {
    let words = id.replace('_', " ");
    let mut chars = words.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
}

fn square(look: &Look) -> Format {
    Format::new()
        .set_background_color(Color::RGB(look.fill))
        .set_font_color(Color::RGB(look.ink))
        .set_font_size(8)
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_shrink()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0xBBBBBB))
}

/// Largeur, en cases du tableur, du dessin d'une salle (étages côte à côte, une colonne vide entre deux).
fn drawing_width(room: &Room) -> usize { room.levels.len() * (room.width + 1) - 1 }

/// La fiche d'une salle, en clair.
fn card(room: &Room, symbols: &BTreeMap<char, Symbol>) -> String {
    let mut text = String::new();
    let roles: Vec<&str> = room.roles.iter().map(|r| role_name(r)).collect();
    text.push_str(&format!("Sert de : {}\n", roles.join(", ")));
    let themes = if room.themes.is_empty() { "tous".to_string() } else { room.themes.join(", ") };
    text.push_str(&format!("Style de donjon : {themes}\n"));
    let levels = if room.levels.len() > 1 { format!(", {} niveaux", room.levels.len()) } else { String::new() };
    text.push_str(&format!("Taille : {} × {} cases{levels} — {} porte(s) possible(s)", room.width, room.depth, room.doors));
    if room.pits > 0 {
        text.push_str(", 1 fosse");
    }
    text.push_str(&format!("\n\n{}\n", room.description));
    let mut inside: Vec<String> = room.creatures.iter().map(|(n, k)| format!("{k} {n}")).collect();
    if room.chests > 0 {
        inside.push(format!("{} coffre(s)", room.chests));
    }
    for boss in &room.bosses {
        inside.push(format!("boss : {boss}"));
    }
    if !inside.is_empty() {
        text.push_str(&format!("\nDedans : {}\n", inside.join(", ")));
    }
    if !room.decor.is_empty() {
        let decor: Vec<String> = room
            .decor
            .iter()
            .map(|(s, k)| {
                let label = symbols.get(s).map(|d| d.label.as_str()).unwrap_or("?");
                format!("{} = {label} (×{k})", decor_look(*s).0)
            })
            .collect();
        text.push_str(&format!("\nDécor : {}", decor.join(", ")));
    }
    text
}

fn write_workbook(rooms: &[Room], symbols: &BTreeMap<char, Symbol>, out: &str) -> Result<()> {
    let mut book = Workbook::new();
    let head = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2F4F4F))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_border(FormatBorder::Thin);
    let cell = Format::new().set_text_wrap().set_align(FormatAlign::Top).set_border(FormatBorder::Thin);
    let room_title = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2F4F4F))
        .set_align(FormatAlign::VerticalCenter);
    let card_format = Format::new().set_text_wrap().set_align(FormatAlign::Top).set_border(FormatBorder::Thin);
    let level_format = Format::new().set_italic().set_font_size(9).set_font_color(Color::RGB(0x555555));
    let feedback = Format::new()
        .set_text_wrap()
        .set_align(FormatAlign::Top)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xFFF8DC));

    // Colonnes : A la fiche, B.. le dessin, puis une colonne vide et « Retours ».
    let drawing = rooms.iter().map(drawing_width).max().unwrap_or(1) as u16;
    let feedback_col = drawing + 2;
    let mut starts = Vec::new();
    {
        let sheet = book.add_worksheet();
        sheet.set_name("Salles")?;
        sheet.set_column_width(0, 46)?;
        for c in 1..=drawing + 1 {
            sheet.set_column_width_pixels(c, 34)?;
        }
        sheet.set_column_width(feedback_col, 60)?;
        sheet.write_string_with_format(0, 0, "Salle", &head)?;
        sheet.merge_range(0, 1, 0, drawing, "Plan vu de dessus (une case = 4 m ; couleurs : onglet Légende)", &head)?;
        sheet.write_string_with_format(0, feedback_col, "Retours (à remplir)", &head)?;
        sheet.set_row_height(0, 24)?;
        sheet.set_freeze_panes(1, 1)?;

        let mut row: u32 = 2;
        for (i, room) in rooms.iter().enumerate() {
            starts.push(row);
            sheet.merge_range(row, 0, row, feedback_col, &format!("{}. {}", i + 1, title(&room.id)), &room_title)?;
            sheet.set_row_height(row, 24)?;
            // Sous le titre : une ligne de noms d'étages, puis le dessin ; assez haut pour la fiche.
            let top = row + 1;
            let rows = (room.depth as u32 + 1).max(9);
            let bottom = top + rows - 1;
            for r in top..=bottom {
                sheet.set_row_height_pixels(r, 34)?;
            }
            sheet.merge_range(top, 0, bottom, 0, &card(room, symbols), &card_format)?;
            sheet.merge_range(top, feedback_col, bottom, feedback_col, "", &feedback)?;
            for (k, (number, lines)) in room.levels.iter().enumerate() {
                let left = 1 + (k * (room.width + 1)) as u16;
                sheet.write_string_with_format(top, left, level_name(*number), &level_format)?;
                for (y, line) in lines.iter().enumerate() {
                    for (x, symbol) in line.chars().enumerate() {
                        if let Some(look) = look(symbol, symbols) {
                            sheet.write_string_with_format(top + 1 + y as u32, left + x as u16, &look.text, &square(&look))?;
                        }
                    }
                }
            }
            row = bottom + 2;
        }
    }

    {
        let list = book.add_worksheet();
        list.set_name("Liste")?;
        for (c, (label, width)) in
            [("N°", 5.0), ("Salle", 24.0), ("Sert de", 34.0), ("Style de donjon", 24.0), ("Taille", 12.0), ("Portes", 8.0)]
                .iter()
                .enumerate()
        {
            list.write_string_with_format(0, c as u16, *label, &head)?;
            list.set_column_width(c as u16, *width)?;
        }
        list.set_freeze_panes(1, 0)?;
        for (i, room) in rooms.iter().enumerate() {
            let r = (i + 1) as u32;
            let roles: Vec<&str> = room.roles.iter().map(|r| role_name(r)).collect();
            let themes = if room.themes.is_empty() { "tous".to_string() } else { room.themes.join(", ") };
            list.write_number_with_format(r, 0, (i + 1) as f64, &cell)?;
            list.write_url_with_text(r, 1, format!("internal:Salles!A{}", starts[i] + 1).as_str(), title(&room.id))?;
            list.write_string_with_format(r, 2, roles.join(", "), &cell)?;
            list.write_string_with_format(r, 3, themes, &cell)?;
            list.write_string_with_format(r, 4, format!("{} × {}", room.width, room.depth), &cell)?;
            list.write_number_with_format(r, 5, room.doors as f64, &cell)?;
        }
    }

    let legend_sheet = book.add_worksheet();
    legend_sheet.set_name("Légende")?;
    legend(legend_sheet, symbols, &head, &cell)?;
    let themes_sheet = book.add_worksheet();
    themes_sheet.set_name("Styles")?;
    themes(themes_sheet, rooms, &head, &cell)?;
    book.save(out)?;
    Ok(())
}

fn legend(sheet: &mut Worksheet, symbols: &BTreeMap<char, Symbol>, head: &Format, cell: &Format) -> Result<()> {
    let fixed = [
        ('#', "Mur"),
        ('.', "Sol"),
        ('O', "Pilier"),
        ('+', "Porte possible : le générateur l'ouvre s'il en a besoin, sinon c'est un mur"),
        ('D', "Grille de cellule (porte intérieure)"),
        ('*', "Fosse possible vers une salle plus bas (sinon du sol)"),
        ('_', "Vide : on voit l'étage du dessous, on peut y sauter"),
        ('H', "Échelle"),
        ('^', "Escalier (la flèche montre le sens de la montée)"),
        ('$', "Coffre"),
        ('S', "Point de départ des joueurs"),
        ('b', "Ennemi éveillé : Brute, Arch(er), Guet(teur), CHEF, BOSS"),
        ('B', "Ennemi endormi"),
    ];
    for (c, (label, width)) in [("Case", 9.0), ("Ce que c'est", 70.0), ("Objet de Veloren", 32.0), ("Pose", 16.0)].iter().enumerate()
    {
        sheet.write_string_with_format(0, c as u16, *label, head)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    let mut r = 1;
    for (symbol, meaning) in fixed {
        if let Some(look) = look(symbol, symbols) {
            sheet.write_string_with_format(r, 0, &look.text, &square(&look))?;
        }
        sheet.write_string_with_format(r, 1, meaning, cell)?;
        sheet.write_string_with_format(r, 2, "", cell)?;
        sheet.write_string_with_format(r, 3, "", cell)?;
        sheet.set_row_height_pixels(r, 26)?;
        r += 1;
    }
    for (symbol, item) in symbols {
        if let Some(look) = look(*symbol, symbols) {
            sheet.write_string_with_format(r, 0, &look.text, &square(&look))?;
        }
        let mut label = item.label.clone();
        if let Some(first) = label.get(0..1) {
            label = first.to_uppercase() + &label[1..];
        }
        sheet.write_string_with_format(r, 1, label, cell)?;
        sheet.write_string_with_format(r, 2, &item.sprite, cell)?;
        sheet.write_string_with_format(r, 3, if item.at == "wall" { "contre le mur" } else { "au milieu" }, cell)?;
        sheet.set_row_height_pixels(r, 26)?;
        r += 1;
    }
    Ok(())
}

fn themes(sheet: &mut Worksheet, rooms: &[Room], head: &Format, cell: &Format) -> Result<()> {
    let mut by_theme: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for room in rooms {
        for theme in &room.themes {
            by_theme.entry(theme).or_default().push(title(&room.id));
        }
    }
    for (c, (label, width)) in [("Style de donjon", 18.0), ("Salles", 10.0), ("Lesquelles", 110.0)].iter().enumerate() {
        sheet.write_string_with_format(0, c as u16, *label, head)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    for (r, (theme, ids)) in by_theme.iter().enumerate() {
        let r = (r + 1) as u32;
        sheet.write_string_with_format(r, 0, *theme, cell)?;
        sheet.write_number_with_format(r, 1, ids.len() as f64, cell)?;
        sheet.write_string_with_format(r, 2, ids.join(", "), cell)?;
    }
    let neutral: Vec<String> = rooms.iter().filter(|r| r.themes.is_empty()).map(|r| title(&r.id)).collect();
    let r = (by_theme.len() + 1) as u32;
    sheet.write_string_with_format(r, 0, "tous (sans style)", cell)?;
    sheet.write_number_with_format(r, 1, neutral.len() as f64, cell)?;
    sheet.write_string_with_format(r, 2, neutral.join(", "), cell)?;
    Ok(())
}
