# Whisper

A whisper is the voice with the vocal folds still. The **Whisper** chip in Play and in Speak, and
`--whisper` on `chimera render speech | text | turkish | spanish | vowels`, set the source's
`voiced` to false; how the voice then whispers is `WhisperParams` (`voice/human_voice.rs`).

## The physics

Nothing here is a filter on the output: the whisper is the airway's own response to a different
source, and only `voiced == false` takes these paths, so the voiced voice is bit for bit as before.

- **The glottis.** The membranous folds close and a chink between the arytenoids stays open, an
  inverted Y (Solomon, McCall, Trosset & Gray 1989): a static opening (`area`, plus `breathyArea`
  per unit of breathiness), with no vibration. The airflow through it is turbulent.
- **The source.** The glottal jet's turbulence is the glottal noise source the voice already has
  for aspiration, driven by the jet's Reynolds number, with its own gain and band while
  whispering (`noiseGain`, `noiseLowpass`, `noiseHighpass`). In a whisper the jet hits the
  narrowed ventricular folds and the epiglottis: a jet on an obstacle radiates far more noise than
  a free one (Shadle 1985), and its spectrum falls above 1-2 kHz.
- **Supraglottic compression.** Whisperers narrow the ventricular folds (Solomon et al. 1989;
  Tsunoda, Niimi & Hirose 1994): `ventricular` holds that narrowing while whispering.
- **The open glottis's acoustics.** With the glottis open the trachea couples to the tract, so
  F1 rises and every formant widens, most of all F1 (Kallail & Emanuel 1984; Jovičić 1998
  measured whispered F1 10-30 % over the voiced one). Nothing sets this: the airway does it. On a
  held female /ɑ/ F1 goes from 842 to 1006 Hz and B1 from 98 to 134 Hz, 18 dB under the voice
  (`tests/whisper.rs`).
- **Prosody without f0.** The planner's pitch targets still arrive; a whisper says them as effort
  and a raised larynx, as whisperers raise intensity and formants where they would raise pitch
  (Higashikawa et al. 1996; Heeren & van Heuven 2014): lung pressure ∝ (f0 ratio)^`accentPressure`,
  the larynx `accentLarynx` cm higher per octave. Only the airway sees the larynx offset.
- **Breaths.** A whispered speech breathes in, audibly, at its sentence pauses
  (`PlanOptions::breaths`, set by Speak's Whisper and the CLI's `--whisper`): the glottis wide open
  and the lungs 1-2 cmH₂O under the air's pressure for the pause, minus 60 ms at each end.
- **A close microphone** (`--close-mic=<cm>`, CLI only): ASMR is recorded a few cm from the mouth,
  where a cardioid's proximity effect lifts the lows (a shelf rising 6 dB an octave below
  c/(4πr)). It is the recording, not the voice, so it is an output filter of the CLI, not part of
  the engine.

Not modelled: mouth clicks and lip noises, and the louder, tenser "stage whisper".

## The judge and the climb

The parameters were hill-climbed against real whispers, the way `docs/hill-climb/` tunes the
voice, with a judge of their own (`tools/whisper/`):

- **Material.** Nine whispered stretches (2-4 s) of Spanish by two women, from public ASMR videos,
  picked because under 3 % of their frames are periodic. The recordings, their list and the
  measurements stay out of git; `refs.py` measures any such list.
- **Features** (`feats.py`): the long-term average spectrum in third octaves, the spectral tilt
  (300 Hz-8 kHz), the alpha ratio (1-5 kHz against 50 Hz-1 kHz), the median F1 and F2 and their
  bandwidths over the vowel-like frames (LPC), and the depth of the amplitude envelope.
- **D** (`judge.py`): CHIMERA whispers each stretch's own words (female voice) and each feature's
  difference is put in a roughly audible unit (3 dB of LTAS, 2 dB/octave of tilt, 3 dB of alpha,
  10 % of F1 or F2, 40 % of a bandwidth, 2 dB of envelope); D is their root mean square, averaged
  over the stretches.
- **J = D + WER/20** (`climb.py`), WER being Whisper large-v3's word error rate on the 20 Spanish
  sentences of `tools/asr/sentences_es.tsv` whispered in both voices: closer to the women's
  whispers, and still understood.

### What the climb found

| Step | Whisper (key changes) | D | WER, whispered |
|---|---|---|---|
| before | the folds held apart (0.29 cm²), aspiration's noise (300-3500 Hz) | 3.25 | 70.7 % |
| 1 | a 0.18 cm² chink, noise 80-1200 Hz at 3×, ventricular folds 0.7 | 1.58 | 23.0 % |
| 2 | 0.25 cm², noise 9×, ventricular 1.0, microphone at 20 cm | 1.07 | 12.7 % |
| 3 (shipped) | 0.35 cm², noise 6×, accent as effort (pressure ∝ ratio²), no larynx | 1.08 | 8.3-9.3 % |

Against the women (means of the nine stretches; human / CHIMERA): spectral tilt +0.8 / +2.6
dB per octave, alpha ratio -0.5 / +1.5 dB, F1 892 / 992 Hz, F2 1944 / 1902 Hz, B1 301 / 243 Hz,
envelope depth 6.3 / 6.8 dB. The LTAS is 3-5 dB short at 200-500 Hz and 2-4 dB over at 1-1.3 kHz
and 2.5-5 kHz. Faster-whisper's GPU and CPU runs differ by about a point of WER on these 40
renders.

Values at their grid's edge (ventricular 1.0, the glottal area) are where the judge pulled them,
and they stay physical: the chink is about the model's /h/ opening (0.33), and the narrowed
ventricular folds leave 0.24 cm². Raising the larynx for an accent made Whisper understand less, so
`accentLarynx` is 0.

## Not yet

- A listener's verdict (the A/B renders exist; nobody has rated them).
- Mouth clicks; a whisper's own consonant timing (whisperers lengthen vowels and stops).
- Male whispers in the judge: the references are two women, so the male voice is only checked by
  Whisper's WER.

