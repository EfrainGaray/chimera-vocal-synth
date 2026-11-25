//! The alveolar tap /ɾ/ (Spanish "caro", "cucaracha"; English "water"). As a 20 ms stop-like
//! gesture it was a lottery: after a low vowel the tongue never reached contact (no tap heard),
//! after a high one it sealed for 50-70 ms with the voice stopped (heard as [d]). A tap is one
//! trill strike: a brief contact that dips the sound while the voice carries on.
use chimera::anatomy::human::{BODY_FEMALE, BODY_MALE, HumanBody};
use chimera::phonetics::Segment;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const FS: f64 = 48000.0;

fn seg(ipa: &str, dur: Option<f64>, stress: u8) -> Segment {
    Segment { dur, stress: Some(stress), ..Segment::new(ipa) }
}

fn rms(x: &[f32], a: f64, b: f64) -> f64 {
    let (i, j) = ((a * FS) as usize, (b * FS) as usize);
    (x[i..j].iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / (j - i) as f64).sqrt()
}

/// Render `segs`; the longest quiet run (ms, 20 dB under the vowels) inside `ipa` and its quietest
/// 5 ms frame relative to the vowels.
fn closure(body: HumanBody, name: &str, f0: f64, segs: &[Segment], ipa: &str) -> (usize, f64) {
    let plan = plan_utterance(segs, &PlanOptions { body: name.into(), t0: 0.05, f0: Some(f0), ..Default::default() });
    let mut v = HumanVoice::new(FS, body, 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&plan.gestures);
    let mut x = vec![0.0f32; (FS * (plan.end + 0.3)) as usize];
    for chunk in x.chunks_mut(128) {
        v.render(chunk);
    }
    let c = plan.segments.iter().find(|s| s.ipa == ipa).unwrap();
    let vowels: Vec<_> = plan.segments.iter().filter(|s| s.kind == "vowel").collect();
    let vowel = vowels.iter().map(|s| rms(&x, s.t0 + 0.03, s.t1 - 0.02)).fold(0.0, f64::max);
    let (mut run, mut longest, mut quietest, mut t) = (0, 0, f64::MAX, c.t0);
    while t + 0.005 <= c.t1 + 0.03 {
        let level = rms(&x, t, t + 0.005) / vowel;
        run = if level < 0.1 { run + 1 } else { 0 };
        longest = longest.max(run);
        quietest = quietest.min(level);
        t += 0.005;
    }
    (longest * 5, quietest)
}

#[test]
fn a_tap_dips_the_sound_briefly_in_every_vowel_context() {
    let voices: [(HumanBody, &str, f64); 2] = [(BODY_MALE, "male", 110.0), (BODY_FEMALE, "female", 200.0)];
    let mut wrong = Vec::new();
    for (body, name, f0) in voices {
        for (a, b) in [("ɑ", "ɑ"), ("u", "ɑ"), ("i", "o"), ("e", "e"), ("o", "u"), ("ɑ", "i")] {
            let segs = [seg("k", None, 0), seg(a, Some(0.1), 1), seg("ɾ", None, 0), seg(b, Some(0.1), 0)];
            let (quiet, quietest) = closure(body, name, f0, &segs, "ɾ");
            let db = 20.0 * quietest.log10();
            eprintln!("{name} {a}ɾ{b}: quiet run {quiet} ms, quietest frame {db:.1} dB");
            if quiet > 30 {
                wrong.push(format!("{name} {a}ɾ{b}: the tap went quiet for {quiet} ms (a stop)"));
            }
            if quietest >= 0.5 {
                wrong.push(format!("{name} {a}ɾ{b}: no tap heard ({db:.1} dB)"));
            }
        }
        // negative: a [t] is a stop, with a silent closure the tap must not have
        let segs = [seg("k", None, 0), seg("ɑ", Some(0.1), 1), seg("t", None, 0), seg("ɑ", Some(0.1), 0)];
        let (quiet, _) = closure(body, name, f0, &segs, "t");
        assert!(quiet > 30, "{name} kata: [t] closed for only {quiet} ms");
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
