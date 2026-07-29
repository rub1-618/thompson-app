#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Stop,
    Ctime,
    Stats,
    WindowOpen,
    WindowHide,
    WindowClose,
    Screen,
    MusicToggle,
    MusicNext,
    MusicPrev,
    MusicInfo,
    Remind,
    OpenBrowser,
    Dictation,
    ToggleSound,
}

pub static CMDS: &[(Command, &[&str])] = &[
    (Command::Stop, &[
        "стоп", "стій", "зупинись", "замовчи", "shut up", "тихо", "мовчи",
        "stop speaking", "stop",
    ]),
    (Command::Ctime, &[
        "котра година", "скільки часу", "який час", "яка година",
        "what time is it", "time please", "поточний час", "час зараз",
    ]),
    (Command::Stats, &[
        "статистика", "стат", "stats", "системна інформація", "system info",
        "скільки памяті", "температура", "стан системи",
        "завантаження процесора", "завантаження системи",
    ]),
    (Command::WindowClose, &[
        "закрий вікно", "close window",
    ]),
    (Command::WindowHide, &[
        "мінімізуй", "заховай вікно", "згорни вікно", "сховай вікно", "hide window", "minimize window",
    ]),
    (Command::WindowOpen, &[
        "відкрий вікно", "покажи вікно", "переключи вікно", "open window", "show window",
    ]),
    (Command::Screen, &[
        "що на екрані", "аналіз екрану", "screenshot analysis", "analyze screen",
        "що бачиш", "подивись на екран", "скріншот",
    ]),
    (Command::MusicToggle, &[
        "музика", "пауза музика", "play", "pause", "відтвори", "плей",
        "стоп музику", "музику на паузу", "music",
    ]),
    (Command::MusicNext, &[
        "наступний трек", "next track", "next song", "наступна пісня",
    ]),
    (Command::MusicPrev, &[
        "попередній трек", "previous track", "prev song", "попередня пісня",
    ]),
    (Command::MusicInfo, &[
        "який трек", "що грає", "what's playing", "current song", "назва пісні",
    ]),
    (Command::Remind, &[
        "нагади", "нагадай", "нагадування", "remind me", "reminder",
        "встанови нагадування",
    ]),
    (Command::OpenBrowser, &[
        "відкрий браузер", "open browser", "браузер",
    ]),
    (Command::Dictation, &[
        "напиши", "надрукуй", "введи текст", "type this", "print this", "type",
    ]),
    (Command::ToggleSound, &[
        "вимкни звук", "вимкнути звук", "увімкнути звук", 
        "shut the sound down", "turn on the sound", "sound", "volume",
    ]),
];