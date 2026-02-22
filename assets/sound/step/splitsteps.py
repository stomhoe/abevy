import argparse
import os

import numpy as np  # pyright: ignore[reportMissingImports]
import soundfile as sf  # pyright: ignore[reportMissingImports]

OUT_DIR = "."
OUT_NAME_PATTERN = "snow{n}.wav"   # examples: "snow{n}.wav", "step_{n:02d}.wav"
START_INDEX = 1


# Detection tuning for sparse snow steps.
MIN_STEP_GAP_S = 0.20
PRE_PAD_S = 0.03
ENV_SMOOTH_S = 0.012
HP_COEFF = 0.97

# Gates
ONSET_GATE_MAD = 1.8
LAST_TAIL_GATE_MAD = 1.2
MAX_LAST_TAIL_S = 1.2
CUT_BEFORE_NEXT_ONSET_S = 0.02  # 12 ms earlier



def moving_average(x: np.ndarray, n: int) -> np.ndarray:
    n = max(1, int(n))
    kernel = np.ones(n, dtype=np.float32) / n
    return np.convolve(x, kernel, mode="same")


def pick_peaks(env: np.ndarray, threshold: float, min_gap: int) -> np.ndarray:
    # Local maxima above threshold with refractory period.
    candidates = np.where(
        (env[1:-1] > env[:-2])
        & (env[1:-1] >= env[2:])
        & (env[1:-1] >= threshold)
    )[0] + 1

    if len(candidates) == 0:
        return candidates

    picked = []
    last = -10**9
    for idx in candidates:
        if idx - last < min_gap:
            if picked and env[idx] > env[picked[-1]]:
                picked[-1] = idx
                last = idx
            continue
        picked.append(idx)
        last = idx

    return np.array(picked, dtype=np.int64)


def find_onset(env_onset: np.ndarray, peak: int, onset_gate: float) -> int:
    i = peak
    while i > 0 and env_onset[i] > onset_gate:
        i -= 1
    return i


def find_last_end(env_body: np.ndarray, peak: int, tail_gate: float, max_tail: int) -> int:
    i = peak
    end_limit = min(len(env_body) - 1, peak + max_tail)
    while i < end_limit and env_body[i] > tail_gate:
        i += 1
    return i


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Split a footsteps WAV into individual step clips.")
    parser.add_argument("input_wav", help="Input WAV file path.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    y, sr = sf.read(args.input_wav)
    if y.ndim > 1:
        y = y.mean(axis=1)
    y = y.astype(np.float32)

    maxv = float(np.max(np.abs(y)))
    if maxv > 0:
        y = y / maxv

    # High-pass-ish difference to emphasize transients.
    hp = np.concatenate([[y[0]], y[1:] - HP_COEFF * y[:-1]])

    # Two envelopes:
    # - env_onset: transient-heavy, for peak/onset detection
    # - env_body: full-band, for tail end on last clip
    env_onset = moving_average(np.abs(hp), int(ENV_SMOOTH_S * sr))
    env_body = moving_average(np.abs(y), int(ENV_SMOOTH_S * sr))

    # Robust threshold on onset envelope.
    med = float(np.median(env_onset))
    mad = float(np.median(np.abs(env_onset - med)))
    if mad <= 1e-8:
        mad = float(np.std(env_onset) + 1e-8)

    high_thr = med + 4.5 * mad
    min_gap = int(MIN_STEP_GAP_S * sr)
    peaks = pick_peaks(env_onset, high_thr, min_gap)

    # Fallback if threshold too strict.
    if len(peaks) == 0:
        high_thr = med + 3.0 * mad
        peaks = pick_peaks(env_onset, high_thr, min_gap)

    if len(peaks) == 0:
        print("No steps detected.")
        return

    onset_gate = med + ONSET_GATE_MAD * mad

    body_med = float(np.median(env_body))
    body_mad = float(np.median(np.abs(env_body - body_med)))
    if body_mad <= 1e-8:
        body_mad = float(np.std(env_body) + 1e-8)
    tail_gate = body_med + LAST_TAIL_GATE_MAD * body_mad

    onsets = [find_onset(env_onset, int(p), onset_gate) for p in peaks]

    os.makedirs(OUT_DIR, exist_ok=True)

    pre_pad = int(PRE_PAD_S * sr)
    max_last_tail = int(MAX_LAST_TAIL_S * sr)
    min_len = int(0.05 * sr)

    # add near top with tunables

    # in main(), after computing pre_pad/max_last_tail/min_len
    pre_next = int(CUT_BEFORE_NEXT_ONSET_S * sr)

    # replace non-last segment end logic:


    saved = 0
    for i, peak in enumerate(peaks):
        a = max(0, onsets[i] - pre_pad)

        # Key behavior:
        # cut at next onset (not at valley/dropoff), so tail is preserved.
        if i < len(peaks) - 1:
            b = max(a + 1, onsets[i + 1] - pre_next)
        else:
            b = max(a + 1, find_last_end(env_body, int(peak), tail_gate, max_last_tail))


        if b - a < min_len:
            continue

        clip = y[a:b]

        # Tiny fade to prevent clicks.
        fade = min(int(0.004 * sr), len(clip) // 8)
        if fade > 0:
            clip = clip.copy()
            clip[:fade] *= np.linspace(0.0, 1.0, fade, dtype=np.float32)
            clip[-fade:] *= np.linspace(1.0, 0.0, fade, dtype=np.float32)

        file_name = OUT_NAME_PATTERN.format(n=START_INDEX + saved)
        out = os.path.join(OUT_DIR, file_name)
        sf.write(out, clip, sr)
        saved += 1

        print(
            f"[{saved:02d}] peak={peak/sr:6.3f}s onset={onsets[i]/sr:6.3f}s "
            f"len={(b-a)/sr:5.3f}s -> {out}"
        )

    print(f"Exported {saved} step WAVs to '{OUT_DIR}/'")


if __name__ == "__main__":
    main()
