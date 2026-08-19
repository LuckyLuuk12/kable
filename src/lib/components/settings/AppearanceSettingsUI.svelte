<script lang="ts">
import { Icon, api, app, type IconTemplate } from "$lib";
import { onMount } from "svelte";

let showCustomTemplates = $state(false);
let showIconUpload = $state(false);
let uploadError = $state("");
let uploadFile = $state<File | null>(null);
let isDragOver = $state(false);

let showCssUpload = $state(false);
let cssUploadError = $state("");
let cssUploadFile = $state<File | null>(null);
let cssIsDragOver = $state(false);

let cssThemes = $state<string[]>(["default"]);
let showCustomThemes = $state(false);

let iconTemplates = $state<IconTemplate[]>([]);

let saveStatus = $state("");

const builtInThemes = ["default", "KasaiSora-Theme", "Modrinth-Theme"];

const customThemes = $derived(cssThemes.filter((theme) => !builtInThemes.includes(theme)));

const hasCustomThemes = $derived(customThemes.length > 0);

const customIconTemplates = $derived(iconTemplates.filter((template) => template.type === "custom"));

function scheduleSave() {
  app.customizationService.scheduleSave();
}

function setStatus(message: string, duration = 2000) {
  saveStatus = message;

  setTimeout(() => {
    if (saveStatus === message) {
      saveStatus = "";
    }
  }, duration);
}

onMount(async () => {
  await Promise.all([loadCssThemes()]);
});

async function loadCssThemes() {
  try {
    cssThemes = (await api.listCssThemes()).map((t) => (typeof t === "string" ? t : t.custom));
  } catch (error) {
    console.error("Failed to load CSS themes:", error);
    cssThemes = ["default"];
  }
}

async function loadIconTemplates() {
  try {
    iconTemplates = await app.customizationService.getIconTemplates();
  } catch (error) {
    console.error("Failed to load icon templates:", error);
    iconTemplates = [];
  }
}

async function selectTheme(themeName: string) {
  try {
    if (themeName === "default") {
      await app.customizationService.setTheme("system");
      return;
    }

    await app.customizationService.setTheme({
      custom: themeName,
    });

    app.customizationService.scheduleSave();
    setStatus("CSS theme updated successfully");
  } catch (error) {
    console.error("Failed to update CSS theme:", error);
    setStatus("Failed to update CSS theme");
  }
}

async function handleCssUpload() {
  if (!cssUploadFile) return;

  try {
    cssUploadError = "";

    const content = await cssUploadFile.text();
    const themeName = cssUploadFile.name.replace(/\.css$/i, "");

    await api.saveCssTheme(themeName, content);
    await loadCssThemes();

    showCssUpload = false;
    cssUploadFile = null;

    setStatus(`CSS theme "${themeName}" uploaded successfully`, 3000);
  } catch (error) {
    console.error("Failed to upload CSS theme:", error);
    cssUploadError = `Upload failed: ${error}`;
  }
}

function handleCssDragOver(event: DragEvent) {
  event.preventDefault();
  cssIsDragOver = true;
}

function handleCssDragLeave() {
  cssIsDragOver = false;
}

function handleCssDrop(event: DragEvent) {
  event.preventDefault();
  cssIsDragOver = false;

  const file = event.dataTransfer?.files?.[0];

  if (!file) return;

  if (!file.name.toLowerCase().endsWith(".css")) {
    cssUploadError = "Please select a .css file";
    return;
  }

  cssUploadFile = file;
  cssUploadError = "";
}

function handleCssFileSelect(event: Event) {
  const file = (event.currentTarget as HTMLInputElement).files?.[0];

  if (!file) return;

  if (!file.name.toLowerCase().endsWith(".css")) {
    cssUploadError = "Please select a .css file";
    return;
  }

  cssUploadFile = file;
  cssUploadError = "";
}

