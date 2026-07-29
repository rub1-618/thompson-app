use crate::core::SettingsEntry;
use crate::commands::Command;

// ! stats-getting

use sysinfo::{Components, MINIMUM_CPU_UPDATE_INTERVAL, System}; // stats

pub fn stats() -> String {
    let mut sys = System::new();

    sys.refresh_cpu_all();
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();
    let cpu = sys.global_cpu_usage();

    sys.refresh_memory();
    let used = sys.used_memory() as f64 / 1073741824.0;
    let total = sys.total_memory() as f64 / 1073741824.0;

    let temp_str = match cpu_temp() {
        Some(t) => format!("{t:.0}"),
        None => "?".to_string()
    };

    format!("CPU: {cpu:.0}%, RAM: {used:.1}/{total:.1} GB, temp: {temp_str}°C")
}

fn cpu_temp() -> Option<f32> {
    let components = Components::new_with_refreshed_list();
    for c in &components {
        let label = c.label().to_lowercase();
        if label.contains("cpu") || label.contains("package")
            || label.contains("tctl") || label.contains("k10temp") {
            return c.temperature();
        }
    }
    None
}

// ! window manipulation

#[derive(Clone, Copy)]
enum WindowAction { Show, Hide, Close }

pub fn window(cmd: Command, text: &str) -> String {
    let action = match cmd {
        Command::WindowOpen  => WindowAction::Show,
        Command::WindowHide  => WindowAction::Hide,
        Command::WindowClose => WindowAction::Close,
        _ => unreachable!()
    };
    let title = strip_window_trigger(text);
    if title.is_empty() {
        return "Вкажіть назву вікна у коммандi пiсля 'window'.".to_string();
    }
    window_impl::apply(action, &title)
}

fn strip_window_trigger(text: &str) -> String {
    const TRIGGERS: [&str; 13] = [
        "закрий вікно", "мінімізуй", "close window",
        "заховай вікно", "згорни вікно", "hide window",
        "сховай вікно", "відкрий вікно", "minimize window",
        "покажи вікно", "переключи вікно", "open window", "show window",
    ];
    let lower= text.to_lowercase();
    for trigger in TRIGGERS {
        if lower.starts_with(trigger) {
            return text[trigger.len()..].trim_start().to_lowercase().to_string()
        }
    }
    "".to_string()
}

#[cfg(not(target_os = "windows"))]
mod window_impl {
    use super::WindowAction;
    use std::process::Command;

    pub fn apply(action: WindowAction, title: &str) -> String {
        if let WindowAction::Show = action {
            return match Command::new(title).spawn() {
                Ok(_) => format!("Відкриваю «{title}»."),
                Err(_) => format!("Не вдалося відкрити «{title}»."),
            };
        }

        if is_hyprland() {
            return hypr_apply(action, title);
        }

        let out = match Command::new("xdotool")
            .args(["search", "--onlyvisible", "--name", title])
            .output()
                {
                    Ok(o) => o,
                    Err(_) => return "xdotool не встановлено.".to_string(),
                };
        let ids = String::from_utf8_lossy(&out.stdout);
        let id = match ids.lines().next() {
            Some(id) => {
                if !id.is_empty() {
                    id
                } else {
                    return format!("Вікно «{title}» не знайдено.")
                }
            }
            _ => return format!("Вікно «{title}» не знайдено.")
        };

        let sub = match action {
            WindowAction::Hide  => "windowminimize",
            WindowAction::Close => "windowclose",
            WindowAction::Show  => unreachable!(),
        };
        match Command::new("xdotool").args([sub, id]).status() {
            Ok(s) => {
                if s.success() {
                    "Готово.".to_string()
                } else {
                    "Не вдалося керувати вікном.".to_string()
                }
            }
            _ => "Не вдалося керувати вікном.".to_string()
        }
    }

    fn hypr_apply(action: WindowAction, title: &str) -> String {
        let addr = match hypr_find_address(title) {
            Some(a) => a,
            None => return format!("Вікно «{title}» не знайдено.")
        };

        let (dispatcher, arg) = match action {
            WindowAction::Close => ("closewindow", format!("address:{addr}")),
            WindowAction::Hide  => ("movetoworkspacesilent", format!("special:minimized,address:{addr}")),
            WindowAction::Show  => unreachable!(),
        };
        match Command::new("hyprctl").args(["dispatch", dispatcher, &arg]).status() {
            Ok(s) => {
                if s.success() {
                    "Готово.".to_string()
                } else {
                    "Не вдалося керувати вікном.".to_string()
                }
            }
            _ => "Не вдалося керувати вікном.".to_string()
        }

    }

