//! The whispered voice (`voiced` false, `WhisperParams`): no pitch, the open glottis's acoustics,
//! any device rate, and the breaths at a whisper's pauses.
use chimera::analysis::lpc::{FormantOpts, estimate_formants};
use chimera::anatomy::human::{BODY_FEMALE, BodyMix, HumanBody};
use chimera::control::score::ControlKey;
use chimera::phonetics::planner::{PhraseInput, PlanOptions, plan_phrases, vowel_artic};
use chimera::phonetics::spanish::to_phrases;
use chimera::voice::human_voice::{HumanVoice, SourcePatch, oversample_for};

/// A held /ɑ/ for 1.5 s, whispered or voiced; the last second.
fn held_a(fs: f64, body: HumanBody, whisper: bool) -> Vec<f64> {
    let mut v = HumanVoice::new(fs, body, oversample_for(fs));
    v.set_articulation(&vowel_artic("ɑ", BodyMix::FEMALE).unwrap(), None);
    v.set_source(&SourcePatch::f0(200.0));
    if whisper {
        v.set_source(&SourcePatch::whisper());
    }
    v.set_gate(1.0);
    let mut out = vec![0.0f32; (1.5 * fs) as usize];
    for c in out.chunks_mut(128) {
        v.render(c);
    }
    assert!(out.iter().all(|x| x.is_finite()), "{fs} Hz went NaN");
    out[(0.5 * fs) as usize..].iter().map(|&x| x as f64).collect()
}

fn rms(x: &[f64]) -> f64 {
    (x.iter().map(|v| v * v).sum::<f64>() / x.len() as f64).sqrt()
}

/// The highest normalised autocorrelation at a pitch period (80-500 Hz), on 40 ms frames of the
/// first difference (the slow breath is no period); the median over frames.
fn periodicity(x: &[f64], fs: f64) -> f64 {
    let d: Vec<f64> = x.windows(2).map(|w| w[1] - w[0]).collect();
    let n = (0.04 * fs) as usize;
    let mut peaks: Vec<f64> = d
        .chunks(n)
        .filter(|f| f.len() == n)
        .map(|f| {
            let r0: f64 = f.iter().map(|v| v * v).sum();
            ((fs / 500.0) as usize..(fs / 80.0) as usize).map(|lag| f[lag..].iter().zip(f).map(|(a, b)| a * b).sum::<f64>() / r0).fold(f64::MIN, f64::max)
        })
        .collect();
    peaks.sort_by(f64::total_cmp);
    peaks[peaks.len() / 2]
}

/// The median F1 and its bandwidth over 30 ms frames, at 12 kHz.
fn f1(x: &[f64], fs: f64) -> (f64, f64) {
    let step = (fs / 12000.0) as usize;
    let y: Vec<f64> = x.iter().step_by(step).copied().collect();
    let n = 360;
    let mut f: Vec<(f64, f64)> = y.chunks(n).filter(|c| c.len() == n).filter_map(|c| estimate_formants(c, 12000.0, FormantOpts { order: Some(12), ..Default::default() }).first().map(|m| (m.freq, m.bw))).collect();
    let mut fr: Vec<f64> = f.iter().map(|m| m.0).collect();
    fr.sort_by(f64::total_cmp);
    f.sort_by(|a, b| a.1.total_cmp(&b.1));
    (fr[fr.len() / 2], f[f.len() / 2].1)
}

#[test]
fn a_whisper_has_no_pitch_and_is_heard() {
    let (voiced, whisper) = (held_a(48000.0, BODY_FEMALE, false), held_a(48000.0, BODY_FEMALE, true));
    let (pv, pw) = (periodicity(&voiced, 48000.0), periodicity(&whisper, 48000.0));
    assert!(pv > 0.8 && pw < 0.4, "periodicity: voiced {pv:.2}, whispered {pw:.2}");
    // quieter than the voice, as a whisper is (15-25 dB), and not silent
    let db = 20.0 * (rms(&whisper) / rms(&voiced)).log10();
    assert!((-30.0..-8.0).contains(&db), "whisper {db:.1} dB against the voice");
}

#[test]
fn the_open_glottis_raises_f1_and_widens_it() {
    // with the glottis open the trachea couples to the tract: F1 rises and its bandwidth grows
    // (whispered vowels' F1 is 10-30 % over the voiced one's: Kallail & Emanuel 1984; Jovičić 1998)
    let ((fv, bv), (fw, bw)) = (f1(&held_a(48000.0, BODY_FEMALE, false), 48000.0), f1(&held_a(48000.0, BODY_FEMALE, true), 48000.0));
    assert!(fw > 1.08 * fv && fw < 1.5 * fv, "F1 voiced {fv:.0} Hz, whispered {fw:.0} Hz");
    assert!(bw > 1.3 * bv, "B1 voiced {bv:.0} Hz, whispered {bw:.0} Hz");
}

#[test]
fn a_whisper_holds_at_device_rates() {
    let reference = rms(&held_a(48000.0, BODY_FEMALE, true));
    for fs in [16000.0, 22050.0, 32000.0, 44100.0] {
        let r = rms(&held_a(fs, BODY_FEMALE, true));
        assert!(r > 0.5 * reference && r < 2.0 * reference, "{fs} Hz: RMS {r:.4}, 48 kHz {reference:.4}");
    }
}

#[test]
fn breaths_come_at_sentence_pauses_only_when_asked() {
    let inputs: Vec<PhraseInput> = to_phrases("Hola, ¿cómo estás? Muy bien. Gracias.").iter().map(|p| PhraseInput { segments: p.segments.clone(), intonation: Some(p.intonation), pause: Some(p.pause) }).collect();
    let plan = |breaths| plan_phrases(&inputs, &PlanOptions { body: BodyMix::FEMALE, breaths, ..Default::default() });
    let inhales = |p: &chimera::phonetics::Plan| p.gestures.iter().filter(|g| g.label_str() == "inhale" && g.targets.get(ControlKey::BREATH).is_some()).count();
    assert_eq!(inhales(&plan(None)), 0);
    // after "estás?" and "bien.", not after the comma's short pause nor after the last sentence
    assert_eq!(inhales(&plan(Some(true))), 2);
    // and drawn in: the breath is negative
    assert!(plan(Some(true)).gestures.iter().filter_map(|g| g.targets.get(ControlKey::BREATH)).any(|b| b < 0.0));
}
