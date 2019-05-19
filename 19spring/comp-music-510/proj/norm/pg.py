import numpy as np
from scipy.io import wavfile
import matplotlib.pyplot as plt
import sounddevice as sd
# import pandas as pd

def running_mean(x, N):
    cumsum = np.cumsum(np.insert(x, 0, 0)) 
    return (cumsum[N:] - cumsum[:-N]) / float(N)

def read_note(path):
    _, data = wavfile.read(path)
    print(path, data.shape)

    # Avg the channels
    return data[:,0] / 65536 + data[:,1] / 65536

def simplify_note(data):
    # Keep only first couple seconds
    data = data[:50000]

    # Keep amp only
    data = np.abs(data)

    return running_mean(data, 441)

def norm_max(raw_notes):
    # 1. Find max values.
    max_vs = []
    for name, raw in raw_notes: 
        data = simplify_note(raw)
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
        data = simplify_note(raw)
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

def main():
    notes = 'CDEFGAB'
    nums = '23456'

    raw_notes = []
    for no in nums:
        for n in notes:
            name = f'{n}{no}'
            raw = read_note(f'../samples/Piano.mf.{name}.wav')
            raw_notes.append((name, raw))

    plt.subplot(2, 1, 1)
    for name, raw in raw_notes:
        plt.plot(simplify_note(raw), label=name)
    plt.legend()

    plt.subplot(2, 1, 2)
    norm_notes = norm_attack(norm_max(raw_notes))
    play_some(norm_notes)

    for name, raw in norm_notes:
        plt.plot(simplify_note(raw), label=name)
    plt.legend()

    plt.show()

main()
