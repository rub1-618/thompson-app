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
        "стоп", "зупинись", "тиша", "замовчи",
        "stop", "stop speaking", "shut up", "silence", 
    ]),
    (Command::Ctime, &[
        "котра година", "скільки часу", "який час",
        "what time is it", "what is the time", "tell me the time"
    ]),
    (Command::Stats, &[
        "статистика", "стати", "системна інформація", "системна інфа",
        "statistics", "stats", "sysinfo", "system information",
    ]),
    (Command::WindowClose, &[
        "закрий вікно", "закрий",
        "close window", "close",
    ]),
    (Command::WindowHide, &[
        "сховай вікно", "мінімізуй вікно", "сховай", "мінімізуй",
        "hide window", "minimize window", "hide", "minimize",
    ]),
    (Command::WindowOpen, &[
        "відкрий вікно", "покажи вікно", "відкрий", "покажи",
        "open window", "show window", "open", "show",
    ]),
    (Command::Screen, &[
        "аналіз екрану", "що на екрані", "подивись на екран",
        "screen analysis", "what's on the screen", "check the screen",
    ]),
    (Command::ToggleSound, &[
        "вимкни звук системи", "увімкни звук системи", "вимкни звук", "увімкни звук",
        "turn off the system sound", "turn on the system sound", "toggle system sound", 
            "toggle sound", "turn the sound on", "turn the sound off",
    ]),
    (Command::MusicToggle, &[
        "вимкни музику", "увімкни музику",
        "turn off the music", "turn on the music", 
            "toggle music",
    ]),
    (Command::MusicNext, &[
        "наступна пісня", "наступний трек",
        "next song", "next track",
    ]),
    (Command::MusicPrev, &[
        "попередня пісня", "попередній трек",
        "previous song", "previous track", "prev song", "prev track",
    ]),
    (Command::MusicInfo, &[
        "який трек", "що грає", "назва пісні", "назва треку",
        "what is the song", "what's playing", "song name", "track name",
    ]),
    (Command::Dictation, &[
        "напиши", "надрукуй", "введи",
        "type", "write", "input",
    ]),
    (Command::Remind, &[
        "нагадай", "напам'ятай", "нагадування", "встанови нагадування через",
        "remind me", "reminder", "set a timer", "set a reminder",
    ]),
    (Command::OpenBrowser, &[
        "відкрий бразуер", "покажи браузер", "бразуер",
        "open browser", "show browser", "browser",
    ]),
];