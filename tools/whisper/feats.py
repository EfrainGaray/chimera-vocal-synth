"""Whisper features of a recording (analysis only; nothing here goes into the repo's git):
LTAS (1/3-octave), spectral tilt, alpha ratio, centroid, flatness, formants and bandwidths of the
vowel-like frames (LPC), voiced-frame fraction, amplitude-modulation depth and rate, pauses."""
import numpy as np, wave
from scipy.signal import resample_poly, lfilter, welch, get_window

BANDS = [100 * 2 ** (k / 3) for k in range(0, 22)]  # 100 Hz .. 12.7 kHz


def load(path, t0=None, t1=None):
    w = wave.open(path)
    sr = w.getframerate()
    x = np.frombuffer(w.readframes(w.getnframes()), dtype="<i2").astype(float) / 32768
    x = x.reshape(-1, w.getnchannels()).mean(1)
    if t0 is not None:
        x = x[int(t0 * sr):int(t1 * sr)]
    if sr != 48000:
        x = resample_poly(x, 48000, sr)
    return x


def frames(x, n, hop):
    k = 1 + max(0, (len(x) - n) // hop)
    return np.stack([x[i * hop:i * hop + n] for i in range(k)])


def lpc(x, order):
    r = np.correlate(x, x, "full")[len(x) - 1:len(x) + order]
    a = np.zeros(order + 1); a[0] = 1; e = r[0]
    if e <= 0:
        return None
    for i in range(1, order + 1):
        k = -(r[i] + np.dot(a[1:i], r[i - 1:0:-1])) / e
        a[1:i + 1] = a[1:i + 1] + k * np.concatenate([a[i - 1:0:-1], [1]])
        e *= 1 - k * k
    return a


def formants(fr, sr):
    a = lpc(fr * np.hamming(len(fr)), 12)
    if a is None:
        return []
    out = []
    for r in np.roots(a):
        if r.imag <= 0:
            continue
        f = np.angle(r) * sr / (2 * np.pi)
        b = -np.log(abs(r)) * sr / np.pi
        if 150 < f < sr / 2 - 200 and b < 700:
            out.append((f, b))
    return sorted(out)


def features(x, sr=48000):
    n, hop = int(0.025 * sr), int(0.010 * sr)
    F = frames(x, n, hop)
    rms = np.sqrt((F ** 2).mean(1)) + 1e-9
    db = 20 * np.log10(rms)
    top = np.percentile(db, 95)
    active = db > top - 30
    # LTAS of active frames
    win = get_window("hann", n)
    spec = (np.abs(np.fft.rfft(F[active] * win, axis=1)) ** 2).mean(0)
    fr = np.fft.rfftfreq(n, 1 / sr)
    ltas = []
    for b in BANDS:
        lo, hi = b / 2 ** (1 / 6), b * 2 ** (1 / 6)
        ltas.append(10 * np.log10(spec[(fr >= lo) & (fr < hi)].sum() + 1e-20))
    ltas = np.array(ltas)
    ref = ltas[[i for i, b in enumerate(BANDS) if 300 <= b <= 5000]].mean()
    ltas -= ref
    sel = [i for i, b in enumerate(BANDS) if 300 <= b <= 8000]
    tilt = np.polyfit(np.log2(np.array(BANDS)[sel]), ltas[sel], 1)[0]
    lo = spec[(fr >= 50) & (fr < 1000)].sum(); hi = spec[(fr >= 1000) & (fr < 5000)].sum()
    alpha = 10 * np.log10(hi / lo)
    # per-frame centroid and flatness
    S = np.abs(np.fft.rfft(F * win, axis=1)) ** 2 + 1e-20
    cen = (S * fr).sum(1) / S.sum(1)
    band = (fr > 100) & (fr < 8000)
    flat = np.exp(np.log(S[:, band]).mean(1)) / S[:, band].mean(1)
    # voiced fraction (autocorrelation at 16 kHz, 80-500 Hz lags)
    y = resample_poly(x, 1, 3)
    Fy = frames(y, 640, 160)
    voiced = []
    for f, a in zip(Fy, active[:len(Fy)]):
        if not a:
            continue
        f = f - f.mean(); ac = np.correlate(f, f, "full")[639:]; ac /= ac[0] + 1e-12
        voiced.append(ac[32:200].max() > 0.5)
    # vowel-like frames: loud (within 12 dB of the top), low centroid (no sibilant)
    vow = (db > top - 12) & (cen < 3000)
    z = resample_poly(x, 1, 4)  # 12 kHz
    z = lfilter([1, -0.9], 1, z)
    Fz = frames(z, int(0.025 * 12000), int(0.010 * 12000))
    f1, f2, b1, b2 = [], [], [], []
    for i in np.nonzero(vow[:len(Fz)])[0]:
        fm = formants(Fz[i], 12000)
        if len(fm) >= 2:
            f1.append(fm[0][0]); b1.append(fm[0][1]); f2.append(fm[1][0]); b2.append(fm[1][1])
    # amplitude modulation: the dB envelope of active speech, its spread and the rate it moves at
    env = np.maximum(db, top - 40)
    depth = float(np.std(env[active]))
    e = env - env.mean()
    m = np.abs(np.fft.rfft(e * np.hanning(len(e)))) ** 2
    mf = np.fft.rfftfreq(len(e), 0.010)
    rng = (mf >= 1.5) & (mf <= 10)
    rate = float(mf[rng][np.argmax(m[rng])]) if rng.any() else 0.0
    # pauses: runs of 150 ms or more under top - 30 dB
    quiet = ~active
    pauses, run = [], 0
    for q in quiet:
        if q:
            run += 1
        else:
            if run >= 15:
                pauses.append(run * 0.010)
            run = 0
    med = lambda v: float(np.median(v)) if len(v) else float("nan")
    return {
        "ltas": [round(v, 2) for v in ltas], "tilt": round(float(tilt), 2), "alpha": round(float(alpha), 2),
        "centroid": round(med(cen[active]), 0), "flatness": round(med(flat[active]), 3),
        "voiced": round(100 * float(np.mean(voiced)), 1) if voiced else 0.0,
        "F1": round(med(f1), 0), "F2": round(med(f2), 0), "B1": round(med(b1), 0), "B2": round(med(b2), 0),
        "mod_depth": round(depth, 2), "mod_rate": round(rate, 2),
        "pause_frac": round(float(quiet.mean()), 3), "pauses_per_s": round(len(pauses) / (len(x) / sr), 3),
        "dur": round(len(x) / sr, 2),
    }


def distance(a, b):
    """How far a render's features (a) are from the human's (b), each term in a just-audible-ish
    unit, root of the mean square: LTAS 3 dB rms over 200 Hz-10 kHz, tilt 2 dB/oct, alpha 3 dB,
    F1 and F2 10 %, B1 and B2 40 %, modulation depth 2 dB."""
    sel = [i for i, f in enumerate(BANDS) if 200 <= f <= 10000]
    la, lb = np.array(a["ltas"])[sel], np.array(b["ltas"])[sel]
    terms = {
        "ltas": np.sqrt(np.mean((la - lb) ** 2)) / 3,
        "tilt": (a["tilt"] - b["tilt"]) / 2,
        "alpha": (a["alpha"] - b["alpha"]) / 3,
        "F1": (a["F1"] / b["F1"] - 1) / 0.10,
        "F2": (a["F2"] / b["F2"] - 1) / 0.10,
        "B1": (a["B1"] / b["B1"] - 1) / 0.40,
        "B2": (a["B2"] / b["B2"] - 1) / 0.40,
        "mod": (a["mod_depth"] - b["mod_depth"]) / 2,
    }
    terms = {k: float(v) for k, v in terms.items() if np.isfinite(v)}
    return float(np.sqrt(np.mean([v * v for v in terms.values()]))), terms
