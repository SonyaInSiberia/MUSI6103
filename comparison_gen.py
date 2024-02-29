import numpy as np
import soundfile as sf
import librosa
import matplotlib.pyplot as plt
import os

def vibrato(x, SAMPLERATE, Modfreq, Width):
    Delay = Width  # basic delay of input sample in sec
    DELAY = round(Delay * SAMPLERATE)  # basic delay in # samples
    WIDTH = round(Width * SAMPLERATE)  # modulation width in # samples
    if WIDTH > DELAY:
        raise ValueError("Width greater than basic delay !!!")

    MODFREQ = Modfreq / SAMPLERATE  # modulation frequency in # samples
    LEN = len(x)  # # of samples in WAV-file
    L = 2 + DELAY + WIDTH * 2  # length of the entire delay
    Delayline = np.zeros(L)  # memory allocation for delay
    y = np.zeros_like(x)  # memory allocation for output vector

    for n in range(LEN - 1):
        M = MODFREQ
        MOD = np.sin(M * 2 * np.pi * n)
        TAP = DELAY + WIDTH * MOD
        i = np.floor(TAP).astype(int)
        frac = TAP - i
        Delayline = np.concatenate(([x[n]], Delayline[: L - 1]))
        # Linear Interpolation
        y[n] = Delayline[i] * frac + Delayline[i - 1] * (1 - frac)

    return y


def process_audio_file(input_filename, output_filename, Modfreq, Width):
    data, samplerate = sf.read(input_filename)

    if data.ndim > 1:
        # More than one channel, apply vibrato to each channel
        processed_data = np.apply_along_axis(
            vibrato, 0, data, samplerate, Modfreq, Width
        )
    else:
        # Single channel
        processed_data = vibrato(data, samplerate, Modfreq, Width)

    sf.write(output_filename, processed_data, samplerate)


def display_and_save_spectrogram(wav_filename, output_image_filename):
    # Load the audio file
    y, sr = librosa.load(wav_filename)

    # Create the spectrogram
    fig, ax = plt.subplots()
    D_highres = librosa.stft(y, hop_length=256, n_fft=4096)
    S_db_hr = librosa.amplitude_to_db(np.abs(D_highres), ref=np.max)
    img = librosa.display.specshow(
        S_db_hr, sr=sr, hop_length=256, x_axis="time", y_axis="log", ax=ax
    )

    # Extract the base name of the WAV file for the title
    title = os.path.basename(wav_filename)

    # Set the title to the name of the WAV file
    ax.set(title=f"Spectrogram of {title}")

    # Add a color bar to the plot
    fig.colorbar(img, ax=ax, format="%+2.f dB")

    # Save the plot to a file
    plt.savefig(output_image_filename)


# Example usage
# process_audio_file("sample_2.wav", "sample_2_py.wav", Modfreq=5, Width=0.01)
mod = [0.01, 0.05, 0.1]
# for i in range(2):
#     for m in mod:
#         input_filename = f"sample{str(i+1)}_{str(m)}_5.0.wav"
#         output_filename = f"sample{str(i+1)}_{str(m)}_5.0.png"
#         display_and_save_spectrogram(input_filename, output_filename)
for i in range(2):
    for m in mod:
        input_filename = f"sample_{str(i+1)}.wav"
        output_audioname = f"sample_{str(i+1)}_{str(m)}_5.0_py.wav"
        process_audio_file(input_filename, output_audioname, 5.0, m)
        output_filename = f"sample_{str(i+1)}_{str(m)}_5.0_py.png"
        display_and_save_spectrogram(output_audioname, output_filename)
