const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const input = document.getElementById("cmd");
const log = document.getElementById("log");
const STATUS_LABELS = {
    listening:  "Слухаю",
    processing: "Обробляю...",
    wake:       "Так?",
    idle:       "",
    no_mic:     "Немає мікрофона",
}

function addMessage(text, role) {
    const el = document.createElement("div");
    el.className = `msg msg-${role}`;
    el.textContent = text;
    log.appendChild(el);
    log.scrollTop = log.scrollHeight;
}

input.addEventListener("keydown", async (e) => {
    if (e.key !== "Enter" || !input.value.trim()) return;
    sendCommand();
});

async function sendCommand() {
    if (!input.value.trim()) return;
    const text = input.value.trim();
    input.value = "";
    addMessage(text, "user");
    try {
        const answer = await invoke("execute_command", { text });
        if (answer) addMessage(answer, "tom")
    } catch (err) {
        addMessage(`[error] ${err}`, "error");
    }
}

async function toggleAiMode() {
    try {
        const mode = await invoke("toggle_ai_mode");
        setAiModeLabel(mode);
    } catch (err) {
        console.error(err);
    }
}

function setAiModeLabel(mode) {
    const on = mode === "gemini";
    if (mode === "gemini") {
        document.getElementById("aimode-label").textContent = "Gemini"
    } else {
        document.getElementById("aimode-label").textContent = "Ollama"
    }
    const sw = document.getElementById("btn-aimode");
    sw.classList.toggle("on", on);
    sw.setAttribute("aria-checked", on);
}

async function toggleMute() {
    try {
        const muted = await invoke("toggle_mute");
        setMuteIcon(muted);
    } catch (err) {
        console.error(err);
    }
}

function setStatus(status) {
    document.getElementById("status").textContent = STATUS_LABELS[status] ?? "";
    const btn = document.getElementById("btn-vinput");
    btn.dataset.status = status;
}


async function toggleListening(is_listening) {
    try {
        const on = await invoke("toggle_listening");
        setVoiceIcon(on);
    } catch (err) {
        console.error(err);
    }
}

function setVoiceIcon(on) {
    const icon = document.querySelector("#btn-vinput .icon");
    icon.classList.toggle("icon-vinput", on);
    icon.classList.toggle("icon-vinput-off", !on);
}

function setMuteIcon(muted) {
    const icon = document.querySelector("#btn-mute .icon");
    icon.classList.toggle("icon-mute", muted);
    icon.classList.toggle("icon-sound", !muted);
}

async function loadSettings() {
    const s = await invoke("get_settings");
    document.getElementById("set-model").value = s.local_model ?? "";
    document.getElementById("set-gemini-key").value = s.gemini_key ?? "";   
    document.getElementById("set-whisper-model").value = s.model_path ?? "";
    document.getElementById("set-accent").value = s.accent ?? "#9d9d9d";
    document.getElementById("set-bg").value = s.bg ?? "#2a2a2a";
    document.getElementById("set-muted").value = s.muted ?? "#4f4f4f";
}

async function saveSettings() {
    const data = {
        local_model: document.getElementById("set-model").value,
        gemini_key: document.getElementById("set-gemini-key").value,
        model_path: document.getElementById("set-whisper-model").value,
        accent: document.getElementById("set-accent").value,
        bg: document.getElementById("set-bg").value,
        muted: document.getElementById("set-muted").value,
    };
    await invoke("save_settings", { data });
    applyTheme(data);
    showPage("chat");
}

function openExternal(url) {
    invoke("open_url", { url });
}

function applyTheme(s) {
    const root = document.documentElement.style;
    if (s.accent) root.setProperty("--accent", s.accent);
    if (s.bg) root.setProperty("--bg", s.bg);
    if (s.muted) root.setProperty("--muted", s.muted);
}

async function init() {
    const s = await invoke("get_settings");
    applyTheme(s);
    setAiModeLabel(s.ai_mode);
    setMuteIcon(s.mute_status);
}

function showPage(page) {
    document.querySelectorAll(".page").forEach((el) => {
        el.classList.toggle("active", el.id === `page-${page}` )
    });
    if (page === "settings") loadSettings();
}

listen("reminder", (e) => addMessage(`🔔 ${e.payload}`, "tom"))
listen("addMessage", (e) => addMessage(e.payload[0], e.payload[1]))
listen("setStatus", (e) => setStatus(e.payload))

window.toggleAiMode = toggleAiMode;
window.toggleMute = toggleMute;
window.toggleListening = toggleListening;
window.showPage = showPage;
window.openExternal = openExternal;
showPage("chat")
init()