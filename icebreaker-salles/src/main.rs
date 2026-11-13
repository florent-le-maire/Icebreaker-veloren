//! Icebreaker : écrit `docs/icebreaker/salles.xlsx`, une ligne par salle de
//! `assets/icebreaker/donjons/salles/*.txt` (nom, rôles, thèmes, taille, décor, plan) et une colonne « Retours »
//! vide pour les remarques de l'équipe. Le décor se lit dans `generation.ron` (`decor.symbols`).
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
    /// Le plan, étages compris, tel qu'écrit (sans les lignes de réglage).
    plan: String,
    levels: usize,
    width: usize,
    depth: usize,
    doors: usize,
    pits: usize,
    chests: usize,
    creatures: BTreeMap<String, usize>,
    decor: BTreeMap<char, usize>,
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
        _ => "créature",
    };
    if symbol.is_ascii_uppercase() { format!("{name} (endormie)") } else { name.to_string() }
}

fn read_room(id: &str, text: &str, symbols: &BTreeMap<char, Symbol>) -> Room {
    let mut room = Room {
        id: id.to_string(),
        description: String::new(),
        roles: Vec::new(),
        themes: Vec::new(),
        bosses: Vec::new(),
        plan: String::new(),
        levels: 0,
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
    let mut plan_lines: Vec<&str> = Vec::new();
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
        } else {
            plan_lines.push(line);
        }
    }
    room.description = room.description.trim().to_string();
    while plan_lines.first().is_some_and(|l| l.is_empty()) {
        plan_lines.remove(0);
    }
    while plan_lines.last().is_some_and(|l| l.is_empty()) {
        plan_lines.pop();
    }
    room.plan = plan_lines.join("\n");
    let mut rows = 0;
    for line in &plan_lines {
        if line.starts_with('[') {
            room.levels += 1;
            rows = 0;
            continue;
        }
        rows += 1;
        room.depth = room.depth.max(rows);
        room.width = room.width.max(line.chars().count());
        for symbol in line.chars() {
            match symbol {
                '+' => room.doors += 1,
                '*' => room.pits += 1,
                '$' => room.chests += 1,
                s if symbols.contains_key(&s) => *room.decor.entry(s).or_default() += 1,
                s if "bagcxefijklnpqrtuwyz".contains(s.to_ascii_lowercase()) => {
                    *room.creatures.entry(creature_name(s)).or_default() += 1;
                },
                _ => {},
            }
        }
    }
    room.levels = room.levels.max(1);
    room
}

fn title(id: &str) -> String {
    let words = id.replace('_', " ");
    let mut chars = words.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
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
    let name = Format::new().set_bold().set_text_wrap().set_align(FormatAlign::Top).set_border(FormatBorder::Thin);
    let plan = Format::new()
        .set_font_name("Consolas")
        .set_font_size(10)
        .set_align(FormatAlign::Top)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xF4F1EA));
    let feedback = Format::new()
        .set_text_wrap()
        .set_align(FormatAlign::Top)
        .set_border(FormatBorder::Thin)
        .set_background_color(Color::RGB(0xFFF8DC));

    let sheet = book.add_worksheet();
    sheet.set_name("Salles")?;
    let columns: [(&str, f64); 12] = [
        ("N°", 5.0),
        ("Salle", 20.0),
        ("Fichier", 18.0),
        ("Rôle(s)", 16.0),
        ("Thème(s)", 16.0),
        ("Taille (cases)", 10.0),
        ("Portes / fosses", 9.0),
        ("Habitants et coffres", 22.0),
        ("Décor", 30.0),
        ("Description", 42.0),
        ("Modèle (plan)", 38.0),
        ("Retours", 45.0),
    ];
    for (c, (label, width)) in columns.iter().enumerate() {
        sheet.write_string_with_format(0, c as u16, *label, &head)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    sheet.set_row_height(0, 32)?;
    sheet.set_freeze_panes(1, 2)?;
    for (i, room) in rooms.iter().enumerate() {
        let r = (i + 1) as u32;
        let size = if room.levels > 1 {
            format!("{} × {}\n{} étages", room.width, room.depth, room.levels)
        } else {
            format!("{} × {}", room.width, room.depth)
        };
        let mut inhabitants: Vec<String> = room.creatures.iter().map(|(n, k)| format!("{k} {n}")).collect();
        if room.chests > 0 {
            inhabitants.push(format!("{} coffre(s)", room.chests));
        }
        for boss in &room.bosses {
            inhabitants.push(format!("boss : {boss}"));
        }
        let decor: Vec<String> = room
            .decor
            .iter()
            .map(|(s, k)| format!("{} ×{k}", symbols.get(s).map(|d| d.label.as_str()).unwrap_or("?")))
            .collect();
        let doors = if room.pits > 0 { format!("{} / {}", room.doors, room.pits) } else { room.doors.to_string() };
        let themes = if room.themes.is_empty() { "(tous)".to_string() } else { room.themes.join(", ") };
        sheet.write_number_with_format(r, 0, (i + 1) as f64, &cell)?;
        sheet.write_string_with_format(r, 1, title(&room.id), &name)?;
        sheet.write_string_with_format(r, 2, &room.id, &cell)?;
        sheet.write_string_with_format(r, 3, room.roles.join(", "), &cell)?;
        sheet.write_string_with_format(r, 4, themes, &cell)?;
        sheet.write_string_with_format(r, 5, size, &cell)?;
        sheet.write_string_with_format(r, 6, doors, &cell)?;
        sheet.write_string_with_format(r, 7, inhabitants.join("\n"), &cell)?;
        sheet.write_string_with_format(r, 8, decor.join("\n"), &cell)?;
        sheet.write_string_with_format(r, 9, &room.description, &cell)?;
        sheet.write_string_with_format(r, 10, &room.plan, &plan)?;
        sheet.write_string_with_format(r, 11, "", &feedback)?;
        let lines = room.plan.lines().count().max(decor.len()).max(4);
        sheet.set_row_height(r, (lines as f64 * 13.5).min(400.0))?;
    }
    sheet.autofilter(0, 0, rooms.len() as u32, (columns.len() - 1) as u16)?;

    let legend_sheet = book.add_worksheet();
    legend_sheet.set_name("Légende")?;
    legend(legend_sheet, symbols, &head, &cell)?;
    let themes_sheet = book.add_worksheet();
    themes_sheet.set_name("Thèmes")?;
    themes(themes_sheet, rooms, &head, &cell)?;
    book.save(out)?;
    Ok(())
}

