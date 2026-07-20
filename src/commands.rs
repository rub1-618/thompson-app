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
}

pub static CMDS: &[(Command, &[&str])] = &[
    (Command::Stop, &[
        "стоп", "стій", "зупинись", "замовчи", "тихо", "мовчи",
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
        "закрий вікно",
    ]),
    (Command::WindowHide, &[
        "мінімізуй", "заховай вікно", "згорни вікно", "сховай вікно",
    ]),
    (Command::WindowOpen, &[
        "відкрий вікно", "покажи вікно", "переключи вікно",
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
        "напиши", "надрукуй", "введи текст", "type this", "print this",
    ]),
];
