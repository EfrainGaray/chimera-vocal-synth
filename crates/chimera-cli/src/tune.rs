//! `--tune k=v,…`: per-render overrides for sweeping the voice without rebuilding. Keys:
//! `source.<field>` (any numeric [`VoiceSource`] field: `breathiness`, `tension`, `roughness`,
//! `brightness`, `pressure`, `drift`, `glide`, `fry`, `vibrato_depth`, `vibrato_rate`, `f0`),
//! `body.<field>` (`chink` cm², `open_phase`, `scale`, `pharynx`, `fold_length`, `tongue_radius`,
//! `lip_width`), `plan.accent`, `plan.declination` (both override the language's melody), `plan.rate`, `plan.unstressed` (the unstressed vowels' length, speech), `song.vibrato` (cents) and `song.consonants` (the sung
//! consonants' length as a multiple of the spoken one, 0.8) (`render notes`).

use chimera::anatomy::human::HumanBody;
use chimera::voice::human_voice::SourcePatch;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tune(pub Vec<(String, f64)>);

impl Tune {
    pub fn parse(spec: &str) -> Result<Tune, String> {
        let mut out = Vec::new();
        for kv in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let (k, v) = kv.split_once('=').ok_or_else(|| format!("--tune: {kv:?} is not key=value"))?;
            let v: f64 = v.trim().parse().map_err(|_| format!("--tune: {v:?} is not a number"))?;
            let k = k.trim();
            if !KEYS.contains(&k) {
                return Err(format!("--tune: unknown key {k:?}; have {}", KEYS.join(", ")));
            }
            out.push((k.to_string(), v));
        }
        Ok(Tune(out))
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.0.iter().rev().find(|(k, _)| k == key).map(|(_, v)| *v)
    }

    /// The `source.*` keys as a patch (`f0` included: the caller decides whether the plan follows).
    pub fn source(&self) -> SourcePatch {
        let g = |k: &str| self.get(&format!("source.{k}"));
        SourcePatch {
            f0: g("f0"),
            glide: g("glide"),
            pressure: g("pressure"),
            breathiness: g("breathiness"),
            tension: g("tension"),
            fry: g("fry"),
            roughness: g("roughness"),
            vibrato_depth: g("vibrato_depth"),
            vibrato_rate: g("vibrato_rate"),
            drift: g("drift"),
            voiced: None,
            brightness: g("brightness"),
        }
    }

    /// `body` with the `body.*` keys applied.
    pub fn body(&self, mut b: HumanBody) -> HumanBody {
        let g = |k: &str| self.get(&format!("body.{k}"));
        if let Some(x) = g("scale") { b.scale = x; }
        if let Some(x) = g("pharynx") { b.pharynx = x; }
        if let Some(x) = g("tongue_radius") { b.tongue_radius = x; }
        if let Some(x) = g("lip_width") { b.lip_width = x; }
        if let Some(x) = g("fold_length") { b.fold_length = x; }
        if let Some(x) = g("open_phase") { b.open_phase = x; }
        if let Some(x) = g("chink") { b.chink = x; }
        if let Some(x) = g("wall_loss") { b.wall_loss = x; }
        if let Some(x) = g("viscous") { b.viscous = x; }
        if let Some(x) = g("radiation") { b.radiation = x; }
        b
    }
}

const KEYS: &[&str] = &[
    "source.f0", "source.glide", "source.pressure", "source.breathiness", "source.tension", "source.fry",
    "source.roughness", "source.vibrato_depth", "source.vibrato_rate", "source.drift", "source.brightness",
    "body.scale", "body.pharynx", "body.tongue_radius", "body.lip_width", "body.fold_length", "body.open_phase",
    "body.chink", "body.wall_loss", "body.viscous", "body.radiation", "plan.accent", "plan.declination", "plan.rate", "plan.unstressed", "song.vibrato", "song.consonants",
];

#[cfg(test)]
mod tests {
    use super::*;
    use chimera::anatomy::human::BODY_MALE;

    #[test]
    fn keys_parse_and_apply() {
        let t = Tune::parse("source.breathiness=0.3, body.chink=0.03,plan.accent=0.4").unwrap();
        assert_eq!(t.source().breathiness, Some(0.3));
        assert_eq!(t.body(BODY_MALE).chink, 0.03);
        assert_eq!(t.get("plan.accent"), Some(0.4));
        assert_eq!(t.get("plan.rate"), None);
    }

    #[test]
    fn a_misspelt_key_is_refused() {
        assert!(Tune::parse("source.breath=0.3").is_err());
        assert!(Tune::parse("source.tension=lots").is_err());
    }
}