fn legend(sheet: &mut Worksheet, symbols: &BTreeMap<char, Symbol>, head: &Format, cell: &Format) -> Result<()> {
    let fixed = [
        ("#", "mur"),
        (".", "sol"),
        ("O", "pilier"),
        ("D", "porte intérieure (cellule)"),
        ("+", "porte possible dans un mur extérieur"),
        ("*", "fosse possible dans le sol"),
        ("_", "vide (trou, on voit l'étage du dessous)"),
        ("H", "échelle"),
        ("^ > v <", "escalier qui monte dans ce sens"),
        ("$", "coffre"),
        ("S", "point d'apparition (entrée)"),
        ("b a g c x", "créatures éveillées : brute, archer, guetteur, chef, boss (majuscule : endormie)"),
    ];
    for (c, (label, width)) in [("Symbole", 12.0), ("Sens", 44.0), ("Sprite", 30.0), ("Pose", 14.0)].iter().enumerate()
    {
        sheet.write_string_with_format(0, c as u16, *label, head)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    let mut r = 1;
    for (symbol, meaning) in fixed {
        sheet.write_string_with_format(r, 0, symbol, cell)?;
        sheet.write_string_with_format(r, 1, meaning, cell)?;
        sheet.write_string_with_format(r, 2, "", cell)?;
        sheet.write_string_with_format(r, 3, "", cell)?;
        r += 1;
    }
    for (symbol, item) in symbols {
        sheet.write_string_with_format(r, 0, symbol.to_string(), cell)?;
        sheet.write_string_with_format(r, 1, &item.label, cell)?;
        sheet.write_string_with_format(r, 2, &item.sprite, cell)?;
        sheet.write_string_with_format(r, 3, if item.at == "wall" { "contre le mur" } else { "au milieu" }, cell)?;
        r += 1;
    }
    Ok(())
}

fn themes(sheet: &mut Worksheet, rooms: &[Room], head: &Format, cell: &Format) -> Result<()> {
    let mut by_theme: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for room in rooms {
        for theme in &room.themes {
            by_theme.entry(theme).or_default().push(&room.id);
        }
    }
    for (c, (label, width)) in [("Thème", 14.0), ("Salles", 14.0), ("Liste", 100.0)].iter().enumerate() {
        sheet.write_string_with_format(0, c as u16, *label, head)?;
        sheet.set_column_width(c as u16, *width)?;
    }
    for (r, (theme, ids)) in by_theme.iter().enumerate() {
        let r = (r + 1) as u32;
        sheet.write_string_with_format(r, 0, *theme, cell)?;
        sheet.write_number_with_format(r, 1, ids.len() as f64, cell)?;
        sheet.write_string_with_format(r, 2, ids.join(", "), cell)?;
    }
    let neutral: Vec<&str> = rooms.iter().filter(|r| r.themes.is_empty()).map(|r| r.id.as_str()).collect();
    let r = (by_theme.len() + 1) as u32;
    sheet.write_string_with_format(r, 0, "(tous)", cell)?;
    sheet.write_number_with_format(r, 1, neutral.len() as f64, cell)?;
    sheet.write_string_with_format(r, 2, neutral.join(", "), cell)?;
    Ok(())
}
