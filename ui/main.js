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

document.getElementById("cmd-trigger-input")
    ?.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") addTrigger();
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

let newTriggers = [];

async function saveCommand() {
    const name = document.getElementById("cmd-name").value.trim();
    const path = document.getElementById("cmd-path").value.trim();
    if (!name || !path || !newTriggers.length) return;
    await invoke("save_command", { name, path, triggers: newTriggers });
    document.getElementById("cmd-name").value = "";
    document.getElementById("cmd-path").value = "";
    newTriggers = [];
    renderTriggers();
    loadCommands();
}

async function loadCommands() {
    const cmds = await invoke("get_commands");
    const list =document.getElementById("commands-list");
    list.innerHTML = "";
    if (!cmds.length) {
        list.innerHTML = '<p class="cmd-empty">Поки що немає команд.</p>';
        return;
    }
    cmds.forEach((c, i) => {
        const item = document.createElement("div")
        item.className = "cmd-item";

        const info = document.createElement("div");
        info.className = "cmd-info";
        const name = document.createElement("strong"); name.textContent = c.name;
        const trig = document.createElement("span"); trig.textContent = c.triggers.join(", ");
        const path = document.createElement("code"); path.textContent = c.path;
        info.append(name, trig, path);

        const del = document.createElement("button");
        del.className = "cmd-del";
        del.textContent = "✕";
        del.onclick = () => deleteCommand(i);

        item.append(info, del);
        list.appendChild(item);
    });
}

async function deleteCommand(index) {
    await invoke("delete_command", { index });
    loadCommands();
}

function addTrigger() {
    const input = document.getElementById("cmd-trigger-input");
    const val = input.value.trim();
    if (!val || newTriggers.includes(val)) return;
    newTriggers.push(val);
    input.value = "";
    renderTriggers();
}

function removeTrigger(index) {
    newTriggers.splice(index, 1);
    renderTriggers();
}

function renderTriggers() {
    const box = document.getElementById("trigger-chips");
    box.innerHTML = "";
    newTriggers.forEach((t, i) => {
        const chip = document.createElement("span");
        chip.className = "chip";
        chip.textContent = t;
        const x = document.createElement("button");
        x.type = "button";
        x.textContent = "✕";
        x.onclick = () => removeTrigger(i);
        chip.appendChild(x);
        box.appendChild(chip);
    });
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
    if (page === "commands") loadCommands();
}

listen("reminder", (e) => addMessage(`🔔 ${e.payload}`, "tom"))
listen("addMessage", (e) => addMessage(e.payload[0], e.payload[1]))
listen("setStatus", (e) => setStatus(e.payload))

window.toggleAiMode = toggleAiMode;
window.toggleMute = toggleMute;
window.toggleListening = toggleListening;
window.saveCommand = saveCommand;
window.deleteCommand = deleteCommand;
window.addTrigger = addTrigger;
window.showPage = showPage;
window.openExternal = openExternal;
showPage("chat")
init()