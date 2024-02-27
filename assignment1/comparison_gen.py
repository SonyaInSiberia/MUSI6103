import numpy as np
import soundfile as sf
import librosa
import matplotlib.pyplot as plt


def fir_comb_filter(input_signal, sample_rate=44100, gain=0.5, delay_sec=0.25):
    delay_samples = int(sample_rate * delay_sec)
    output_signal = np.zeros_like(input_signal)
    print(input_signal.ndim)
    if input_signal.ndim > 1:
        for channel in range(input_signal.shape[1]):  # Iterate over channels
            for n in range(len(input_signal)):
                output_signal[n, channel] = input_signal[n, channel]
                if n - delay_samples >= 0:
                    output_signal[n, channel] += (
                        gain * input_signal[n - delay_samples, channel]
                    )
    else:
        for n in range(len(input_signal)):
            output_signal[n] = input_signal[n]
            if n - delay_samples >= 0:
                output_signal[n] += gain * input_signal[n - delay_samples]
    return output_signal


def iir_comb_filter(input_signal, sample_rate, gain, delay_sec):
    delay_samples = int(sample_rate * delay_sec)
    output_signal = np.zeros_like(input_signal)
    print(input_signal.ndim)
    if input_signal.ndim > 1:
        for channel in range(input_signal.shape[1]):  # Iterate over channels
            for n in range(len(input_signal)):
                output_signal[n, channel] = input_signal[n, channel]
                if n - delay_samples >= 0:
                    output_signal[n, channel] += (
                        gain * output_signal[n - delay_samples, channel]
                    )
    else:
        for n in range(len(input_signal)):
            output_signal[n] = input_signal[n]
            if n - delay_samples >= 0:
                output_signal[n] += gain * output_signal[n - delay_samples]
    return output_signal


def process_audio_file(
    file_path,
    output_path,
    sample_rate_new=44100,
    gain=0.5,
    delay_sec=0.25,
    filter_type="fir",
):
    # Read the input file
    input_signal, original_sample_rate = sf.read(file_path)
    librosa.resample(input_signal, orig_sr=original_sample_rate, target_sr=sample_rate_new)
    print(input_signal.shape)

    # Apply comb filter
    if filter_type == "fir":
        output_signal = fir_comb_filter(input_signal, sample_rate_new, gain, delay_sec)
    else:  # Assume IIR if not FIR
        output_signal = iir_comb_filter(input_signal, sample_rate_new, gain, delay_sec)

    # )  # Ensure signal is within float32 range
    output_signal_int16 = (output_signal * 32767).astype(
        np.int16
    )  # Convert to int16 for WAV file
    # if output_signal_int16.ndim > 1:
    #     output_signal_int16 = output_signal_int16.T
    print(output_signal_int16.shape)
    sf.write(output_path, output_signal_int16, sample_rate_new)


def compare_audio_files(file1_path, file2_path):
    # Load the audio files
    
    y1, sr1 = librosa.load(file1_path, sr=None, mono=False)
    y2, sr2 = librosa.load(file2_path, sr=None, mono=False)
    print(y1.shape, y2.shape, sr1, sr2)
    # Ensure both files have the same sample rate and number of channels
    assert sr1 == sr2, "Sample rates differ between the two audio files."
    assert y1.shape == y2.shape, "Audio dimensions differ between the two audio files."

    # Compute the difference
    difference = y1 - y2
    print(difference)
    return difference, sr1


def plot_difference(
    difference,
    sample_rate,
    title="Difference between audio files",
    save_path="difference_plot.png",
):
    # Assuming the difference array could be multi-channel, we plot each channel separately
    if difference.ndim == 1:
        difference = difference[np.newaxis, :]
    channels = difference.shape[0]
    time = np.arange(difference.shape[1]) / sample_rate

    fig, axs = plt.subplots(channels, 1, figsize=(10, 4 * channels), sharex=True)
    fig.suptitle(title)

    if channels == 1:
        axs.plot(time, difference[0, :])
        axs.set_title("Channel 1")
        axs.set_xlabel("Time (s)")
        axs.set_ylabel("Amplitude")
    else:
        for i in range(channels):
            axs[i].plot(time, difference[i, :])
            axs[i].set_title(f"Channel {i + 1}")
            axs[i].set_xlabel("Time (s)")
            axs[i].set_ylabel("Amplitude")

    plt.tight_layout(rect=[0, 0.03, 1, 0.95])
    plt.savefig(save_path)  # Save the plot to a file
    plt.show()


input_file = "real2.wav"
output_file_fir = "real2_gt_fir.wav"
output_file_iir = "real2_gt_iir.wav"
fir_comp = "real2_fir_48000_0.5_0.25.wav"
iir_comp = "real2_iir_48000_0.5_0.25.wav"

process_audio_file(input_file, output_file_fir, 48000, 0.5, 0.25, 'fir')
process_audio_file(input_file, output_file_iir, 48000, 0.5, 0.25, 'iir')
difference_fir, sr = compare_audio_files(fir_comp, output_file_fir)
difference_iir, sr = compare_audio_files(iir_comp, output_file_iir)
plot_difference(difference_fir, sr, title=fir_comp, save_path="real2_fir_diff.png")
plot_difference(difference_iir, sr, title=iir_comp, save_path="real2_iir_diff.png")
