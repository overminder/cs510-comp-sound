# A playground for plotting sound amplitude, normalizing sound, and
# playing chord.

import os
import numpy as np
from scipy.io import wavfile
import soundfile as sf
import matplotlib.pyplot as plt
import sounddevice as sd

### IO / Driver

def load_and_norm_notes(note_names, dyn='mf', nsamples=300000):
    raw = {
        name: load_note(f'../samples/Piano.{dyn}.{name}.wav')
        for name in note_names
    }
    # Remove none
    raw = {name: data for (name, data) in raw.items() if data is not None}

    # Only take that many samples, to speed up processing.
    if nsamples is not None:
        raw = {name: data[:nsamples + 50000] for (name, data) in raw.items()}

    # Balance left and right channels
    raw = {name: balance_lr(data) for (name, data) in raw.items()}

    # Stereo to mono
    # raw = {name: (data[:,0] + data[:,1]) / 2 for (name, data) in raw.items()}

    # Normalize volume and align attacks 
    normed = norm_attack(norm_max(raw, 0.5),
            # pp contains quite some noise, so threshold needs to be larger (40%)
            thres=None if dyn != 'pp' else 0.2)

    # Crop the result
    if nsamples is not None:
        normed = {name: data[:nsamples] for (name, data) in normed.items()}

    return raw, normed

def save_notes(notes, dyn):
    for name, ss in notes.items(): 
        path = f'../samples/normed/{name}.{dyn}.flac'
        sf.write(path, data=ss, samplerate=44100, subtype='PCM_24')
        print('Save', os.path.split(path)[-1], ss.shape)

### Sound Manipulation

# Linearly scale {name: samples} so that the max value of each sample
# are given by the anchor.
def norm_max(raw, anchor=None):
    # 1. Find max values.
    max_vs = np.array([np.max(amplitude(ss, merge2=True)) for ss in raw.values()])

    # 2. Find multipliers to norm.
    if anchor is None:
        anchor = np.max(max_vs)
    pliers = anchor / max_vs

    # 3. Apply multipliers.
    out = {name: ss * pliers[i] for (i, (name, ss)) in enumerate(raw.items())}

    return out

# Slide {name: samples} so that they reach thres at exactly the margin_before
# sample. This assumes that in the original samples, thres always happen
# after margin_before.
def norm_attack(raw, thres=None, margin_before=500):
    if thres is None:
        # Guess the thres.
        # Assume the samples are already normalized, we use 20% of the
        # max amp as the threshold.
        thres = np.max(amplitude(next(iter(raw.values())))) * 0.2

    out = {}
    for name, ss in raw.items():
        amp = amplitude(ss, merge2=True)

        # Find the delay before reaching thres.
        delay = np.argmax(amp > thres)

        # Slide by that many.
        out[name] = ss[delay - margin_before:]

    return out

def balance_lr(ss):
    l = np.max(amplitude(ss[:,0]))
    r = np.max(amplitude(ss[:,1]))
    rate = l / r
    return np.column_stack((ss[:,0], ss[:,1] * rate))

### IO

def load_note(path):
    if not os.path.isfile(path):
        return None
    data, srate = sf.read(path)
    assert srate == 44100
    print('Load', os.path.split(path)[-1], data.shape)
    return data

### Utils

# 0 -> C0
def number_to_pitch(n):
    p = 'CDEFGAB'[n % 7]
    nth = int(n / 7)
    return f'{p}{nth}'

# C0 -> 0
def pitch_to_number(p):
    return 'CDEFGAB'.index(p[0]) + int(p[1]) * 7

# Make pitch name from given note names and numbers
def make_note_names(notes, nums, b=False):
    for no in nums:
        for n in notes:
            if b:
                yield f'{n}{no}'
                yield f'{n}b{no}'
            else:
                yield f'{n}{no}'

# Running mean of data x with window size N
def running_mean(x, N):
    cumsum = np.cumsum(np.insert(x, 0, 0))
    return (cumsum[N:] - cumsum[:-N]) / float(N)

def amplitude(data, window=441, merge2=False):
    # Keep amp only
    data = np.abs(data)
    dim = len(data.shape)

    if dim == 1:
        return running_mean(data, window)
    elif dim == 2:
        if merge2:
            return running_mean(data[:,0] + data[:,1], window)
        else:
            d1 = running_mean(data[:,0], window)
            d2 = running_mean(data[:,1], window)
            return np.column_stack((d1, d2))

def play_some(notes):
    for name, ss in notes:
        print(name)
        sd.play(ss[:100000], 44100)
        sd.wait()

def main2():
    notes = 'CDEFGAB'
    for n in notes:
        name = f'{n}4'
        _, raw = wavfile.read(f'../samples/Piano.mf.{name}.wav')
        mono = raw[:,0] / 65536 + raw[:,1] / 65536
        # mono = (raw[:,0] / 2 + raw[:,1] / 2).astype('int16')
        play_some([(name, mono)])

def make_play_chord(notes, chords):
    to_play = []
    N = 100000
    for chs in chords:
        wave = np.sum(np.stack([notes[ch][:N] for ch in chs]), axis=0)
        to_play.append((''.join(chs), wave))
    play_some(to_play)

# Something wrong with pp.A3
def plot_notes(raw, normed):
    plt.subplot(2, 1, 1)
    for name, ss in raw.items():
        plt.plot(amplitude(ss), label=name)
    plt.legend()

    plt.subplot(2, 1, 2)
    for name, ss in normed.items():
        plt.plot(amplitude(ss), label=name)
    plt.legend()

    plt.show()

def try_flac(name='C4'):
    path = f'../samples/Piano.mf.{name}.wav'
    data, rate = sf.read(path, dtype='float64')
    print(data.shape, rate, data.dtype)

    # sd.play(data[:100000], rate)
    # sd.wait()

    sf.write('foo.flac', data, rate, subtype='PCM_24')

def main():
    # raw, normed = load_and_norm_notes(make_note_names('CDEFGAB', '1234567'))
    # raw, normed = load_and_norm_notes(make_note_names('CDEFGAB', '1234567'), dyn)

    def make_chord(smap):
        ks = list(smap.keys())
        while ks:
            r = ks[:7]
            ks = ks[7:]
            yield r

    for dyn in ['pp', 'mf', 'ff']:
        raw, normed = load_and_norm_notes(make_note_names('CDEFGAB',
            '1234567', b=True), dyn)
        # make_play_chord(normed, make_chord(normed))
        save_notes(normed, dyn)

        # plt.plot([np.max(amplitude(ss)) for ss in normed.values()], label=dyn)
    # plt.legend()
    # plt.show()

main()
