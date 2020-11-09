/// A representation of an RGB color
pub type Color = glm::DVec3;

const SRGB_GAMMA: f64 = 2.2;

/// Construct a new color from an sRGB hex string, such as `hex_color(0xab23f0)`,
/// applying gamma correction to return the approximate intensities.
pub fn hex_color(x: u32) -> Color {
    let r = ((x >> 16) & 0xff) as f64 / 255.0;
    let g = ((x >> 8) & 0xff) as f64 / 255.0;
    let b = ((x >> 0) & 0xff) as f64 / 255.0;
    glm::vec3(r.powf(SRGB_GAMMA), g.powf(SRGB_GAMMA), b.powf(SRGB_GAMMA))
}

/// Convert a color to a clamped triple of unsigned bytes
pub fn color_bytes(color: &Color) -> [u8; 3] {
    [
        (color.x.max(0.0).min(1.0).powf(1.0 / SRGB_GAMMA) * 255.0) as u8,
        (color.y.max(0.0).min(1.0).powf(1.0 / SRGB_GAMMA) * 255.0) as u8,
        (color.z.max(0.0).min(1.0).powf(1.0 / SRGB_GAMMA) * 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_work() {
        let black = hex_color(0x000000);
        let white = hex_color(0xffffff);
        let red = hex_color(0xff0000);
        assert_eq!(color_bytes(&black), [0, 0, 0]);
        assert_eq!(color_bytes(&white), [255, 255, 255]);
        assert_eq!(color_bytes(&red), [255, 0, 0]);
    }
}
