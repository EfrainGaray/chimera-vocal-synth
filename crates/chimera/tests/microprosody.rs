//! The consonants' marks on a spoken pitch (Birkholz & Zhang 2020; Hombert, Ohala & Ewan 1979): a
//! vowel after a voiceless obstruent starts above its target, the pitch dips through a voiced
//! obstruent, and a high vowel is spoken above a low one (Whalen & Levitt 1995). A sung note keeps
//! its pitch.
use chimera::control::score::{ControlKey, Gesture};
use chimera::phonetics::Segment;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};

fn seg(ipa: &str, stress: u8) -> Segment {
    Segment { stress: Some(stress), ..Segment::new(ipa) }
}

fn pitch_gestures(segs: &[Segment]) -> (Vec<Gesture>, Vec<(f64, f64)>) {
    let plan = plan_utterance(segs, &PlanOptions { body: "male".into(), t0: 0.05, f0: Some(110.0), ..Default::default() });
    let pitch = plan.gestures.iter().filter(|g| g.label_str() == "pitch").cloned().collect();
    let vowels = plan.segments.iter().filter(|s| s.kind == "vowel").map(|s| (s.t0, s.t1)).collect();
    (pitch, vowels)
}

fn f0(g: &Gesture) -> f64 {
    g.targets.get(ControlKey::F0).unwrap()
}

#[test]
fn a_vowel_after_a_voiceless_stop_starts_high() {
    let (pitch, vowels) = pitch_gestures(&[seg("ɑ", 0), seg("p", 0), seg("ɑ", 1)]);
    let (t0, _) = vowels[1];
    // the vowel's own target, and a short, stronger one at its onset above it
    let own = pitch.iter().find(|g| (g.t0 - t0).abs() < 1e-9).expect("the vowel's pitch");
    let onset = pitch.iter().find(|g| g.t0 < t0 && g.t1 > t0 && g.t1 - g.t0 < 0.05).expect("an onset mark");
    assert!(f0(onset) > f0(own) * 1.05, "onset {:.1} Hz, target {:.1}", f0(onset), f0(own));
    assert!(onset.strength > own.strength);
}

#[test]
fn the_pitch_dips_through_a_voiced_stop_but_not_a_voiceless_one() {
    let dip = |c: &str| {
        let (pitch, vowels) = pitch_gestures(&[seg("ɑ", 1), seg(c, 0), seg("ɑ", 0)]);
        let (_, v0_end) = vowels[0];
        let (v1_start, _) = vowels[1];
        let own = pitch.iter().find(|g| (g.t0 - vowels[0].0).abs() < 1e-9).map(f0).unwrap();
        pitch.iter().filter(|g| g.t0 >= v0_end - 0.03 && g.t1 <= v1_start + 0.02).map(|g| f0(g) / own).fold(f64::INFINITY, f64::min)
    };
    let b = dip("b");
    assert!(b < 0.95, "through [b] the pitch target falls to {b:.3} of the vowel's");
    assert_eq!(dip("p"), f64::INFINITY, "a voiceless stop leaves no dip (its mark is on the next vowel's onset)");
}

#[test]
fn a_high_vowel_is_spoken_above_a_low_one() {
    let target = |v: &str| {
        let (pitch, vowels) = pitch_gestures(&[seg(v, 1)]);
        pitch.iter().find(|g| (g.t0 - vowels[0].0).abs() < 1e-9).map(f0).unwrap()
    };
    let (i, a) = (target("i"), target("ɑ"));
    assert!(i > a * 1.04, "[i] {i:.1} Hz, [ɑ] {a:.1}");
}

#[test]
fn a_sung_note_keeps_its_pitch() {
    let note = |ipa: &str| Segment { f0: Some(220.0), ..seg(ipa, 1) };
    let (pitch, _) = pitch_gestures(&[note("ɑ"), note("p"), note("ɑ"), note("b"), note("ɑ")]);
    for g in &pitch {
        assert!((f0(g) - 220.0).abs() < 1e-9, "a sung pitch gesture at {:.1} Hz", f0(g));
    }
}
