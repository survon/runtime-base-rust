# Audio Files Directory

This directory contains audio files for the Big Band Mix album.

## Download Instructions

Audio files are not included in the repository due to size constraints.

### Option 1: Automatic Download (via install.sh)
Run the Survon OS installer and select the option to download audio files when prompted.

### Option 2: Manual Download
1. Download the archive from Internet Archive:
   ```bash
   cd ~/manifests/core/big_band_mix/audio
   curl -L "https://archive.org/compress/BigBandMixRecordings1935-1945/formats=VBR%20MP3&file=/BigBandMixRecordings1935-1945.zip" -o BigBandMix.zip
   ```

2. Extract the files:
   ```bash
   unzip BigBandMix.zip
   rm BigBandMix.zip
   ```

3. The files should match these filenames:
    - 01.MoonlightSerenade.mp3
    - 02.BeginTheBeguine.mp3
    - 03.TaintNoUse.mp3
    - ... (and 22 more tracks)

## About This Collection

**Big Band Mix (Recordings 1935-1945)**  
Public Domain / CC BY-NC-SA 3.0

Featuring legendary orchestras including Glenn Miller, Benny Goodman, Art Shaw, and more.

Original 78rpm recordings from various labels (Bluebird, Columbia, Decca, Victor, Okeh).

Source: https://archive.org/details/BigBandMixRecordings1935-1945

## File Requirements

- Supported formats: MP3, WAV, FLAC, OGG, M4A, AAC
- Non-audio files (like this README.md) are automatically ignored
- If audio files are missing, the Jukebox will display "Missing Audio" warnings
