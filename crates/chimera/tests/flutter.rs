//! A held note's pitch wanders: Klatt & Klatt (1990)'s flutter, three slow sines summing to about
//! ±1.5 Hz on a 100 Hz voice (±26 cents), on top of the random drift. Steadier than that, a voice
//! reads as a machine's.
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

#[test]
fn a_held_pitch_wanders_by_klatts_flutter() {
    let fs = 48000.0;
    let mut v = HumanVoice::with_defaults(fs);
    v.set_source(&SourcePatch { f0: Some(110.0), drift: Some(0.0), ..Default::default() });
    v.set_gate(1.0);
    let mut out = vec![0.0f32; 128];
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for i in 0..(2.0 * fs / 128.0) as usize {
        v.render(&mut out);
        if i as f64 * 128.0 > 0.5 * fs {
            let f = v.glottis.f0_now;
            lo = lo.min(f);
            hi = hi.max(f);
        }
    }
    let span = 1200.0 * (hi / lo).log2();
    assert!(span > 30.0 && span < 70.0, "the pitch spans {span:.0} cents ({lo:.2}-{hi:.2} Hz)");
}
