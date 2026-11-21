//! Pour savoir de quel côté regarde un modèle de Veloren : le nombre de voxels sur chaque face de sa boîte.
//! Un meuble adossé a un dos plein. `cargo run -p icebreaker-modeles --example cotes -- <fichier.vox>…`
fn main() {
    for chemin in std::env::args().skip(1) {
        let data = dot_vox::load(&chemin).expect("lecture");
        let m = &data.models[0];
        let (sx, sy, sz) = (m.size.x as u8, m.size.y as u8, m.size.z as u8);
        let compte = |f: &dyn Fn(&dot_vox::Voxel) -> bool| m.voxels.iter().filter(|v| f(v)).count();
        println!(
            "{chemin} {sx}×{sy}×{sz} : x=0 {} | x=max {} | y=0 {} | y=max {}",
            compte(&|v| v.x == 0),
            compte(&|v| v.x == sx - 1),
            compte(&|v| v.y == 0),
            compte(&|v| v.y == sy - 1),
        );
        let _ = sz;
    }
}
