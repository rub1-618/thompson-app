# Thompson

A local-first AI-assistant which speaks and texts in english and is supported on Linux + Windows.

![build](https://github.com/rub1-618/thompson-app/actions/workflows/build.yml/badge.svg)

# Features:
- Voice i/o with wake word "Tom". Has offline STT ( whisper ), VAD
- TTS: espeak-ng ( Linux ) / SAPI ( Windows )
- AI-chat with one-click switching between Ollama ( local ) and Gemini API ( cloud ).
- Screen analysis with Gemini Vision.
- Music control commands: MPRIS ( Linux ) / GSMTC + media-keys ( Windows )
- CPU/RAM/temp stats command, reminders, dictation and custom commands!

# Requirements: 
- Ollama + model ( local AI ); 
- whisper-model, for example: ggml-base.bin (voice); 
- Gemini API-key ( cloud / vision ); 
- WebView2 ( Windows ).

# Install ( prebuilt )
You can install a prebuilt version on GitHub Actions -> artifacts / release; 
- Linux — chmod +x ... && ./...AppImage (and also > --appimage-extract-and-run with no FUSE); 
- Windows - .msi.

# Install ( built from source )
> cargo install tauri-cli --version "^2.0"
> cargo run                                 # dev
> cargo tauri build --bundles appimage      # Linux
> cargo tauri build --bundles msi           # Windows

P.S.: if you're using Arch you shall use this one:
> NO_STRIP=1 cargo tauri build --bundles appimage

# Usage:
| command | what it does |
| --- | --- |
| "what time it is" | tells your current time and date |
| "screenshot analysis" | tells the screen analysis with Gemini Vision |
| "play / pause" | toggles music in your current player |
| "next song" | switches to the next song in your player |
| "previous song | switches back in your player's playlist |
| "remind me in ... minutes to ..." | sets a reminder and tells you when the time's up |
| "open browser" | opens google page in browser |
| "type this '...'" | types the phrase specified |
| "stop" | stops his answer even if he's talking |

# Data & config:
" ~/.local/share/thompson/ " (Linux)
" %APPDATA%"\thompson\ " ( Windows)

P.S.: When specifying the whisper model select the ABSOLUTE path for it to work.

# Tech stack:
- Rust
- HTML / CSS / JS
- Tauri 2
- whisper-rs
- cpal
- Ollama
- Gemini
- mpris / GSMTC
- espeak-ng / SAPI
- xcap

# Roadmap ( v0.2.0 ):
- more languages support
- barge-in
- audio i/o selection
- win32-window
- installer with GPU and system detection