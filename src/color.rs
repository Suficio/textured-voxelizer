use cgmath::Vector4;

pub fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

pub fn rgb2hsv(rgb: Vector4<u8>) -> Vector4<f32> {
    let r = (rgb[0] as f32) / 255f32;
    let g = (rgb[1] as f32) / 255f32;
    let b = (rgb[2] as f32) / 255f32;
    let a = (rgb[3] as f32) / 255f32;

    let mut max = r;
    if g > max {
        max = g
    }
    if b > max {
        max = b
    }

    let mut min = r;
    if g < min {
        min = g
    }
    if b < min {
        min = b
    }

    let mid = max - min;

    let mut h;
    if mid == 0f32 {
        h = 0f32;
    } else if max == r {
        h = modulus((g - b) / mid, 6f32);
    } else if max == g {
        h = (b - r) / mid + 2f32;
    } else if max == b {
        h = (r - g) / mid + 4f32;
    } else {
        h = 0f32;
    }

    h *= std::f32::consts::PI / 3f32;
    if h < 0f32 {
        h += 2f32 * std::f32::consts::PI
    }

    let s = if max == 0f32 { 0f32 } else { mid / max };

    Vector4::<f32>::new(h, s, max, a)
}

pub fn hsv2rgb(hsv: Vector4<f32>) -> Vector4<u8> {
    let h = hsv[0] * 180f32 / std::f32::consts::PI;
    let s = hsv[1];
    let v = hsv[2];

    let c = v * s;
    let x = c * (1f32 - (modulus(h / 60f32, 2f32) - 1f32).abs());
    let m = v - c;

    let (r, g, b) = match h {
        hh if hh < 60f32 => (c, x, 0f32),
        hh if hh < 120f32 => (x, c, 0f32),
        hh if hh < 180f32 => (0f32, c, x),
        hh if hh < 240f32 => (0f32, x, c),
        hh if hh < 300f32 => (x, 0f32, c),
        hh if hh < 360f32 => (c, 0f32, x),
        _ => (0f32, 0f32, 0f32),
    };

    Vector4::new(
        ((r + m) * 255f32) as u8,
        ((g + m) * 255f32) as u8,
        ((b + m) * 255f32) as u8,
        (hsv[3] * 255f32) as u8,
    )
}

pub fn hsv_distance(a: &Vector4<f32>, b: &Vector4<f32>) -> f32 {
    // (a.x - b.x).powf(2.0) * 8./21.
    // + (a.y - b.y).powf(2.0) * 5./21.
    // + (a.z - b.z).powf(2.0) * 4./21.
    // + (a.w - b.w).powf(2.0) * 4./21.

    (a.x.sin() * a.y - b.x.sin() * b.y).powf(2.0)
        + (a.x.cos() * a.y - b.x.cos() * b.y).powf(2.0)
        + (a.z - b.z).powf(2.0)
        + (a.w - b.w).powf(2.0)
}

pub fn hsv_average(colors: &Vec<Vector4<u8>>) -> Vector4<f32> {
    let n = colors.len() as f32;
    let mut h_avg = 0f32;
    let mut s_avg = 0f32;
    let mut v_avg = 0f32;
    let mut a_avg = 0f32;

    for c in colors {
        let color = rgb2hsv(*c);
        h_avg += color.x;
        s_avg += color.y;
        v_avg += color.z;
        a_avg += color.w;
    }

    Vector4::<f32>::new(h_avg / n, s_avg / n, v_avg / n, a_avg / n)
}

pub fn convert_colorset_to_hsv(colorset: &Vec<brs::Color>) -> Vec<Vector4<f32>> {
    let mut new = Vec::<Vector4<f32>>::with_capacity(colorset.len());
    for c in colorset.iter() {
        new.push(rgb2hsv(Vector4::new(c.r(), c.g(), c.b(), c.a())));
    }

    new
}

pub fn match_hsv_to_colorset(colorset: &[Vector4<f32>], color: &Vector4<f32>) -> usize {
    let mut min = 0;
    let mut min_distance = hsv_distance(&colorset[0], color);
    for (i, colorset_color) in colorset.iter().enumerate().skip(1) {
        let distance = hsv_distance(colorset_color, color);
        if distance < min_distance {
            min_distance = distance;
            min = i;
        }
    }

    min
}
