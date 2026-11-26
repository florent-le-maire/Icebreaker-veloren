//! La défense réactive (IA.7) : « le joueur arme un coup : esquive ou parade tirée après le délai de
//! réaction ». Un coup lent est esquivé ou paré ; une compétence rapide passe, parce qu'elle tombe
//! avant la fin du délai.

/// Ce qu'il fait d'un coup qui arrive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geste {
    /// Une roulade hors du coup.
    Esquiver,
    /// Sa garde, tenue jusqu'à ce que le coup soit passé.
    Parer,
}

/// Sa réponse à un coup armé : tirée une fois, quand il voit le coup partir.
#[derive(Clone, Copy, Debug)]
pub struct Defense {
    /// Il a vu le coup s'armer à cette heure.
    pub vu_a: f64,
    /// Ce qu'il a tiré (rien : il encaisse).
    pub geste: Option<Geste>,
    /// Sa roulade est partie.
    pub fait: bool,
}

impl Defense {
    /// Le geste à faire à ce tick : rien avant la fin de son délai de réaction ; la roulade une
    /// seule fois, la parade tant que le coup n'est pas passé.
    pub fn geste(&mut self, temps: f64, reaction: f64) -> Option<Geste> {
        let geste = self.geste.filter(|_| temps - self.vu_a >= reaction)?;
        if geste == Geste::Esquiver && self.fait {
            return None;
        }
        self.fait = true;
        Some(geste)
    }
}
