import { invoke } from "@tauri-apps/api/core";
import type { CategorizedLauncherSettings } from "../types";

/**
 * Settings API
 * Pure Tauri invoke calls for launcher settings management
 */

export async function loadSettings(): Promise<CategorizedLauncherSettings> {
  return await invoke("load_settings");
}

export async function saveSettings(
  settings: CategorizedLauncherSettings,
): Promise<void> {
  return await invoke("save_settings_command", { settings });
}

export async function loadCustomCss(themeName: string): Promise<string> {
  return await invoke("load_custom_css", { themeName });
}

export async function setSelectedCssTheme(themeName: string): Promise<void> {
  return await invoke("set_selected_css_theme", { themeName });
}

export async function getSelectedCssTheme(): Promise<string> {
  return await invoke("get_selected_css_theme");
}

export async function getCssThemes(): Promise<string[]> {
  return await invoke("get_css_themes");
}

export async function saveCssTheme(
  themeName: string,
  cssContent: string,
): Promise<string> {
  return await invoke("save_css_theme", { themeName, cssContent });
}

export async function deleteCssTheme(themeName: string): Promise<void> {
  return await invoke("delete_css_theme", { themeName });
}

export async function loadCssTheme(themeName: string): Promise<string> {
  return await invoke("load_css_theme", { themeName });
}

export async function openCssThemesDirectory(): Promise<void> {
  return await invoke("open_css_themes_directory");
}
