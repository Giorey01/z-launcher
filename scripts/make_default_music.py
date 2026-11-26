#!/usr/bin/env python3
"""Makes the launcher's built-in background track, android/app/src/main/assets/music/chill.ogg.

An original loop written for Z-Launcher (no samples, no third-party audio): Fmaj7-Em7-Dm7-Cmaj7 at
80 BPM with a pad, a bass, an arpeggio and soft drums, synthesised here. Usage:

    python3 scripts/make_default_music.py out.wav
    ffmpeg -i out.wav -af "lowpass=f=3800,aecho=0.8:0.6:120|260:0.25|0.15,volume=0.9" \
        -c:a libvorbis -q:a 4 android/app/src/main/assets/music/chill.ogg
"""
import math, random, struct, sys, wave
SR = 22050; BPM = 80; BEAT = 60 / BPM; BAR = 4 * BEAT
CHORDS = [[53, 57, 60, 64], [52, 55, 59, 62], [50, 53, 57, 60], [48, 52, 55, 59]]
BARS_PER_CHORD = 2; LOOPS = 3
total = int(SR * BAR * BARS_PER_CHORD * len(CHORDS) * LOOPS)
out = [0.0] * total
hz = lambda n: 440 * 2 ** ((n - 69) / 12)
def add(start, dur, fn, gain):
    s0 = int(start * SR)
    for i in range(int(dur * SR)):
        j = s0 + i
        if j >= total: break
        out[j] += gain * fn(i / SR)
random.seed(7)
t = 0.0
for loop in range(LOOPS):
    for chord in CHORDS:
        span = BAR * BARS_PER_CHORD
        for n in chord:  # pad: two detuned sines, slow swell
            f = hz(n)
            env = lambda x, d=span: min(1, x / 0.8) * min(1, (d - x) / 0.6)
            add(t, span, lambda x, f=f, e=env: e(x) * (math.sin(2*math.pi*f*x) + math.sin(2*math.pi*f*1.004*x)) * 0.5, 0.06)
        for b in range(BARS_PER_CHORD * 2):  # bass on beats 1 and 3
            f = hz(chord[0] - 12)
            add(t + b * 2 * BEAT, 1.6 * BEAT, lambda x, f=f: math.exp(-x * 1.5) * min(1, x / 0.01) * math.sin(2*math.pi*f*x), 0.22)
        arp = [chord[0] + 12, chord[2] + 12, chord[1] + 12, chord[3] + 12]
        for k in range(BARS_PER_CHORD * 8):  # eighth-note arpeggio, a few skipped
            if random.random() < 0.22: continue
            f = hz(arp[k % 4]); swing = 0.06 * BEAT if k % 2 else 0
            add(t + k * BEAT / 2 + swing, 1.2, lambda x, f=f: math.exp(-x * 3.2) * min(1, x / 0.004) * (math.sin(2*math.pi*f*x) + 0.3 * math.sin(4*math.pi*f*x)), 0.09)
        for b in range(BARS_PER_CHORD * 4):  # kick on 1 and 3, snare-ish on 2 and 4, hats on offbeats
            bt = t + b * BEAT
            if b % 2 == 0:
                add(bt, 0.35, lambda x: math.exp(-x * 9) * math.sin(2*math.pi*(50 + 70 * math.exp(-x * 30)) * x), 0.35)
            else:
                add(bt, 0.2, lambda x: math.exp(-x * 18) * (random.random() * 2 - 1), 0.07)
            add(bt + BEAT / 2 + 0.05 * BEAT, 0.05, lambda x: math.exp(-x * 80) * (random.random() * 2 - 1), 0.035)
        t += span
peak = max(abs(v) for v in out) or 1
with wave.open(sys.argv[1], 'wb') as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
    w.writeframes(b''.join(struct.pack('<h', int(v / peak * 0.8 * 32767)) for v in out))