    fn hypr_find_address(needle: &str) -> Option<String> {
        let out = Command::new("hyprctl").arg("clients").output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let needle = needle.to_lowercase();

        let mut addr: Option<String> = None;
        let mut is_tom = false;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("Window ") {
                addr = rest.split_whitespace().next().map(|s| format!("0x{s}"));
                is_tom = false;
            } else {
                let l = line.trim();
                if l == "class: thompson-app" {
                    is_tom = true;
                }
                if !is_tom {
                    if let Some(v) = l.strip_prefix("class: ").or_else(|| l.strip_prefix("title: ")) {
                        if v.to_lowercase().contains(&needle) {
                            return addr;
                        }
                    }
                }
            }
        }
        None
    }

    fn is_hyprland() -> bool {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
    }
}

#[cfg(target_os = "windows")]
mod window_impl {
    use super::WindowAction;
    use std::process::Command;
    use windows::Win32::Foundation::{HWND, LPARAM, BOOL, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, IsWindowVisible,
        ShowWindow, PostMessageW, SW_MINIMIZE, WM_CLOSE,
    };
    use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_FLAG_NO_UI};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{HSTRING, PCWSTR, w};

    struct Search<'a> {
        needle: &'a str,
        found: Option<HWND>,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let search = &mut *(lparam.0 as *mut Search);

            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }

            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buf);
            let title = String::from_utf16_lossy(&buf[..len as usize]);   

            if title.to_lowercase().contains(&search.needle.to_lowercase()) {
                search.found = Some(hwnd);
                return BOOL(0);
            }
            BOOL(1)
        }
    }

    pub fn apply(action: WindowAction, title: &str) -> String {
        if let WindowAction::Show = action {
            return win_show(title);
        }

        let mut search = Search { needle: title, found: None };
        unsafe {
            let _ = EnumWindows(
                Some(enum_cb),
                LPARAM(&mut search as *mut _ as isize),
            );
        }

        let hwnd = match search.found {
            Some(h) => h,
            None => return format!("Вікно «{title}» не знайдено."),
        };

        let ok = unsafe {
            match action {
                WindowAction::Hide  => { let _ = ShowWindow(hwnd, SW_MINIMIZE); true }
                WindowAction::Close => PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).is_ok(),
                WindowAction::Show  => unreachable!()
            }
        };
        if ok {
            "Готово.".to_string()
        } else {
            "Не вдалося керувати вікном.".to_string()
        }
    }

    fn win_show(title: &str) -> String {
        let file = HSTRING::from(title);
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask:  SEE_MASK_FLAG_NO_UI,
            lpVerb: w!("open"),
            lpFile: PCWSTR(file.as_ptr()),
            nShow: SW_SHOWNORMAL.0, 
            ..Default::default()
        };
        match unsafe { ShellExecuteExW(&mut sei) } {
            Ok(_) => format!("Відкриваю «{title}»."),
            Err(_) => format!("Не вдалося відкрити «{title}»."),
        };
    }
}

// ! screen analysis with Gemini Vision

use xcap::Monitor;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Cursor;

pub async fn screen(settings: &SettingsEntry) -> String {
    let b64 = match tokio::task::spawn_blocking(capture_screen_base64).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return format!("Не вдалось зробити скріншот: {}", e),
        Err(_) => return "Помилка захоплення екрана.".to_string(),
    };

    crate::ai::ask_gemini_vision(
        &b64, 
        "Що ти бачиш на цьому скріншоті? Опиши коротко.", 
        settings
    ).await
}

pub fn capture_screen_base64() -> Result<String, String> {
    let monitors = match Monitor::all() {
        Ok(m) => m,
        Err(e) => return Err(format!("{}", e)),
    };

    let monitor = match monitors.into_iter().next() {
        Some(m) => m,
        None => return Err("Monitor not found.".to_string()),
    };

    let image = match monitor.capture_image() {
        Ok(i) => i,
        Err(e) => return Err(format!("{}", e)),
    };

    let mut buf = Cursor::new(Vec::new());
    image.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string()).unwrap_or_default();

    let base64_str = STANDARD.encode(buf.into_inner());

    Ok(base64_str)
}

// ! music manipulations

#[cfg(not(target_os = "windows"))]
mod media_impl {
    use mpris::{Player, PlayerFinder};
    
