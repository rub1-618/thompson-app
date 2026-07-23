use sysinfo::{Components, System, MINIMUM_CPU_UPDATE_INTERVAL}; // stats

pub fn stats() -> String {
    let mut sys = System::new();

    // ! CPU
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

pub fn window(text: &str) -> String {
    // todo: win32 / compositor on Linux;
    let _ = text;
    "Керування вікнами ще не реалізовано.".to_string()
}

pub fn screen() -> String {
    // todo: screenshot + Gemini Vision (future)
    "Аналіз екрану ще не доступний.".to_string()
}

#[cfg(not(target_os = "windows"))]
mod music_impl {
    use mpris::{Player, PlayerFinder};
    
    fn active_player() -> Result<Player, String> {
        let finder = PlayerFinder::new().map_err(|e| e.to_string())?;
        finder.find_active().map_err(|e| e.to_string())
    }

    pub fn toggle() -> String {
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
}

#[cfg(target_os = "windows")]
mod music_impl {
    use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PREV_TRACK, VK_MEDIA_PLAY_PAUSE};
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;


    pub fn toggle() -> String {
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
}

pub fn music_toggle() -> String { music_impl::toggle() }
pub fn music_next()   -> String { music_impl::next() }
pub fn music_prev()   -> String { music_impl::prev() }
pub fn music_info()   -> String { music_impl::info() }