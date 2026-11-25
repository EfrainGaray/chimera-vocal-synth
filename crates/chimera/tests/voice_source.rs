//! How the human voice's source settings shape the glottis.
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

#[test]
fn a_pressed_voice_still_closes_abruptly() {
    // a sung or spoken voice closes softly (tension 0.5-0.55), but the "Pressed" control's top end
    // keeps the abrupt closure of a pressed voice, and the closure sharpens with tension throughout
    let mut v = HumanVoice::with_defaults(48000.0);
    let mut sharpness = |tension: f64| {
        v.set_source(&SourcePatch { tension: Some(tension), ..Default::default() });
        v.glottis.params.closure_sharpness
    };
    assert!(sharpness(0.55) < 0.25);
    assert!(sharpness(1.0) > 0.9);
    let mut last = -1.0;
    for i in 0..=20 {
        let t = i as f64 / 20.0;
        let s = sharpness(t);
        assert!(s > last, "closure sharpness falls at tension {t}");
        last = s;
    }
}

/// A harmonic's power in the output: the largest over ±3 % of its frequency, for the drift.
fn harmonic(x: &[f32], f: f64) -> f64 {
    let at = |f: f64| {
        let n = x.len() as f64;
        let (mut re, mut im) = (0.0, 0.0);
        for (i, s) in x.iter().enumerate() {
            let w = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n).cos();
            let a = std::f64::consts::TAU * f * i as f64 / 48000.0;
            re += w * *s as f64 * a.cos();
            im += w * *s as f64 * a.sin();
        }
        re * re + im * im
    };
    (0..21).map(|i| at(f * (0.97 + 0.003 * i as f64))).fold(0.0, f64::max)
}

#[test]
fn brightness_tilts_the_voice_and_its_default_changes_nothing() {
    // the Bright control: at its default the closure is what tension alone gives; turned up, the
    // folds close more abruptly and the voice carries more high-frequency energy
    let mut v = HumanVoice::with_defaults(48000.0);
    v.set_source(&SourcePatch { tension: Some(0.55), ..Default::default() });
    let default = v.glottis.params.closure_sharpness;
    assert!((default - 0.22).abs() < 1e-12, "{default}");
    let mut tilt = |b: f64| {
        let mut v = HumanVoice::with_defaults(48000.0);
        v.set_source(&SourcePatch { tension: Some(0.55), brightness: Some(b), ..Default::default() });
        v.set_gate(1.0);
        let mut out = vec![0.0f32; 48000];
        for c in out.chunks_mut(128) {
            v.render(c);
        }
        // the harmonics' power at 2-4 kHz over the first six: the share of high frequencies in the
        // voice itself. (The mean squared first difference read the aspiration noise too, which
        // a wider chink and a noise band to 4 kHz made the larger part of the high frequencies.)
        let x = &out[24000..];
        let f0 = v.glottis.params.f0;
        let (mut low, mut high) = (0.0, 0.0);
        for k in 1..=36 {
            let p = harmonic(x, k as f64 * f0);
            if k <= 6 {
                low += p;
            } else if k >= 18 {
                high += p;
            }
        }
        (v.glottis.params.closure_sharpness, 10.0 * (high / low).log10())
    };
    let (dark, mid, bright) = (tilt(0.0), tilt(0.25), tilt(1.0));
    assert_eq!(mid.0, default);
    assert!(dark.0 < mid.0 && mid.0 < bright.0);
    println!("high-frequency share (dB): dark {:.1}, default {:.1}, bright {:.1}", dark.1, mid.1, bright.1);
    // little room to darken (the default closure is already soft), a lot to brighten
    assert!(dark.1 < mid.1 && mid.1 < bright.1 - 3.0, "{dark:?} {mid:?} {bright:?}");
}
