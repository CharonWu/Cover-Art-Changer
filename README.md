## Cover Art Changer
A command-line tool written in Rust to export or replace cover art of music files.

### Usage

- Export cover art: 
  - Export single cover art: "-e audio.flac" 
  - Export multiple cover arts from a directory: "-e playlist"

- Replace cover art:
  - Replace single cover art: "-r audio.flac cover.jpeg"
  - Replace all cover arts of audio files in the same directory with the same cover art: "-r playlist cover.jpeg"

### Supported format

- Audio:
  - flac
  
- Image:
  - jpeg