    fn active_player() -> Result<Player, String> {
        let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
        finder.find_active().map_err(|e| e.to_string())
    }

    pub fn media_toggle() -> String {
        match active_player() {
            Ok(pl) => match pl.play_pause() {
                Ok(_) => "Музика перемкнена".to_string(),
                Err(_) => "Не керувати музикою.".to_string(),
            }
            Err(_) => "Немає активного плеєра.".to_string()
        }
    }

    pub fn next() -> String {
        match active_player() {
            Ok(pl) => match pl.next() {
                Ok(_) => "Музика перемкнена".to_string(),
                Err(_) => "Не вдалося перемкнути музику.".to_string(),
            }
            Err(_) => "Немає активного плеєра.".to_string()
        }
    }

    pub fn prev() -> String {
        match active_player() {
            Ok(pl) => match pl.previous() {
                Ok(_) => "Музика перемкнена".to_string(),
                Err(_) => "Не вдалося перемкнути музику.".to_string(),
            }
            Err(_) => "Немає активного плеєра.".to_string()
        }
    }

    pub fn info() -> String {
        let player = match active_player() {
            Ok(pl) => pl,
            Err(_) => return "Немає активного плеєра.".to_string()
        };

        let meta = match player.get_metadata() {
            Ok(m) => m,
            Err(_) => return "Немає інформації про трек.".to_string()
        };

        let title = meta.title().unwrap_or("Назва треку невідомa.");
        let artist = meta.artists().and_then(|a| a.first().copied()).unwrap_or("");
        if artist.is_empty() {
            format!("Зараз грає: {title}")
        } else {
            format!("Зараз грає: {title} від {artist}.")
        }
    }

    pub fn sound_toggle() -> String {
        match std::process::Command::new("wpctl")
            .args([
                "set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"
            ]).status()
                {
                    Ok(s) => {
                        if s.success() {
                            "Звук перемкнено.".to_string()
                        } else {
                            "Не вдалося керувати звуком.".to_string()
                        }
                    },
                    _ => "Не вдалося керувати звуком.".to_string()
                }
    }
}

#[cfg(target_os = "windows")]
mod media_impl {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        keybd_event, 
        KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, 
        VK_MEDIA_NEXT_TRACK, VK_MEDIA_PREV_TRACK, 
        VK_MEDIA_PLAY_PAUSE, VK_VOLUME_MUTE,
    };
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

    pub fn media_toggle() -> String {
        unsafe {
            keybd_event(VK_MEDIA_PLAY_PAUSE.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_MEDIA_PLAY_PAUSE.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
        "Музика перемкнена".to_string()
    }

    pub fn next() -> String {
        unsafe {
            keybd_event(VK_MEDIA_NEXT_TRACK.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_MEDIA_NEXT_TRACK.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
        "Музика перемкнена".to_string()
    }

    pub fn prev() -> String {
        unsafe {
            keybd_event(VK_MEDIA_PREV_TRACK.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_MEDIA_PREV_TRACK.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
        "Музика перемкнена".to_string()
    }

    pub fn info() -> String {
        let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(op) => {match op.get() {
                Ok(m) => m,
                Err(_) => return "Немає інформації про трек.".to_string()
            }}
            Err(_) => return "Немає інформації про трек.".to_string()
        };
        let session = match manager.GetCurrentSession() {
            Ok(s) => s,
            Err(_) => return "Немає активного плеєра.".to_string()
        };
        let props = match session.TryGetMediaPropertiesAsync() {
            Ok(op) => match op.get() {
                Ok(p) => p,
                Err(_) => return "Немає інформації про трек.".to_string()
            },
            Err(_) => return "Немає інформації про трек.".to_string()
        };
        let title = props.Title().map(|e| e.to_string()).unwrap_or_default();
        let artist = props.AlbumArtist().map(|e| e.to_string()).unwrap_or_default();
        if artist.is_empty() {
            format!("Зараз грає: {title}")
        } else {
            format!("Зараз грає: {title} від {artist}.")
        }
    }

    pub fn sound_toggle() -> String {
        unsafe {
            keybd_event(VK_VOLUME_MUTE.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(VK_VOLUME_MUTE.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
        "Звук перемкнено.".to_string()
    }
}

pub fn music_toggle() -> String { media_impl::media_toggle() }
pub fn music_next()   -> String { media_impl::next() }
pub fn music_prev()   -> String { media_impl::prev() }
pub fn music_info()   -> String { media_impl::info() }
pub fn sound_toggle() -> String { media_impl::sound_toggle() }