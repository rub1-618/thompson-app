use std::collections::VecDeque;
use std::process::{Child, Command};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub enum TtsCommand {
    Speak(String),
    Stop,
    SetMute(bool),
}

pub fn spawn_tts(voice: String, rate: i32, speaking: Arc<AtomicBool>) -> Sender<TtsCommand> {
    let (tx, rx) = std::sync::mpsc::channel::<TtsCommand>();
    std::thread::spawn(move || worker(rx, voice, rate, speaking));
    tx
}

fn worker(rx: Receiver<TtsCommand>, voice: String, rate: i32, speaking: Arc<AtomicBool>) {
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut current: Option<Child> = None;
    let mut muted = true;

    loop {
        loop {
            match rx.try_recv() {
                Ok(TtsCommand::Speak(text)) => queue.push_back(text),
                Ok(TtsCommand::SetMute(m)) => {
                    muted = m;
                    if muted {
                        queue.clear();
                        if let Some(mut c) = current.take() {let _ = c.kill();}
                    }
                }
                Ok(TtsCommand::Stop) => {
                    queue.clear();
                    if let Some(mut c) = current.take() {
                        let _ = c.kill();
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        if let Some(c) = &mut current {
            if let Ok(Some(_)) = c.try_wait() {
                current = None;
            }
        }

        if current.is_none() {
            if let Some(text) = queue.pop_front() {
                if !muted {
                    if let Ok(child) = synth_command(&text, &voice, rate).spawn() {
                        current = Some(child);
                    }
                }
            }
        }
        speaking.store(current.is_some(), Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(not(target_os = "windows"))]
fn synth_command(text: &str, _voice: &str, rate: i32) -> Command {
    let wpm = (175 + rate * 12).clamp(80, 400); // SAPI-rate | words per minute
    let mut cmd = Command::new("espeak-ng");
    cmd.args(["-v", "en", "-s", &wpm.to_string(), text]);
    cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    cmd
}

#[cfg(target_os = "windows")]
fn synth_command(text: &str, voice: &str, rate: i32) -> Command {
    let safe = text.replace('\'', "''").replace(['\r', '\n', '\0'], "");
    let script = format!(
        "Add-Type -AssemblyName System.Speech; \
        $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
        $v = $s.GetInstalledVoices() | Where-Object {{ $_.VoiceInfo.Name -like '*{voice}*' }} | Select-Object -First 1; \
        if ($v) {{ $s.SelectVoice($v.VoiceInfo.Name) }}; \
        $s.Rate = {rate}; \
        $s.Speak('{safe}')"
    );

    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", &script]);
    cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    cmd
}