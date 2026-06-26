export type Light = "green" | "yellow" | "red";

export interface StageSound {
  preset: string;
  custom_path: string | null;
}

export interface SoundsConfig {
  enabled: boolean;
  green: StageSound;
  yellow: StageSound;
  red: StageSound;
}

export interface Config {
  idle_timeout_secs: number;
  stealth: boolean;
  stealth_acknowledged: boolean;
  theme: string;
  locale: string;
  window: { x: number; y: number };
  sounds: SoundsConfig;
}

export const SOUND_PRESETS = [
  "soft-chime",
  "double-ping",
  "alert",
  "chime",
  "bell",
  "ping",
  "pop",
] as const;

export type SoundPreset = (typeof SOUND_PRESETS)[number];
