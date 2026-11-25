//! `chimera`: offline renders to WAV, speech diagnostics and table calibration over the CHIMERA
//! engine. Ports of the TypeScript tools (`tools/render-*.ts`, `speech-diagnostics.ts`,
//! `calibrate-*.ts`) with their arguments, defaults and printed summaries. Spectrograms of the
//! WAVs: `bun tools/spectrogram.ts in.wav out.png [title] [maxFreq] [windowMs]`.

mod calibrate;
mod compare;
mod diagnose;
mod js;
mod render;
mod wav;

use chimera::anatomy::human::{BODY_CHILD, BODY_FEMALE, BODY_MALE, BodyMix, HumanBody};
use chimera::phonetics::syllables::Lang;
use chimera::voice::human_voice::WhisperParams;
use std::process::ExitCode;

const USAGE: &str = "\
usage: chimera <command> [args] [--float] [--whisper]

  render speech  <out> [\"ipa / ipa\"] [male|female|child]
  render text    <out> [\"English text.\"] [male|female|child]
  render turkish <out> [\"Türkçe metin.\"] [male|female|child]
  render spanish <out> [\"Texto en español.\"] [male|female|child]
  render song    [outdir] [male|female|child]
  render notes   <in.json> <out> [male|female|child] [--artic tracks.json] [--tele voice.tele]
                 (melody file; midi <= 0 is a rest; \"lang\": \"tr\" or \"es\" reads Latin-script lyrics as Turkish or Spanish;
                 --artic also writes the articulators at 100 Hz)
  render beatbox [outdir] [preset name | pattern] [male|female|child] [bpm] [bars]
  render creature [creatureId] [outdir]
  render vowels  [male|female|child] [out]
  syllables      \"lyrics\" [en|tr|es]  (what the language front ends read: one JSON syllable per sung
                 note; tr and es read Latin script as Turkish and Spanish)
  diagnose       [outdir]
  compare csd    [csd-dir] [outdir] [--songs a|b|all|en001a,…] [--body female] [--jobs n]
                 [--against earlier/score.json]  (score.json: distance to the singer, and the verdict)
  compare tsp    [tsp-dir] [outdir] [--speakers FA,MA,…] [--jobs n] [--dump]
                 [--against earlier/score.json]  (distance to 23 real speakers, phone by phone)
  calibrate vowels     [male|female|child|all] [--missing] [--out file]
                       (--missing: fit only vowels the table lacks; existing entries stay untouched)
  calibrate consonants [--out file]

Output names are stems (<out>.wav). --float writes 32-bit float WAVs instead of 16-bit PCM.
--whisper says render speech, text, turkish, spanish and vowels in the whispered voice (no fold
vibration: turbulence at an open glottis); --whisper=area=0.05,noiseGain=2 sets its WhisperParams
(the whisper hill climb's knobs, docs/whisper.md). --close-mic=<cm> records any render through a
cardioid microphone that close to the mouth (its proximity effect lifts the lows, as in ASMR).
Calibration writes the tables the engine embeds, in crates/chimera/src/phonetics/.";

/// The output rate of every render.
pub const FS: f64 = 48000.0;

/// The repository root (data files and the generated tables are found from here).
pub fn repo_path(rel: &str) -> String {
    format!("{}/../../{rel}", env!("CARGO_MANIFEST_DIR"))
}

/// Options shared by every command.
pub struct Ctx {
    /// write 32-bit float WAVs
    pub float: bool,
    /// speak in the whispered voice, so
    pub whisper: Option<WhisperParams>,
    /// record through a cardioid microphone this many cm from the mouth (`--close-mic=<cm>`)
    pub close_mic: Option<f64>,
}

/// `key=value,…` (WhisperParams' camelCase fields) over the defaults.
fn whisper_params(s: &str) -> Result<WhisperParams, String> {
    let mut m = serde_json::Map::new();
    for kv in s.split(',').filter(|kv| !kv.is_empty()) {
        let (k, v) = kv.split_once('=').ok_or_else(|| format!("--whisper: {kv:?} is not key=value"))?;
        let x: f64 = v.parse().map_err(|_| format!("--whisper: {k}={v} is not a number"))?;
        m.insert(k.to_string(), serde_json::json!(x));
    }
    let keys: Vec<String> = m.keys().cloned().collect();
    let p: WhisperParams = serde_json::from_value(serde_json::Value::Object(m)).map_err(|e| format!("--whisper: {e}"))?;
    // serde's default fills unknown keys silently: refuse a typo
    let known = serde_json::to_value(p).map_err(|e| e.to_string())?;
    if let Some(k) = keys.iter().find(|k| known.get(k.as_str()).is_none()) {
        return Err(format!("--whisper: no parameter {k}; have {}", known.as_object().unwrap().keys().cloned().collect::<Vec<_>>().join(", ")));
    }
    Ok(p)
}

impl Ctx {
    pub fn write_wav(&self, path: &str, x: &[f32]) -> Result<(), String> {
        let near;
        let x = match self.close_mic {
            Some(cm) => {
                near = wav::close_mic(x, FS, cm);
                &near[..]
            }
            None => x,
        };
        wav::write(path, x, FS as u32, self.float).map_err(|e| format!("{path}: {e}"))
    }
}

/// The body for a name (`male`, `female`, `child`).
pub fn body(name: &str) -> Result<HumanBody, String> {
    match name {
        "male" => Ok(BODY_MALE),
        "female" => Ok(BODY_FEMALE),
        "child" => Ok(BODY_CHILD),
        _ => Err(format!("unknown body {name:?}; have male, female, child")),
    }
}

/// The app's body mix for a body preset (female 0..1, child 0..1), as `src/ui/body.ts` PRESETS.
pub fn body_mix(name: &str) -> Result<BodyMix, String> {
    match name {
        "male" => Ok(BodyMix::new(0.0, 0.0)),
        "female" => Ok(BodyMix::new(1.0, 0.0)),
        "child" => Ok(BodyMix::new(0.0, 1.0)),
        _ => Err(format!("unknown body {name:?}; have male, female, child")),
    }
}

/// `<out>.wav` for an output stem (a trailing `.wav` is accepted too).
pub fn wav_name(stem: &str) -> String {
    format!("{}.wav", stem.strip_suffix(".wav").unwrap_or(stem))
}

pub fn peak(x: &[f32]) -> f64 {
    x.iter()
        .fold(0.0f64, |p, &v| chimera::math::js_max(p, (v as f64).abs()))
}

/// Positional argument `i`, or a default.
pub fn arg<'a>(a: &'a [String], i: usize, default: &'a str) -> &'a str {
    a.get(i).map(String::as_str).unwrap_or(default)
}

fn run(args: Vec<String>) -> Result<(), String> {
    let float = args.iter().any(|a| a == "--float");
    let whisper = match args.iter().find(|a| a.as_str() == "--whisper" || a.starts_with("--whisper=")) {
        None => None,
        Some(a) => Some(whisper_params(a.strip_prefix("--whisper=").unwrap_or(""))?),
    };
    let close_mic = match args.iter().find_map(|a| a.strip_prefix("--close-mic=")) {
        None => None,
        Some(cm) => Some(cm.parse::<f64>().ok().filter(|cm| *cm > 0.0).ok_or_else(|| format!("--close-mic={cm}: a distance in cm"))?),
    };
    let args: Vec<String> = args.into_iter().filter(|a| a != "--float" && !(a.as_str() == "--whisper" || a.starts_with("--whisper=") || a.starts_with("--close-mic="))).collect();
    let ctx = Ctx { float, whisper, close_mic };
    let cmd: Vec<&str> = args.iter().take(2).map(String::as_str).collect();
    match cmd.as_slice() {
        ["render", kind, ..] => {
            let rest = &args[2..];
            match *kind {
                "speech" => render::speech(&ctx, rest),
                "text" => render::text(&ctx, rest),
                "turkish" => render::turkish(&ctx, rest),
                "spanish" => render::spanish(&ctx, rest),
                "song" => render::song(&ctx, rest),
                "notes" => render::notes(&ctx, rest),
                "beatbox" => render::beatbox(&ctx, rest),
                "creature" => render::creature(&ctx, rest),
                "vowels" => render::vowels(&ctx, rest),
                _ => Err(format!("unknown render kind {kind:?}\n\n{USAGE}")),
            }
        }
        ["compare", "csd", ..] => compare::csd(&ctx, &args[2..]),
        ["compare", "tsp", ..] => compare::tsp::run(&ctx, &args[2..]),
        ["diagnose", ..] => diagnose::run(&ctx, &args[1..]),
        ["syllables", text] => {
            let lang = match args.get(2).map(String::as_str) {
                None | Some("en") => Lang::En,
                Some("tr") => Lang::Tr,
                Some("es") => Lang::Es,
                Some(l) => return Err(format!("no front end for {l}: en, tr or es")),
            };
            render::sung_lyrics(text, lang).and_then(|s| serde_json::to_string(&s).map_err(|e| e.to_string())).map(|j| println!("{j}"))
        }
        ["calibrate", "vowels", ..] => calibrate::vowels(&args[2..]),
        ["calibrate", "consonants", ..] => calibrate::consonants(&args[2..]),
        ["help" | "--help" | "-h", ..] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(USAGE.to_string()),
    }
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
