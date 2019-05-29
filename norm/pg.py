import os
import numpy as np
from scipy.io import wavfile
import matplotlib.pyplot as plt
import sounddevice as sd
# import pandas as pd

def running_mean(x, N):
    cumsum = np.cumsum(np.insert(x, 0, 0)) 
    return (cumsum[N:] - cumsum[:-N]) / float(N)

def read_note(path):
    if not os.path.isfile(path):
        return None
    _, data = wavfile.read(path, mmap=True)
    print(path, data.shape)

    # Avg the channels
    return data[:,0] / 65536 + data[:,1] / 65536

def amplitude(data, window=441):
    # Keep amp only
    data = np.abs(data)

    return running_mean(data, window)

def norm_max(raw_notes):
    # 1. Find max values.
    max_vs = []
    for name, raw in raw_notes: 
        data = amplitude(raw)
        max_vs.append(np.max(data))
    max_vs = np.array(max_vs)

    # 2. Norm. 
    pliers = np.max(max_vs) / max_vs

    out = []
    for i, (name, raw) in enumerate(raw_notes):
        out.append((name, raw * pliers[i]))

    return out

def norm_attack(raw_notes, thres=0.01, margin_before=500):
    # 1. Find max values.
    # raw_delays = []
    for i, (name, raw) in enumerate(raw_notes):
        data = amplitude(raw)
        delay = np.argmax(data > thres)
        raw_notes[i] = name, raw[delay - margin_before:]
        # raw_delays.append(delay)

    return raw_notes

def play_some(raw_notes):
    for name, raw in raw_notes:
        print(name)
        sd.play(raw[:50000], 44100)
        sd.wait()

def main2():
    notes = 'CDEFGAB'
    for n in notes:
        name = f'{n}4'
        _, raw = wavfile.read(f'../samples/Piano.mf.{name}.wav')
        mono = raw[:,0] / 65536 + raw[:,1] / 65536
        # mono = (raw[:,0] / 2 + raw[:,1] / 2).astype('int16')
        play_some([(name, mono)])

def make_names(notes, nums, b=False):
    for no in nums:
        for n in notes:
            if b:
                yield f'{n}{no}'
                yield f'{n}b{no}'
            else:
                yield f'{n}{no}'

def number_to_pitch(n):
    p = 'CDEFGAB'[n % 7]
    nth = int(n / 7)
    return f'{p}{nth}'

def pitch_to_number(p):
    return 'CDEFGAB'.index(p[0]) + int(p[1]) * 7

def make_play_chord(notes):
    notes = dict(notes)
    start = pitch_to_number('C1') 
    to_play = []
    for pn in range(start, start + 7 * 7):
        p1 = number_to_pitch(pn)
        p2 = number_to_pitch(pn + 2)
        p3 = number_to_pitch(pn + 4)
        if p1 in notes and p2 in notes and p3 in notes:
            c = notes[p1][:50000] + notes[p2][:50000] + notes[p3][:50000]
            to_play.append((p1 + p2 + p3, c))
    play_some(to_play)

# Something wrong with pp.A3
def load_and_norm_notes(nsamples=100000):
    raw = [
        (name, read_note(f'../samples/Piano.mf.{name}.wav'))
        # for name in make_names('CDEFGAB', '1234567', b=True)
        for name in make_names('CDEF', '4')
    ]
    # Remove none
    if nsamples is not None:
        raw = [(name, data[:nsamples + 50000]) for (name, data) in raw]
    raw = [(name, data) for (name, data) in raw if data is not None]
    normed = norm_attack(norm_max(raw))
    if nsamples is not None:
        normed = [(name, data[:nsamples]) for (name, data) in normed]
    return (raw, normed)

def plot_notes(raw, normed):
    plt.subplot(2, 1, 1)
    for name, ss in raw:
        plt.plot(amplitude(ss), label=name)
    plt.legend()

    plt.subplot(2, 1, 2)
    for name, ss in normed:
        plt.plot(amplitude(ss), label=name)
    plt.legend()

    plt.show()

def show_fit_polynomial_curve(normed):
    y = amplitude(dict(normed)['C4'])
    X = np.arange(len(y))
    plt.plot(y, label=f'C4')
    for degree in range(10, 15):
        z = np.poly1d(np.polyfit(X, y, degree))
        y_pred = z(X)
        plt.plot(y_pred, label=f'C4-fit:{degree}')
    plt.legend()
    plt.show()

def fit_polynomial_curve(normed, center='C4', degree=25):
    zs = {}
    for name, data in normed:
        y = amplitude(data, 1)
        x = np.arange(len(y))
        z = np.poly1d(np.polyfit(x, y, degree))
        zs[name] = z(x)
        print(len(y), len(x))

    target_ratio = zs[center]
    out = []
    for name, data in normed:
        ratio = target_ratio / zs[name]
        print(len(ratio), len(data))
        out.append((name, data * ratio))
    return out


# Loops don't quite work

def max_n(xs, n):
    return xs.argsort()[-n:][::-1]

def see_fft(normed):
    rtz = Goertzel()
    freqs = list(range(100, 1000, 1))
    for name, ss in normed:
        amp = rtz.amp(ss, freqs)
        plt.plot(freqs, amp, label=name)
        # print(name, amp)
    plt.legend()
    plt.show()

def transpose(xss):
    return list(map(list, zip(*xss)))

def see_freq_over_time(normed):
    c4 = dict(normed)['D4']
    rtz = Goertzel(1000)
    res = []
    freqs = np.array(list(range(200, 750, 1)))
    for i in range(20):
        amp = rtz.amp(c4[i*500:i*500+1000], freqs)
        tops = freqs[max_n(amp, 5)]
        res.append(tops)
    res = transpose(res)
    for i, r in enumerate(res):
        plt.plot(r, label=f'{i}th')
    plt.legend()
    plt.show()

class Goertzel:
    def __init__(self, window_size=10000, sample_rate=44100):
        self.sample_rate = sample_rate
        self.window_size = window_size
        self.inv_f_step = self.window_size / self.sample_rate
        self.f_step_normalized = 1.0 / self.window_size

    def make_simple_binned_freqs(self, freqs):
        return [f * self.inv_f_step for f in freqs]

    def amp(self, samples, freqs):
        ks = self.make_simple_binned_freqs(freqs)
        results = []
        for k in ks:
            f = k * self.f_step_normalized
            coeff = 2.0 * np.cos(2.0 * np.pi * f)

            q1 = 0.0
            q2 = 0.0
            for n in range(self.window_size):
                q0 = coeff * q1 - q2 + samples[n]
                q2 = q1
                q1 = q0

            results.append(q1 ** 2 + q2 ** 2 - coeff * q1 * q2)

        return np.array(results)

def save_notes(ns):
    for name, data in ns:
        data = data[:100000]
        mx = np.max(np.abs(data))
        data = (data * (10000 / mx)).astype('int16')
        wavfile.write(f'../samples/normed/{name}.wav', data=data,
                rate=44100)

def main():
    raw, normed = load_and_norm_notes()
    fits = fit_polynomial_curve(normed)
    # make_play_chord(normed)
    plot_notes(normed, fits)
    # make_loop(normed)
    # play_some(normed)
    # make_loop_by_rev(normed)
    # see_fft(normed)
    # see_freq_over_time(normed)
    # save_notes(normed)

main()