async function removeCssTheme(themeName: string) {
  if (builtInThemes.includes(themeName)) {
    setStatus("Cannot remove built-in themes");
    return;
  }

  if (!confirm(`Are you sure you want to remove the "${themeName}" CSS theme?`)) {
    return;
  }

  try {
    await api.deleteCssTheme(themeName);
    await loadCssThemes();

    setStatus("CSS theme removed successfully");
  } catch (error) {
    console.error("Failed to remove CSS theme:", error);
    setStatus("Failed to remove CSS theme");
  }
}

async function openCssThemesDirectory() {
  try {
    await api.openCssThemesDirectory();
  } catch (error) {
    console.error("Failed to open CSS themes directory:", error);
    setStatus("Failed to open CSS themes directory");
  }
}

async function selectIconTemplate(templateId: string) {
  try {
    if (!app.customizationService.settings?.appearance) return;

    app.customizationService.settings.appearance.icon_template = templateId;

    app.customizationService.selectedIconTemplate = templateId;
    app.customizationService.scheduleSave();

    setStatus("Icon template updated successfully");
  } catch (error) {
    console.error("Failed to update icon template:", error);
    setStatus("Failed to update icon template");
  }
}

async function handleIconUpload() {
  if (!uploadFile) return;

  try {
    uploadError = "";

    const content = await uploadFile.text();

    const format = uploadFile.name.endsWith(".yml") || uploadFile.name.endsWith(".yaml") ? "yaml" : "json";

    // Keep validation/install logic in the service if you add it there.
    // For now the service expects an already parsed IconTemplate.
    const template = JSON.parse(content) as IconTemplate;

    if (format === "yaml") {
      throw new Error("YAML icon template importing is not implemented");
    }

    await app.customizationService.saveCustomIconTemplate(template);
    await loadIconTemplates();

    showIconUpload = false;
    uploadFile = null;

    setStatus(`Template "${template.name}" installed successfully`, 3000);
  } catch (error) {
    console.error("Failed to upload icon template:", error);
    uploadError = `Upload failed: ${error}`;
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  isDragOver = true;
}

function handleDragLeave() {
  isDragOver = false;
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver = false;

  const file = event.dataTransfer?.files?.[0];

  if (!file) return;

  const name = file.name.toLowerCase();

  if (!name.endsWith(".json") && !name.endsWith(".yml") && !name.endsWith(".yaml")) {
    uploadError = "Please select a .json, .yml, or .yaml file";
    return;
  }

  uploadFile = file;
  uploadError = "";
}

function handleFileSelect(event: Event) {
  const file = (event.currentTarget as HTMLInputElement).files?.[0];

  if (!file) return;

  uploadFile = file;
  uploadError = "";
}

async function removeCustomTemplate(templateId: string) {
  if (!confirm("Are you sure you want to remove this icon template?")) {
    return;
  }

  try {
    await app.customizationService.deleteCustomIconTemplate(templateId);
    await loadIconTemplates();

    setStatus("Template removed successfully");
  } catch (error) {
    console.error("Failed to remove icon template:", error);
    setStatus("Failed to remove icon template");
  }
}

async function openIconsDirectory() {
  try {
    await app.customizationService.openIconsDirectory();
  } catch (error) {
    console.error("Failed to open icons directory:", error);
    setStatus("Failed to open icons directory");
  }
}

function updateSoundEnabled(enabled: boolean) {
  app.customizationService.setSoundEnabled(enabled);
  app.customizationService.scheduleSave();
}

function updateMusicEnabled(enabled: boolean) {
  app.customizationService.setMusicEnabled(enabled);
  app.customizationService.scheduleSave();
}

function updateMasterVolume(volume: number) {
  const sound = app.customizationService.settings?.appearance?.sound_settings;
  if (!sound) return;

  sound.master_volume = volume;

  app.customizationService.setMasterVolume(volume);
  app.customizationService.scheduleSave();
}

function updateSoundVolume(volume: number) {
  const sound = app.customizationService.settings?.appearance?.sound_settings;
  if (!sound) return;

  sound.sound_volume = volume;

  app.customizationService.setSoundVolume(volume);
  app.customizationService.scheduleSave();
}

function updateMusicVolume(volume: number) {
  const sound = app.customizationService.settings?.appearance?.sound_settings;
  if (!sound) return;

  sound.music_volume = volume;

  app.customizationService.setMusicVolume(volume);
  app.customizationService.scheduleSave();
}

async function selectSoundpack(packName: string) {
  try {
    const sound = app.customizationService.settings?.appearance?.sound_settings;
    if (!sound) return;

    sound.selected_soundpack = packName;

    await app.customizationService.loadSoundpack(packName);
    app.customizationService.scheduleSave();

    setStatus("Soundpack changed successfully");
  } catch (error) {
    console.error("Failed to change soundpack:", error);
    setStatus("Failed to change soundpack");
  }
}
</script>

{#if app.customizationService.settings?.appearance}
  {@const appearance = app.customizationService.settings.appearance}

  <div class="settings-tab">
    <h2>Appearance Settings</h2>
    <p>Customize the look and feel of the launcher.</p>

    <form>
      <!-- Theme -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="theme">Theme</label>
          <p class="setting-description">Choose the launcher appearance.</p>
        </div>

        <div class="setting-control">
          <select
            id="theme"
            value={typeof appearance.theme === "object" ? `custom:${appearance.theme.custom}` : (appearance.theme ?? "system")}
            onchange={(event) => {
              const value = (event.currentTarget as HTMLSelectElement).value;

              if (value === "light" || value === "dark" || value === "system") {
                app.customizationService.setTheme(value);
              } else if (value.startsWith("custom:")) {
                app.customizationService.setTheme({
                  custom: value.substring("custom:".length),
                });
              }

              app.customizationService.scheduleSave();
            }}
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>

            {#if hasCustomThemes}
              <optgroup label="Custom Themes">
                {#each customThemes as theme (theme)}
                  <option value={`custom:${theme}`}>
                    {theme}
                  </option>
                {/each}
              </optgroup>
            {/if}
          </select>
        </div>
      </div>

      <!-- CSS Theme Management -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="css-theme-mangement">CSS Theme Management</label>
          <p class="setting-description">Upload, remove, or open the CSS themes folder.</p>
        </div>

        <div class="setting-control">
          <button type="button" onclick={() => (showCssUpload = !showCssUpload)}>
            {showCssUpload ? "Cancel Upload" : "Upload Custom Theme"}
          </button>

          <button type="button" onclick={openCssThemesDirectory}> Open Themes Directory </button>
        </div>
      </div>

      {#if showCssUpload}
        <div class="setting-item">
          <div class="setting-info">
            <label for="css-file-input">Upload Zone</label>
            <p class="setting-description">Drag & drop or click to select a CSS theme file (.css).</p>
          </div>

          <div class="setting-control">
            <label
              for="css-file-input"
              class:drag-over={cssIsDragOver}
              class:error={cssUploadError}
              class="upload-zone"
              ondragover={handleCssDragOver}
              ondragleave={handleCssDragLeave}
              ondrop={handleCssDrop}
            >
              <input type="file" id="css-file-input" accept=".css" onchange={handleCssFileSelect} hidden />

              <div class="upload-placeholder">
                <h4>Drag & drop or click to select a CSS file</h4>
                <p>Accepted: .css</p>

                {#if cssUploadFile}
                  <div class="file-info">
                    <span class="file-name">{cssUploadFile.name}</span>

                    <div class="file-actions">
                      <button type="button" onclick={handleCssUpload}> Upload </button>

                      <button type="button" onclick={() => (cssUploadFile = null)}> Remove </button>
                    </div>
                  </div>
                {/if}

                {#if cssUploadError}
                  <div class="error-message">{cssUploadError}</div>
                {/if}
              </div>
            </label>
          </div>
        </div>
      {/if}

      {#if hasCustomThemes}
        <div class="setting-item">
          <div class="setting-info">
            <label for="custom-css-themes">Custom CSS Themes</label>
            <p class="setting-description">Manage custom CSS themes. Built-in themes cannot be removed.</p>
          </div>

          <div class="setting-control custom-templates-list">
            <button type="button" class="dropdown-toggle" onclick={() => (showCustomThemes = !showCustomThemes)}>
              {showCustomThemes ? "Hide Custom Themes" : "Manage Custom Themes"}
            </button>

            {#if showCustomThemes}
              <div class="custom-templates-rows">
                {#each customThemes as theme (theme)}
                  <div class="custom-template-row">
                    <span class="template-name">{theme}</span>

                    <button type="button" class="icon-btn btn-danger" title="Remove Custom Theme" onclick={() => removeCssTheme(theme)}>
                      <Icon name="delete" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Language -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="language">Language</label>
          <p class="setting-description">Currently available language.</p>
        </div>

        <div class="setting-control">
          <select id="language" bind:value={appearance.language} onchange={scheduleSave}>
            <option value="english">English</option>
          </select>
        </div>
      </div>

      <!-- Icon Template -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="selected-icon-template">Icon Template</label>
          <p class="setting-description">Choose the icon template used by the launcher.</p>
        </div>

        <div class="setting-control">
          <select
            id="selected-icon-template"
            value={appearance.icon_template ?? ""}
            onchange={(event) => selectIconTemplate((event.currentTarget as HTMLSelectElement).value)}
          >
            {#each iconTemplates as template (template.id)}
              <option value={template.id}>
                {template.name}
              </option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Icon Template Management -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="icon-template-management">Icon Template Management</label>
          <p class="setting-description">Upload, remove, or open the icon templates folder.</p>
        </div>

        <div class="setting-control template-management">
          <button type="button" onclick={() => (showIconUpload = !showIconUpload)}>
            {showIconUpload ? "Cancel Upload" : "Upload Custom Template"}
          </button>

          <button type="button" onclick={openIconsDirectory}> Open Icons Directory </button>
        </div>
      </div>

      {#if showIconUpload}
        <div class="setting-item">
          <div class="setting-info">
            <h4>Upload Zone</h4>
            <p class="setting-description">Drag & drop or click to select a template (.json, .yml, .yaml).</p>
          </div>

          <div class="setting-control">
            <label
              for="template-file-input"
              class:drag-over={isDragOver}
              class:error={uploadError}
              class="upload-zone"
              ondragover={handleDragOver}
              ondragleave={handleDragLeave}
              ondrop={handleDrop}
            >
              <input type="file" id="template-file-input" accept=".json,.yml,.yaml" onchange={handleFileSelect} hidden />

              <div class="upload-placeholder">
                <h4>Drag & drop or click to select a template</h4>
                <p>Accepted: .json, .yml, .yaml</p>

                {#if uploadFile}
                  <div class="file-info">
                    <span class="file-name">{uploadFile.name}</span>

                    <div class="file-actions">
                      <button type="button" onclick={handleIconUpload}> Upload </button>

                      <button type="button" onclick={() => (uploadFile = null)}> Remove </button>
                    </div>
                  </div>
                {/if}

                {#if uploadError}
                  <div class="error-message">{uploadError}</div>
                {/if}
              </div>
            </label>
          </div>
        </div>
      {/if}

      <!-- Custom Icon Templates -->

      {#if customIconTemplates.length > 0}
        <div class="setting-item">
          <div class="setting-info">
            <label for="custom-icon-templates">Custom Icon Templates</label>
            <p class="setting-description">Manage your custom icon templates.</p>
          </div>

          <div class="setting-control custom-templates-list">
            <button type="button" class="dropdown-toggle" onclick={() => (showCustomTemplates = !showCustomTemplates)}>
              {showCustomTemplates ? "Hide Custom Templates" : "Manage Custom Templates"}
            </button>

            {#if showCustomTemplates}
              <div class="custom-templates-rows">
                {#each customIconTemplates as template (template.id)}
                  <div class="custom-template-row">
                    <span class="template-name">
                      {template.name}
                    </span>

                    <button type="button" class="icon-btn btn-danger" title="Remove" onclick={() => removeCustomTemplate(template.id)}>
                      <Icon name="delete" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Sound -->

      {#if appearance.sound_settings}
        {@const sound = appearance.sound_settings}

        <div class="setting-item">
          <div class="setting-info">
            <label for="sound-enabled">Sound Effects</label>
            <p class="setting-description">Enable UI sound effects.</p>
          </div>

          <div class="setting-control">
            <input
              id="sound-enabled"
              type="checkbox"
              bind:checked={sound.enabled}
              onchange={(event) => updateSoundEnabled((event.currentTarget as HTMLInputElement).checked)}
            />
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label for="music-enabled">Background Music</label>
            <p class="setting-description">Enable background music.</p>
          </div>

          <div class="setting-control">
            <input
              id="music-enabled"
              type="checkbox"
              bind:checked={sound.music_enabled}
              onchange={(event) => updateMusicEnabled((event.currentTarget as HTMLInputElement).checked)}
            />
          </div>
        </div>

        {#if sound.enabled || sound.music_enabled}
          <div class="setting-item">
            <div class="setting-info">
              <label for="master-volume"> Master Volume </label>
              <p class="setting-description">
                Overall volume: {sound.master_volume ?? 100}%
              </p>
            </div>

            <div class="setting-control slider-control">
              <input
                id="master-volume"
                type="range"
                min="0"
                max="100"
                step="5"
                value={sound.master_volume ?? 100}
                oninput={(event) => updateMasterVolume(Number((event.currentTarget as HTMLInputElement).value))}
              />
            </div>
          </div>

          {#if sound.enabled}
            <div class="setting-item">
              <div class="setting-info">
                <label for="sound-volume"> Sound Effects Volume </label>
                <p class="setting-description">
                  Volume for UI sounds: {sound.sound_volume ?? 100}%
                </p>
              </div>

              <div class="setting-control slider-control">
                <input
                  id="sound-volume"
                  type="range"
                  min="0"
                  max="100"
                  step="5"
                  value={sound.sound_volume ?? 100}
                  oninput={(event) => updateSoundVolume(Number((event.currentTarget as HTMLInputElement).value))}
                />
              </div>
            </div>
          {/if}

          {#if sound.music_enabled}
            <div class="setting-item">
              <div class="setting-info">
                <label for="music-volume"> Music Volume </label>
                <p class="setting-description">
                  Volume for background music: {sound.music_volume ?? 50}%
                </p>
              </div>

              <div class="setting-control slider-control">
                <input
                  id="music-volume"
                  type="range"
                  min="0"
                  max="100"
                  step="5"
                  value={sound.music_volume ?? 50}
                  oninput={(event) => updateMusicVolume(Number((event.currentTarget as HTMLInputElement).value))}
                />
              </div>
            </div>
          {/if}

          <div class="setting-item">
            <div class="setting-info">
              <label for="soundpack">Soundpack</label>
              <p class="setting-description">Select the soundpack used by the launcher.</p>
            </div>

            <div class="setting-control">
              <select
                id="soundpack"
                value={sound.selected_soundpack ?? "default"}
                onchange={(event) => selectSoundpack((event.currentTarget as HTMLSelectElement).value)}
              >
                {#each app.customizationService.availableSoundpacks as pack (pack.name)}
                  <option value={pack.name}>
                    {pack.displayName}
                  </option>
                {/each}
              </select>
            </div>
          </div>
        {/if}
      {/if}
    </form>

    {#if saveStatus}
      <div class="warning-card" class:success={saveStatus.includes("successfully")} class:error={saveStatus.includes("Failed")}>
        {saveStatus}
      </div>
    {/if}
  </div>
{/if}

<style lang="scss">
.settings-tab {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.75rem;

  padding: 2rem 2.5rem;
  margin-bottom: 2rem;

  background: $color-surface-1;
  border: 1px solid $color-border-muted;
  border-radius: $radius-2xl;
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
}

.settings-tab h2 {
  margin: 0;

  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0.02em;

  color: $color-text;

  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.settings-tab > p {
  margin: -1rem 0 0;

  color: $color-text-muted;
  font-size: 0.95rem;
  line-height: 1.5;
}

form {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 2.5rem;

  min-width: 0;
  padding: 1.25rem 0;

  border-bottom: 1px solid $color-border-muted;

  &:last-child {
    border-bottom: none;
  }
}

.setting-info {
  flex: 1 1 16rem;
  min-width: 13rem;

  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.setting-info label,
.setting-info > h4 {
  margin: 0;

  color: $color-text;
  font-size: 1rem;
  font-weight: 500;
  line-height: 1.4;
}

.setting-description {
  margin: 0;

  color: $color-text-muted;
  font-size: 0.875rem;
  line-height: 1.45;
}

.setting-control {
  flex: 1 1 18rem;
  min-width: 12rem;

  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.75rem;
}

.template-management {
  gap: 0.75rem;
}

/* Inputs */

select,
input[type="text"] {
  width: 100%;
  max-width: 20rem;

  padding: 0.55rem 0.8rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.9rem;

  outline: none;

  transition:
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    border-color: $color-accent-muted;
    background: $color-surface-3;
  }

  &:focus {
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

select {
  cursor: pointer;
}

/* Checkboxes */

input[type="checkbox"] {
  width: 1.1rem;
  height: 1.1rem;

  margin: 0;

  accent-color: $color-accent;
  cursor: pointer;
}

/* Range sliders */

.slider-control {
  gap: 0.75rem;

  /* Range sliders */

  input[type="range"] {
    --range-progress: 50%;

    flex: 1 1 auto;
    width: 100%;
    min-width: 8rem;
    max-width: none;
    height: 0.35rem;
    margin: 0;

    appearance: none;
    -webkit-appearance: none;

    border-radius: 999px;

    background: linear-gradient(to right, $color-accent 0%, $color-accent var(--range-progress), $color-surface-3 var(--range-progress), $color-surface-3 100%);

    cursor: pointer;
    outline: none;
  }

  /* Chromium / WebKit */

  input[type="range"]::-webkit-slider-runnable-track {
    height: 0.35rem;
    border-radius: 999px;
    background: transparent;
  }

  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;

    width: 1rem;
    height: 1rem;

    margin-top: -0.325rem;

    border: 2px solid $color-surface-1;
    border-radius: 50%;

    background: $color-accent;
    box-shadow: 0 0 0 1px $color-accent;

    transition:
      transform 0.15s ease,
      box-shadow 0.2s ease;
  }

  input[type="range"]:hover::-webkit-slider-thumb {
    transform: scale(1.15);

    box-shadow:
      0 0 0 1px $color-accent,
      0 0 0 4px color-mix(in srgb, $color-accent 15%, transparent);
  }

  input[type="range"]:focus-visible::-webkit-slider-thumb {
    box-shadow:
      0 0 0 1px $color-focus,
      0 0 0 4px color-mix(in srgb, $color-focus 20%, transparent);
  }

  /* Firefox */

  input[type="range"]::-moz-range-track {
    height: 0.35rem;
    border-radius: 999px;
    background: $color-surface-3;
  }

  input[type="range"]::-moz-range-progress {
    height: 0.35rem;
    border-radius: 999px;
    background: $color-accent;
  }

  input[type="range"]::-moz-range-thumb {
    width: 1rem;
    height: 1rem;

    border: 2px solid $color-surface-1;
    border-radius: 50%;

    background: $color-accent;
    box-shadow: 0 0 0 1px $color-accent;

    transition:
      transform 0.15s ease,
      box-shadow 0.2s ease;
  }

  input[type="range"]:hover::-moz-range-thumb {
    transform: scale(1.15);

    box-shadow:
      0 0 0 1px $color-accent,
      0 0 0 4px color-mix(in srgb, $color-accent 15%, transparent);
  }

  input[type="range"]:focus-visible::-moz-range-thumb {
    box-shadow:
      0 0 0 1px $color-focus,
      0 0 0 4px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

/* Buttons */

button {
  padding: 0.55rem 0.9rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.875rem;
  font-weight: 500;

  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    transform 0.15s ease;

  &:hover {
    background: $color-surface-3;
    border-color: $color-accent;
    color: $color-accent;
  }

  &:active {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

/* Upload */

.upload-zone {
  width: 100%;
  min-height: 9rem;

  display: flex;
  align-items: center;
  justify-content: center;

  padding: 1.5rem;

  border: 1px dashed $color-border;
  border-radius: $radius-xl;

  background: $color-surface-2;

  cursor: pointer;

  transition:
    border-color 0.2s ease,
    background 0.2s ease;

  &:hover,
  &.drag-over {
    border-color: $color-accent;
    background: color-mix(in srgb, $color-accent 5%, $color-surface-2);
  }

  &.error {
    border-color: $color-error;
    background: color-mix(in srgb, $color-error 5%, $color-surface-2);
  }
}

.upload-placeholder {
  width: 100%;

  display: flex;
  flex-direction: column;
  align-items: center;

  text-align: center;

  h4 {
    margin: 0 0 0.35rem;

    color: $color-text;
    font-size: 0.95rem;
    font-weight: 500;
  }

  > p {
    margin: 0 0 1rem;

    color: $color-text-muted;
    font-size: 0.8rem;
  }
}

.file-info {
  width: 100%;

  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;

  padding: 0.75rem 1rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-3;

  .file-name {
    min-width: 0;

    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;

    color: $color-text;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .file-actions {
    display: flex;
    flex-shrink: 0;
    gap: 0.5rem;
  }
}

.error-message {
  margin-top: 0.75rem;

  color: $color-error;
  font-size: 0.8rem;
}

/* Custom templates */

.custom-templates-list {
  width: 100%;

  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.75rem;
}

.dropdown-toggle {
  width: auto;
  max-width: 20rem;
}

.custom-templates-rows {
  width: 100%;

  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.custom-template-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;

  width: 100%;
  padding: 0.65rem 0.75rem 0.65rem 1rem;

  border: 1px solid $color-border-muted;
  border-radius: $radius-lg;

  background: $color-surface-3;

  transition:
    border-color 0.2s ease,
    background 0.2s ease;

  &:hover {
    border-color: $color-border;
    background: $color-surface-2;
  }

  .template-name {
    flex: 1;
    min-width: 0;

    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;

    color: $color-text;
    font-size: 0.875rem;
    font-weight: 500;
  }
}

.custom-template-row .icon-btn {
  width: 2rem;
  height: 2rem;

  flex-shrink: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  padding: 0;

  border-color: color-mix(in srgb, $color-error 40%, $color-border);

  background: color-mix(in srgb, $color-error 12%, $color-surface-2);

  color: $color-error;

  &:hover {
    border-color: $color-error;

    background: color-mix(in srgb, $color-error 20%, $color-surface-2);

    color: $color-error;
  }
}

/* Status */

.warning-card {
  padding: 0.75rem 1rem;

  border: 1px solid $color-warning;
  border-radius: $radius-md;

  background: color-mix(in srgb, $color-warning 10%, $color-surface-2);

  color: $color-warning;
  font-size: 0.875rem;

  &.success {
    border-color: $color-success;

    background: color-mix(in srgb, $color-success 10%, $color-surface-2);

    color: $color-success;
  }

  &.error {
    border-color: $color-error;

    background: color-mix(in srgb, $color-error 10%, $color-surface-2);

    color: $color-error;
  }
}

/* Responsive */

@media (max-width: 700px) {
  .settings-tab {
    padding: 1.5rem;
  }

  .setting-item {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }

  .setting-info {
    min-width: 0;
  }

  .setting-control {
    width: 100%;
    min-width: 0;

    justify-content: flex-start;
  }

  select,
  input[type="text"] {
    max-width: none;
  }

  .custom-templates-list {
    align-items: stretch;
  }

  .dropdown-toggle {
    max-width: none;
  }
}
</style>
