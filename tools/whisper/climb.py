"""Coordinate hill climb of the whisper's WhisperParams on J = D + WER/20: D from the whisper judge
(judge.py, features against human whispers) and WER, Whisper large-v3's word error rate (%) on the
20 Spanish sentences whispered in both voices (tools/asr/spanish_wer.py), so 5 points of WER weigh
as much as 0.25 of D. One parameter at a time over its grid; a value is kept when J falls by 0.02
or more.

usage: climb.py <human.json> <workdir> '<start params as JSON>' [rounds]
Logs every evaluation to <workdir>/climb.log and the best to <workdir>/best.json.
"""
import json, os, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
PY = os.environ.get("ASR_PYTHON", os.path.expanduser("~/.cache/chimera-asr/.venv/bin/python"))
GRID = {
    "noiseGain": [1.0, 3.0, 6.0, 9.0, 12.0],
    "area": [0.06, 0.12, 0.18, 0.25, 0.35],
    "noiseLowpass": [900, 1200, 1600, 2500, 3500],
    "noiseHighpass": [50, 80, 150, 300],
    "ventricular": [0.0, 0.5, 0.85, 1.0],
    "pressure": [0.8, 1.0, 1.25, 1.5],
    "accentPressure": [0.0, 1.0, 2.0],
    "accentLarynx": [0.0, 0.3, 0.8],
    "mic": [0, 10, 20, 40],
}


def main():
    human, work, start = sys.argv[1], sys.argv[2], json.loads(sys.argv[3])
    rounds = int(sys.argv[4]) if len(sys.argv) > 4 else 1
    os.makedirs(work, exist_ok=True)
    log = open(os.path.join(work, "climb.log"), "a")
    cache = {}

    def evaluate(p):
        key = ",".join(f"{k}={v}" for k, v in sorted(p.items()))
        if key in cache:
            return cache[key]
        subprocess.run([PY, os.path.join(HERE, "judge.py"), human, os.path.join(work, "eval"), key], check=True, capture_output=True)
        r = json.load(open(os.path.join(work, "eval", "whisper_eval.json")))
        w = ",".join(f"{k}={v}" for k, v in sorted(p.items()) if k != "mic")
        flags = [f"--whisper={w}"] + ([f"--close-mic={p['mic']}"] if p.get("mic") else [])
        subprocess.run([PY, os.path.join(REPO, "tools/asr/spanish_wer.py"), os.path.join(work, "wer"), *flags], check=True, capture_output=True)
        r["wer"] = json.load(open(os.path.join(work, "wer", "wer.json")))["wer"]
        r["J"] = r["D"] + r["wer"]["all"] / 20
        cache[key] = r
        print(f"J={r['J']:.3f} D={r['D']:.3f} WER={r['wer']['all']} {key} terms={r['terms']}", file=log, flush=True)
        return r

    best, cur = start, evaluate(start)
    print(f"start J {cur['J']:.3f} D {cur['D']:.3f} WER {cur['wer']['all']}", flush=True)
    for _ in range(rounds):
        for k, values in GRID.items():
            for v in values:
                if v == best.get(k):
                    continue
                trial = dict(best, **{k: v})
                r = evaluate(trial)
                if r["J"] <= cur["J"] - 0.02:
                    best, cur = trial, r
                    print(f"{k}={v}: J {cur['J']:.3f} D {cur['D']:.3f} WER {cur['wer']['all']} {cur['terms']}", flush=True)
    json.dump({"best": best, "J": cur["J"], "D": cur["D"], "wer": cur["wer"], "terms": cur["terms"], "synth": cur["synth"], "human": cur["human"]},
              open(os.path.join(work, "best.json"), "w"), indent=1)
    print("best", best, f"J {cur['J']:.3f} D {cur['D']:.3f} WER {cur['wer']['all']}", flush=True)


if __name__ == "__main__":
    main()
