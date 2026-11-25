"""The whisper judge: CHIMERA whispers each human segment's words (female voice) and its features
are compared with the human's, phrase by phrase (tools/whisper/feats.py: LTAS, spectral tilt,
alpha ratio, F1/F2 and their bandwidths, amplitude-modulation depth). Segments the human voiced
(10 % of frames or more) are left out.

usage: judge.py <human.json> <outdir> [k=v,...]

k=v are WhisperParams fields (`chimera render spanish --whisper=k=v,...`), and mic=<cm> the
microphone distance (--close-mic). Writes <outdir>/whisper_eval.json; prints D, the mean distance
(0: the human's features), and its terms. CHIMERA_BIN overrides target/release/chimera.
"""
import json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from feats import load, features, distance

REPO = os.path.dirname(os.path.dirname(HERE))
BIN = os.environ.get("CHIMERA_BIN", os.path.join(REPO, "target/release/chimera"))


def flags(params):
    kv = [x for x in params.split(",") if x]
    mic = [x.split("=")[1] for x in kv if x.startswith("mic=")]
    w = ",".join(x for x in kv if not x.startswith("mic="))
    return [f"--whisper={w}"] + ([f"--close-mic={mic[0]}"] if mic and float(mic[0]) > 0 else [])


def main():
    human_path, out = sys.argv[1], sys.argv[2]
    params = sys.argv[3] if len(sys.argv) > 3 else ""
    segs = [s for s in json.load(open(human_path))["segments"] if s["voiced"] < 10]
    os.makedirs(out, exist_ok=True)

    def one(i):
        s = segs[i]
        stem = f"{out}/w{i:02d}"
        subprocess.run([BIN, "render", "spanish", stem, s["text"], "female", *flags(params)], check=True, capture_output=True)
        f = features(load(stem + ".wav"))
        d, terms = distance(f, s)
        return {"text": s["text"], "d": d, "terms": terms, "feat": f}

    with ThreadPoolExecutor(len(segs)) as pool:
        rows = list(pool.map(one, range(len(segs))))
    keys = ["tilt", "alpha", "centroid", "F1", "F2", "B1", "B2", "mod_depth", "voiced"]
    mean = lambda rs, k: round(float(np.mean([r[k] for r in rs])), 2)
    D = float(np.mean([r["d"] for r in rows]))
    terms = {k: round(float(np.sqrt(np.mean([r["terms"].get(k, 0) ** 2 for r in rows]))), 2) for k in rows[0]["terms"]}
    report = {"D": D, "terms": terms, "synth": {k: mean([r["feat"] for r in rows], k) for k in keys},
              "human": {k: mean(segs, k) for k in keys}, "params": params, "rows": rows}
    json.dump(report, open(f"{out}/whisper_eval.json", "w"), indent=1, ensure_ascii=False)
    print(f"D = {D:.3f}  terms {terms}")
    print("synth", report["synth"])
    print("human", report["human"])


if __name__ == "__main__":
    main()
