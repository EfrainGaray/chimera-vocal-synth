//! Mono WAV output, as `tools/lib/wav.ts` wrote it: 16-bit PCM, non-finite samples as 0, clamped
//! to [−1, 1], `Math.round(v · 32767)`. `--float` writes 32-bit IEEE float instead (the engine's
//! samples exactly, for comparisons).

use chimera::math::js_round;
use std::path::Path;

pub fn write(path: &str, x: &[f32], sample_rate: u32, float: bool) -> std::io::Result<()> {
    let (format, bytes_per_sample): (u16, u32) = if float { (3, 4) } else { (1, 2) };
    let data_bytes = x.len() as u32 * bytes_per_sample;
    let mut b = Vec::with_capacity(44 + data_bytes as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&format.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&sample_rate.to_le_bytes());
    b.extend_from_slice(&(sample_rate * bytes_per_sample).to_le_bytes());
    b.extend_from_slice(&(bytes_per_sample as u16).to_le_bytes());
    b.extend_from_slice(&(bytes_per_sample as u16 * 8).to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_bytes.to_le_bytes());
    for &s in x {
        if float {
            b.extend_from_slice(&s.to_le_bytes());
        } else {
            let mut v = s as f64;
            if !v.is_finite() {
                v = 0.0;
            }
            v = v.clamp(-1.0, 1.0);
            b.extend_from_slice(&(js_round(v * 32767.0) as i16).to_le_bytes());
        }
    }
    if let Some(dir) = Path::new(path).parent()
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, b)
}

/// `x` as a cardioid microphone `cm` from the mouth hears it, against one far away. Half its
/// response is a pressure-gradient's, whose near-field gain over the far field is 1 + c/(jωr) for a
/// point source; on axis the cardioid's is 1 + c/(2jωr), a shelf rising 6 dB an octave below
/// c/(4πr) (550 Hz at 5 cm). The microphone's own low cut (a 2nd-order Butterworth at 80 Hz, and
/// the shelf's floor at 50 Hz) bounds it. The peak is brought back to at most 0.9.
pub fn close_mic(x: &[f32], fs: f64, cm: f64) -> Vec<f32> {
    use std::f64::consts::PI;
    let wp = 343.0 / (2.0 * cm / 100.0);
    let wh = 2.0 * PI * 50.0;
    let k = 2.0 * fs;
    // (s + wp) / (s + wh), bilinear
    let (b0, b1, a1) = ((k + wp) / (k + wh), (wp - k) / (k + wh), (wh - k) / (k + wh));
    // RBJ high-pass, Q = 1/sqrt(2), 80 Hz
    let w0 = 2.0 * PI * 80.0 / fs;
    let alpha = w0.sin() / (2.0 * std::f64::consts::FRAC_1_SQRT_2);
    let cw = w0.cos();
    let n = 1.0 + alpha;
    let (h0, h1, h2, g1, g2) = ((1.0 + cw) / 2.0 / n, -(1.0 + cw) / n, (1.0 + cw) / 2.0 / n, -2.0 * cw / n, (1.0 - alpha) / n);
    let (mut x1, mut y1) = (0.0, 0.0);
    let (mut u1, mut u2, mut v1, mut v2) = (0.0, 0.0, 0.0, 0.0);
    let mut out: Vec<f64> = Vec::with_capacity(x.len());
    for &s in x {
        let s = s as f64;
        let y = b0 * s + b1 * x1 - a1 * y1;
        x1 = s;
        y1 = y;
        let z = h0 * y + h1 * u1 + h2 * u2 - g1 * v1 - g2 * v2;
        u2 = u1;
        u1 = y;
        v2 = v1;
        v1 = z;
        out.push(z);
    }
    let peak = out.iter().fold(0.0f64, |p, v| p.max(v.abs()));
    let g = if peak > 0.9 { 0.9 / peak } else { 1.0 };
    out.iter().map(|v| (v * g) as f32).collect()
}
