//! Noise and random draws of the cave generator: no dependency, the same
//! results in the game and in the browser.

#[inline]
fn hash(x: i32, y: i32, z: i32, s: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f)
        ^ s.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

#[inline]
fn val(x: i32, y: i32, z: i32, s: u32) -> f32 { (hash(x, y, z, s) >> 8) as f32 / 8_388_607.5 - 1.0 }

/// Smooth noise of the position, in -1..1 (features about one unit wide).
pub fn bruit3(p: [f32; 3], s: u32) -> f32 {
    let f = [p[0].floor(), p[1].floor(), p[2].floor()];
    let (x, y, z) = (f[0] as i32, f[1] as i32, f[2] as i32);
    let smooth = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v, w) = (smooth(p[0] - f[0]), smooth(p[1] - f[1]), smooth(p[2] - f[2]));
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let plane = |z: i32| {
        lerp(
            lerp(val(x, y, z, s), val(x + 1, y, z, s), u),
            lerp(val(x, y + 1, z, s), val(x + 1, y + 1, z, s), u),
            v,
        )
    };
    lerp(plane(z), plane(z + 1), w)
}

pub fn bruit2(x: f32, y: f32, s: u32) -> f32 { bruit3([x, y, 0.37], s) }

/// Three octaves of `bruit3`, the finer ones weighted by `detail` (0..1), in -1..1.
pub fn fbm3(p: [f32; 3], s: u32, detail: f32) -> f32 {
    let (mut sum, mut weight, mut amp, mut freq) = (0.0, 0.0, 1.0, 1.0);
    for i in 0..3 {
        sum += amp * bruit3([p[0] * freq, p[1] * freq, p[2] * freq], s.wrapping_add(i * 101));
        weight += amp;
        amp *= detail;
        freq *= 2.1;
    }
    sum / weight
}

/// Random draws from a seed (splitmix64).
pub struct Alea(u64);

impl Alea {
    pub fn new(graine: u64) -> Self { Self(graine ^ 0x1ceb_4ea4_e400_0001) }

    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// In 0..1.
    pub fn unite(&mut self) -> f32 { (self.u64() >> 40) as f32 / 16_777_216.0 }

    pub fn entre(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.unite() }

    pub fn chance(&mut self, p: f32) -> bool { self.unite() < p }

    /// In 0..n (0 if n is 0).
    pub fn indice(&mut self, n: usize) -> usize { if n == 0 { 0 } else { (self.u64() % n as u64) as usize } }
}
