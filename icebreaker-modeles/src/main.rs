//! Icebreaker F.23 : écrit nos modèles dans assets/voxygen/voxel/sprite/icebreaker/, et une image de contrôle par
//! modèle dans le dossier donné en argument. Lancer depuis la racine du dépôt :
//! `cargo run -p icebreaker-modeles [dossier d'images]`.

const DOSSIER: &str = "assets/voxygen/voxel/sprite/icebreaker";

fn main() {
    let images = std::env::args().nth(1);
    std::fs::create_dir_all(DOSSIER).expect("lancer depuis la racine du dépôt");
    if let Some(d) = &images {
        std::fs::create_dir_all(d).expect("dossier d'images");
    }
    for m in icebreaker_modeles::tous() {
        let chemin = format!("{DOSSIER}/{}.vox", m.id);
        std::fs::write(&chemin, m.vox()).expect("écriture du .vox");
        let (ox, oy, oz) = m.offset();
        println!(
            "{chemin} : {} × {} × {}, {} voxels, offset: ({ox:.1}, {oy:.1}, {oz:.1})",
            m.taille.0,
            m.taille.1,
            m.taille.2,
            m.nombre_de_voxels()
        );
        if let Some(d) = &images {
            m.image(6).save(format!("{d}/{}.png", m.id)).expect("écriture de l'image");
        }
    }
}
