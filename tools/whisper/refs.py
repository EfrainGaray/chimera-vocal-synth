"""Measure human whispered speech for the whisper judge (tools/whisper/judge.py).

usage: refs.py <segments.json> <audio dir> <out human.json>

segments.json lists [{"file", "t0", "t1", "text"}]: a WAV in <audio dir> (<file>.wav), the
whispered stretch in seconds and its words. The recordings, the list and human.json stay out of
git: they are other people's voices (AGENTS.md, "Nothing copyrighted goes in git").
"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from feats import load, features

segs_path, audio, out_path = sys.argv[1:4]
out = []
for s in json.load(open(segs_path)):
    ft = features(load(os.path.join(audio, s["file"] + ".wav"), s["t0"], s["t1"]))
    out.append({**s, **ft})
    print(f"{s['file'][:12]} {s['t0']:5.1f} voiced {ft['voiced']:5.1f}% tilt {ft['tilt']:6.2f} alpha {ft['alpha']:6.2f} "
          f"F1 {ft['F1']:5.0f} F2 {ft['F2']:5.0f} B1 {ft['B1']:4.0f} B2 {ft['B2']:4.0f} mod {ft['mod_depth']:5.2f}")
json.dump({"segments": out}, open(out_path, "w"), indent=1, ensure_ascii=False)
