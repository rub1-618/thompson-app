use crate::dispatcher;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, mpsc};
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::sync::mpsc::UnboundedSender;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use rapidfuzz::fuzz::ratio;

const SPEECH_START: f32 = 0.008;
const SPEECH_END: f32 = 0.004;
const SILENCE_MS: usize = 800;
const IN_RATE: usize = 48000;
const OUT_RATE: usize = 16000;
const WAKE_WORDS: [&str; 5] = ["tom", "tomi", "тому", "tom", "thompson"]; // дом + тому -- for better model perfomance
const STOP_WORDS: [&str; 4] = ["stop", "stop speaking", "shut up", "silence"];
const FILLERS: [&str; 11] = [
    "tom", "tomi", "тому", "tom", "thompson", "say", "tell",
    "please", "can", "you", "pls"
];
const WAKE_THRESHOLD: f64 = 0.55;
const STOP_THRESHOLD: f64 = 0.80;

pub enum VoiceEvent {
    Status(&'static str), // listening | processing | wake | idle | no_mic
    Command(String),
    Stop,
}

#[derive(PartialEq)]
pub enum VoiceState {
    Idle, Awake
}

pub enum VoiceAction {
    None,
    Stop,
    Wake,
    Dispatch(String),
}

fn is_stop(text: &str) -> bool {
    dispatcher::match_whole_phrase(text, &STOP_WORDS, STOP_THRESHOLD)
}

fn is_wake(text: &str) -> bool {
    let first3: Vec<String> = text
        .split_whitespace()
        .take(3)
        .map(|w| w.to_lowercase())
        .collect();

    for word in first3 {
        for wake in WAKE_WORDS {
            if ratio(word.chars(), wake.chars()) >= WAKE_THRESHOLD {
                return true
            }
        }
    }
    false
}

fn strip_fillers(text: &str) -> String {
    text.split_whitespace()
        .filter(|w| !FILLERS.contains(&w.to_lowercase().as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn process(state: &mut VoiceState, text: &str) -> VoiceAction {
    let text = text.trim();
    if is_stop(text) {
        *state = VoiceState::Idle;
        return VoiceAction::Stop;
    }

    match state {
        VoiceState::Awake => {
            *state = VoiceState::Idle;
            return VoiceAction::Dispatch(strip_fillers(text));
        }

        VoiceState::Idle => {
            if is_wake(text) {
                let command = strip_fillers(text);
                if command.is_empty() {
                    *state = VoiceState::Awake;
                    return VoiceAction::Wake
                } else {
                    return VoiceAction::Dispatch(command);
                }
            } else {
                return VoiceAction::None
            }
        }
    }
}

fn rms(buf: &[f32]) -> f32 {
    if buf.is_empty() {  return 0.0; }
    (buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32).sqrt()
}

fn transcribe(state: &mut whisper_rs::WhisperState, audio: &[f32]) -> String {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    state.full(params, audio).expect("whisper full");
    let mut text = String::new();
    for segment in state.as_iter() {
        text.push_str(&segment.to_string());
    }
    text
}

pub fn listen_loop(running: Arc<AtomicBool>, events: UnboundedSender<VoiceEvent>, 
    speaking: Arc<AtomicBool>, model_path: String) {
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            let _ = events.send(VoiceEvent::Status("no mic"));
            return;
        }
    };
    let config = device.default_input_config().expect("config");
    let channels = config.channels() as usize;

    let (sample_tx, sample_rx) = mpsc::channel::<Vec<f32>>();
    let stream = device.build_input_stream(
        config.into(), 
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mono: Vec<f32> = data
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
                .collect();
            let _ = sample_tx.send(mono);
        },
        move |err| eprintln!("stream error: {err}"), 
        None
    ).expect("stream");
    stream.play().expect("play");

    let ctx = WhisperContext::new_with_params(
        &model_path,
        WhisperContextParameters::default(),
    ).expect("whisper model");
    let mut wstate = ctx.create_state().expect("state");

    let mut vstate = VoiceState::Idle;
    let mut in_speech = false;
    let mut utter: Vec<f32> = vec![];
    let mut silence = 0usize;
    let silence_limit = IN_RATE * SILENCE_MS / 1000;

    let _ = events.send(VoiceEvent::Status("listening"));

    while running.load(Ordering::Relaxed) {
        let chunk = match sample_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(c) => {c}
            Err(mpsc::RecvTimeoutError::Timeout) => {continue}
            Err(mpsc::RecvTimeoutError::Disconnected) => {break}
        };

        if speaking.load(Ordering::Relaxed) {
            in_speech = false;
            utter.clear();
            silence = 0;
            continue;
        }
        
        let level = rms(&chunk);

        if level > SPEECH_START {
            in_speech = true;
            silence = 0;
        }
        if in_speech {
            utter.extend_from_slice(&chunk);
            if level < SPEECH_END {
                silence += chunk.len();
                if silence >= silence_limit {
                    let audio: Vec<f32> = utter.iter()
                        .step_by(IN_RATE / OUT_RATE)
                        .copied().collect();
                    let _ = events.send(VoiceEvent::Status("processing"));
                    let text = transcribe(&mut wstate, &audio);

                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        match process(&mut vstate, trimmed) {
                            VoiceAction::None => {}
                            VoiceAction::Stop => {let _ = events.send(VoiceEvent::Stop);},
                            VoiceAction::Wake => {let _ = events.send(VoiceEvent::Status("wake"));}
                            VoiceAction::Dispatch(cmd)  => {let _ = events.send(VoiceEvent::Command(cmd));},
                        }
                    }
                    utter.clear();
                    in_speech = false;
                    silence = 0;
                    let _ = events.send(VoiceEvent::Status("listening"));
                }
            } else {
                silence = 0;
            }
        }
    }
    let _ = events.send(VoiceEvent::Status("idle"));
}