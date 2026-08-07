#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Stop,
    Ctime,
    Stats,
    WindowOpen,
    WindowHide,
    WindowClose,
    Screen,
    ToggleSound,
    MusicToggle,
    MusicNext,
    MusicPrev,
    MusicInfo,
    Remind,
    OpenBrowser,
    Dictation,
}

pub static CMDS: &[(Command, &[&str])] = &[
    (Command::Stop, &[
        "stop", "stop speaking", "shut up", "silence", 
    ]),
    (Command::Ctime, &[
        "what time is it", "what is the time", "tell me the time"
    ]),
    (Command::Stats, &[
        "statistics", "stats", "sysinfo", "system information",
    ]),
    (Command::WindowClose, &[
        "close window", "close",
    ]),
    (Command::WindowHide, &[
        "hide window", "minimize window", "hide", "minimize",
    ]),
    (Command::WindowOpen, &[
        "open window", "show window", "open", "show",
    ]),
    (Command::Screen, &[
        "screen analysis", "what's on the screen", "check the screen",
    ]),
    (Command::ToggleSound, &[
        "turn off the system sound", "turn on the system sound", "toggle system sound", 
            "toggle sound", "turn the sound on", "turn the sound off",
    ]),
    (Command::MusicToggle, &[
        "turn off the music", "turn on the music", 
            "toggle music",
    ]),
    (Command::MusicNext, &[
        "next song", "next track",
    ]),
    (Command::MusicPrev, &[
        "previous song", "previous track", "prev song", "prev track",
    ]),
    (Command::MusicInfo, &[
        "what is the song", "what's playing", "song name", "track name",
    ]),
    (Command::Dictation, &[
        "type", "write", "input",
    ]),
    (Command::Remind, &[
        "remind me", "reminder", "set a timer", "set a reminder",
    ]),
    (Command::OpenBrowser, &[
        "open browser", "show browser", "browser",
    ]),
];