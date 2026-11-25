"""Spanish intelligibility of the engine, scored by Whisper: render each sentence of
sentences_es.tsv with `chimera render spanish` in the male and female voices, transcribe it with
faster-whisper (greedy, language fixed to Spanish, so the same audio gives the same words) and count
word errors. Accents, case and punctuation are ignored (Whisper writes "esta" for "está" as often
as not).

usage: spanish_wer.py <outdir> [--model large-v3] [--whisper] [--against earlier/wer.json]

--whisper renders in the whispered voice (`chimera render spanish --whisper`). Writes
<outdir>/wer.json and the renders. Set up once with tools/asr/setup.sh. The model is
faster-whisper's large-v3 by default (multilingual; small.en, the English judge, can't hear
Spanish); it runs on a CUDA GPU when ctranslate2 sees one, else on the CPU.
"""

import json
import os
import subprocess
import sys
import unicodedata
from concurrent.futures import ThreadPoolExecutor

import ctranslate2
from faster_whisper import WhisperModel

from speech_wer import edits, load16k

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
CACHE = os.path.expanduser("~/.cache/chimera-asr/models")
BODIES = ("male", "female")
SLACK = 3.0
MARGIN = 1.5


def words(text):
    t = "".join(c for c in unicodedata.normalize("NFD", text.lower()) if unicodedata.category(c) != "Mn")
    t = "".join(c if c.isalpha() or c == " " else " " for c in t)
    return t.split()


def main():
    args = sys.argv[1:]
    opt = {}
    for flag in ("--against", "--model"):
        if flag in args:
            i = args.index(flag)
            opt[flag] = args[i + 1]
            del args[i : i + 2]
    whisper = "--whisper" in args
    args = [a for a in args if a != "--whisper"]
    out = args[0]
    os.makedirs(out, exist_ok=True)
    sentences = [l.strip() for l in open(os.path.join(HERE, "sentences_es.tsv")) if l.strip()]
    chimera = os.path.join(REPO, "target/release/chimera")
    jobs = [(text, body, os.path.join(out, f"es-{body}-{i + 1:02d}")) for i, text in enumerate(sentences) for body in BODIES]

    def render(job):
        text, body, stem = job
        cmd = [chimera, "render", "spanish", stem, text, body] + (["--whisper"] if whisper else [])
        subprocess.run(cmd, check=True, capture_output=True)

    with ThreadPoolExecutor(os.cpu_count()) as pool:
        list(pool.map(render, jobs))

    gpu = ctranslate2.get_cuda_device_count() > 0
    model = WhisperModel(opt.get("--model", "large-v3"), device="cuda" if gpu else "cpu",
                         compute_type="float16" if gpu else "int8", cpu_threads=os.cpu_count(), download_root=CACHE)
    rows, totals = [], {}
    for text, body, stem in jobs:
        segs, _ = model.transcribe(load16k(stem + ".wav"), language="es", beam_size=1, temperature=0.0,
                                   condition_on_previous_text=False, vad_filter=False)
        heard = " ".join(s.text.strip() for s in segs)
        ref = words(text)
        e = edits(ref, words(heard))
        rows.append({"body": body, "text": text, "heard": heard, "errors": e, "words": len(ref)})
        for key in (body, "all"):
            t = totals.setdefault(key, [0, 0])
            t[0] += e
            t[1] += len(ref)
    wer = {k: round(100.0 * e / n, 2) for k, (e, n) in totals.items()}
    with open(os.path.join(out, "wer.json"), "w") as f:
        json.dump({"model": opt.get("--model", "large-v3"), "whisper": whisper, "wer": wer, "sentences": rows}, f, indent=1, ensure_ascii=False)
    for r in rows:
        if r["errors"]:
            print(f"  {r['body']:6s} {r['errors']}/{r['words']}  {r['text']}  →  {r['heard']}")
    print("word error rate (%): " + ", ".join(f"{k} {v}" for k, v in sorted(wer.items())))
    if "--against" in opt:
        old, new = json.load(open(opt["--against"]))["wer"]["all"], wer["all"]
        verdict = "WORSE" if new > old + SLACK else "BETTER" if new <= old - MARGIN else "NO CLEAR CHANGE"
        print(f"verdict against {opt['--against']}: {verdict} (word error rate {old} → {new} %)")


if __name__ == "__main__":
    main()
