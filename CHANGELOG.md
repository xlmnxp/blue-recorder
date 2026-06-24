## [0.3.0] - 2026-06-24

### 🚀 Features

- Move to VP9 and change location of progress dialog, improve wayland support
- Improve audio process handling and ensure complete file writing before merging
- Improve UI dark and light mode
- Improve webm format
- Remove anything related to GTK from Blue Recorder Core
- Improve ffmpeg handling for video encoding and error management
- Hide close button when app is recording
- Update processing UI and spinner labels to be inline and short
- Record and process the video in separated thread
- Support APNG
- Allow user to enter framerates as number
- Implement area capture  in wayland
- Enhance area selection by integrating monitor geometry parsing
- Update README and UI for advanced settings; add localization for advanced settings
- Fix wayland recording in snap
- Improve Wayland session handling and overlay teardown for screen recording
- Fix snap audio
- Sync audio with offsets
- Enhance Wayland pipeline shutdown and improve UI responsiveness on stop
- Update application ID handling for Flatpak and Snap environments
- *(locales)* Add Brazilian Portuguese translation by ecs.eliel21@gmail.com

### 🐛 Bug Fixes

- *(#7)* Unable to find audio sources
- *(#7)* Unable to find audio sources
- *(#7)* Cannot found XDG_SESSION_TYPE
- *(#22)* Ask before overwrite the file
- *(#22)* Ask before overwrite the file
- *(#25)* Do changes suggest by Rafał Mikrut (@qarmin)
- Store videos in tmp folder and fix wayland portal
- Correct area capture sizes

### 💼 Other

- Upgrade to core 22 and fix build issue
- Troubleshooting
- Correct a few paper cuts (#51)
- Fix translation type (#57)
