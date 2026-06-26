import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "./themes";
import { t, type Locale } from "./i18n";

type Light = "green" | "yellow" | "red";

interface Config {
  idle_timeout_secs: number;
  stealth: boolean;
  stealth_acknowledged: boolean;
  theme: string;
  locale: string;
  window: { x: number; y: number };
}

function setActiveLight(state: Light): void {
  document.querySelectorAll<HTMLElement>("[data-light]").forEach((el) => {
    const light = el.dataset.light as Light;
    el.classList.toggle("active", light === state);
  });
}

function applyMainLocale(locale: Locale): void {
  const strings = t(locale);
  const housing = document.querySelector(".housing") as HTMLElement | null;
  if (housing) {
    housing.title = strings.main.dragHint;
  }
  const settingsBtn = document.getElementById("settings-btn");
  if (settingsBtn) {
    settingsBtn.title = strings.main.settingsHint;
  }
}

async function loadConfig(): Promise<Config> {
  const config = await invoke<Config>("get_config");
  applyTheme(config.theme);
  applyMainLocale((config.locale as Locale) || "en");
  return config;
}

function setupDrag(): void {
  const housing = document.querySelector(".housing") as HTMLElement | null;
  housing?.addEventListener("mousedown", async (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    await getCurrentWindow().startDragging();
  });
}

window.addEventListener("DOMContentLoaded", async () => {
  await loadConfig();
  setActiveLight("green");
  setupDrag();

  await listen<{ state: Light }>("state-changed", (event) => {
    setActiveLight(event.payload.state);
  });

  await listen<Config>("config-changed", (event) => {
    applyTheme(event.payload.theme);
    applyMainLocale(event.payload.locale as Locale);
  });

  document.getElementById("settings-btn")?.addEventListener("click", () => {
    invoke("show_settings");
  });

  const window = getCurrentWindow();
  window.onMoved(async () => {
    const pos = await window.outerPosition();
    const config = await invoke<Config>("get_config");
    config.window = { x: pos.x, y: pos.y };
    await invoke("save_config", { config });
  });
});
