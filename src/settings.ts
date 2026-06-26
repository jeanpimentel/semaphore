import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "./themes";
import { t, type Locale } from "./i18n";

interface Config {
  idle_timeout_secs: number;
  stealth: boolean;
  stealth_acknowledged: boolean;
  theme: string;
  locale: string;
  window: { x: number; y: number };
}

let currentLocale: Locale = "en";

function applyLocale(locale: Locale): void {
  currentLocale = locale;
  const strings = t(locale);
  document.getElementById("settings-title")!.textContent = strings.settings.title;
  document.getElementById("label-theme")!.textContent = strings.settings.theme;
  document.getElementById("label-language")!.textContent = strings.settings.language;
  document.getElementById("label-stealth")!.textContent = strings.settings.stealth;
  document.getElementById("label-connect")!.textContent = strings.settings.connect;
  document.getElementById("btn-cancel")!.textContent = strings.settings.cancel;
  document.getElementById("btn-save")!.textContent = strings.settings.save;
  document.getElementById("stealth-note")!.textContent = strings.settings.stealthNote;
  document.getElementById("connect-cursor")!.textContent = strings.tools.cursor;
  document.getElementById("connect-claude")!.textContent = strings.tools.claude;
  document.getElementById("connect-codex")!.textContent = strings.tools.codex;
  document.getElementById("connect-gemini")!.textContent = strings.tools.gemini;
  document.getElementById("connect-copilot")!.textContent = strings.tools.copilot;
  document.getElementById("connect-all")!.textContent = strings.tools.all;
  document.getElementById("about-title")!.textContent = strings.about.title;
  document.getElementById("about-description")!.textContent = strings.about.description;
  document.getElementById("about-controls-title")!.textContent = strings.about.controlsTitle;
  document.getElementById("about-tray-title")!.textContent = strings.about.trayTitle;

  const lightsList = document.getElementById("about-lights")!;
  lightsList.innerHTML = "";
  for (const item of strings.about.lights) {
    const li = document.createElement("li");
    li.textContent = item;
    lightsList.appendChild(li);
  }

  const controlsList = document.getElementById("about-controls")!;
  controlsList.innerHTML = "";
  for (const item of strings.about.controls) {
    const li = document.createElement("li");
    li.textContent = item;
    controlsList.appendChild(li);
  }

  const trayList = document.getElementById("about-tray")!;
  trayList.innerHTML = "";
  for (const item of strings.about.trayMenu) {
    const li = document.createElement("li");
    li.textContent = item;
    trayList.appendChild(li);
  }
}

async function loadConfig(): Promise<Config> {
  const config = await invoke<Config>("get_config");
  applyTheme(config.theme);
  applyLocale((config.locale as Locale) || "en");
  (document.getElementById("theme-select") as HTMLSelectElement).value = config.theme;
  (document.getElementById("locale-select") as HTMLSelectElement).value = config.locale;
  (document.getElementById("stealth-checkbox") as HTMLInputElement).checked = config.stealth;
  return config;
}

async function maybeAcknowledgeStealth(config: Config): Promise<Config> {
  const checkbox = document.getElementById("stealth-checkbox") as HTMLInputElement;
  if (!checkbox.checked || config.stealth_acknowledged) {
    return config;
  }
  const strings = t(currentLocale);
  const ok = confirm(strings.settings.stealthNote);
  if (!ok) {
    checkbox.checked = false;
    config.stealth = false;
    return config;
  }
  config.stealth_acknowledged = true;
  return config;
}

async function saveConfigFromForm(): Promise<void> {
  let config = await invoke<Config>("get_config");
  config.theme = (document.getElementById("theme-select") as HTMLSelectElement).value;
  config.locale = (document.getElementById("locale-select") as HTMLSelectElement).value;
  config.stealth = (document.getElementById("stealth-checkbox") as HTMLInputElement).checked;
  config = await maybeAcknowledgeStealth(config);
  await invoke("save_config", { config });
  applyTheme(config.theme);
  applyLocale(config.locale as Locale);
  await invoke("set_stealth", { enabled: config.stealth });
  await emit("config-changed", config);
}

async function connectTool(tool: string): Promise<void> {
  const strings = t(currentLocale);
  try {
    await invoke("install_hooks", { tool });
    alert(strings.tools.connected);
  } catch {
    alert(strings.tools.failed);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  await loadConfig();

  document.getElementById("locale-select")?.addEventListener("change", (e) => {
    applyLocale((e.target as HTMLSelectElement).value as Locale);
  });

  document.getElementById("settings-form")?.addEventListener("submit", async (e) => {
    e.preventDefault();
    await saveConfigFromForm();
    await getCurrentWindow().close();
  });

  document.getElementById("btn-cancel")?.addEventListener("click", () => {
    getCurrentWindow().close();
  });

  document.getElementById("connect-cursor")?.addEventListener("click", () => connectTool("cursor"));
  document.getElementById("connect-claude")?.addEventListener("click", () => connectTool("claude-code"));
  document.getElementById("connect-codex")?.addEventListener("click", () => connectTool("codex"));
  document.getElementById("connect-gemini")?.addEventListener("click", () => connectTool("gemini-cli"));
  document.getElementById("connect-copilot")?.addEventListener("click", () => connectTool("copilot-cli"));
  document.getElementById("connect-all")?.addEventListener("click", () => connectTool("all"));
});
