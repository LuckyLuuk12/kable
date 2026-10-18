<script lang="ts">
import { Icon, api, app, type IconTemplate } from "$lib";
import { onMount } from "svelte";

interface CssThemeEntry {
  id: string;
  name: string;
  type: "builtin" | "custom" | string;
}

let showCustomTemplates = $state(false);
let showIconUpload = $state(false);
let uploadError = $state("");
let uploadFile = $state<File | null>(null);
let isDragOver = $state(false);

let showCssUpload = $state(false);
let cssUploadError = $state("");
let cssUploadFile = $state<File | null>(null);
let cssIsDragOver = $state(false);

const cssThemes = $derived(app.customizationService.cssThemes);
const cssThemesLoading = $derived(app.customizationService.isCssThemesLoading);
const selectedCssThemeId = $derived(app.customizationService.selectedCssTheme);
let showCustomThemes = $state(false);

let iconTemplates = $state<IconTemplate[]>([]);
let saveStatus = $state("");
let uploadCssInProgress = $state(false);
let uploadIconInProgress = $state(false);

let statusTimeout: ReturnType<typeof setTimeout> | undefined;

const builtInThemeIds = new Set(["builtin:kasai-sora", "builtin:modrinth", "builtin:official"]);

const customThemes = $derived(cssThemes.filter((theme) => theme.type === "custom" || theme.id.startsWith("custom:")));
const builtInThemes = $derived(cssThemes.filter((theme) => theme.type === "builtin" || builtInThemeIds.has(theme.id)));
const hasCustomThemes = $derived(customThemes.length > 0);
const customIconTemplates = $derived(iconTemplates.filter((template) => template.type === "custom"));

const customization = $derived(app.customizationService);
const appearance = $derived(customization.settings?.appearance ?? null);

function scheduleSave() {
  customization.scheduleSave();
}

function setStatus(message: string, duration = 2500) {
  saveStatus = message;

  if (statusTimeout) {
    clearTimeout(statusTimeout);
  }

  statusTimeout = setTimeout(() => {
    if (saveStatus === message) {
      saveStatus = "";
    }
  }, duration);
}

onMount(() => {
  void Promise.allSettled([app.customizationService.loadCssThemes(), loadIconTemplates()]);

  return () => {
    if (statusTimeout) {
      clearTimeout(statusTimeout);
    }
  };
});

async function loadIconTemplates() {
  try {
    iconTemplates = await customization.getIconTemplates();
  } catch (error) {
    console.error("Failed to load icon templates:", error);
    setStatus("Failed to load icon templates");
  }
}

async function selectTheme(themeId: string) {
  const nextThemeId = themeId || null;

  if (nextThemeId && !cssThemes.some((theme) => theme.id === nextThemeId)) {
    setStatus("The selected CSS theme is unavailable");
    return;
  }

  try {
    await customization.setTheme(nextThemeId);

    setStatus(nextThemeId ? "CSS theme updated successfully" : "Default appearance restored");
  } catch (error) {
    console.error("Failed to update CSS theme:", error);
    setStatus("Failed to update CSS theme");
  }
}

async function handleCssUpload() {
  if (!cssUploadFile || uploadCssInProgress) return;

  uploadCssInProgress = true;
  cssUploadError = "";

  try {
    const content = await cssUploadFile.text();
    const themeName = cssUploadFile.name.replace(/\.css$/i, "").trim();

    if (!themeName) {
      throw new Error("The CSS filename must contain a theme name");
    }

    await customization.saveCssTheme(themeName, content);

    showCssUpload = false;
    cssUploadFile = null;

    setStatus(`CSS theme "${themeName}" installed successfully`, 3000);
  } catch (error) {
    console.error("Failed to upload CSS theme:", error);
    cssUploadError = `Upload failed: ${String(error)}`;
  } finally {
    uploadCssInProgress = false;
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
  if (file) setCssUploadFile(file);
}

function handleCssFileSelect(event: Event) {
  const file = (event.currentTarget as HTMLInputElement).files?.[0];
  if (file) setCssUploadFile(file);
}

function setCssUploadFile(file: File) {
  if (!file.name.toLowerCase().endsWith(".css")) {
    cssUploadError = "Please select a .css file";
    cssUploadFile = null;
    return;
  }

  cssUploadFile = file;
  cssUploadError = "";
}

async function removeCssTheme(themeId: string) {
  const theme = cssThemes.find((entry) => entry.id === themeId);

  if (!theme || (theme.type !== "custom" && !theme.id.startsWith("custom:"))) {
    setStatus("Built-in themes cannot be removed");
    return;
  }

  if (!confirm(`Are you sure you want to remove the "${theme.name}" CSS theme?`)) {
    return;
  }

  try {
    await customization.deleteCssTheme(themeId);
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

async function selectIconTemplate(templateId: string | null) {
  if (!appearance) return;

  if (templateId && !iconTemplates.some((template) => template.id === templateId)) {
    setStatus("The selected icon template is unavailable");
    return;
  }

  try {
    appearance.icon_template = templateId;
    customization.selectedIconTemplate = templateId;
    scheduleSave();

    setStatus(templateId ? "Icon template updated successfully" : "Default icon template restored");
  } catch (error) {
    console.error("Failed to update icon template:", error);
    setStatus("Failed to update icon template");
  }
}

async function handleIconUpload() {
  if (!uploadFile || uploadIconInProgress) return;

  uploadIconInProgress = true;
  uploadError = "";

  try {
    const content = await uploadFile.text();
    const extension = uploadFile.name.split(".").pop()?.toLowerCase();

    if (extension === "yml" || extension === "yaml") {
      throw new Error("YAML icon template importing is not implemented. Use a JSON template.");
    }

    if (extension !== "json") {
      throw new Error("Please select a .json file");
    }

    const template = JSON.parse(content) as IconTemplate;

    if (!template || typeof template !== "object" || !template.name || !template.id || !template.icons) {
      throw new Error("Invalid icon template: expected an object containing id, name, and icons");
    }

    await customization.saveCustomIconTemplate(template);
    await loadIconTemplates();

    showIconUpload = false;
    uploadFile = null;

    setStatus(`Template "${template.name}" installed successfully`, 3000);
  } catch (error) {
    console.error("Failed to upload icon template:", error);
    uploadError = `Upload failed: ${String(error)}`;
  } finally {
    uploadIconInProgress = false;
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
  if (file) setIconUploadFile(file);
}

function handleFileSelect(event: Event) {
  const file = (event.currentTarget as HTMLInputElement).files?.[0];
  if (file) setIconUploadFile(file);
}

function setIconUploadFile(file: File) {
  const name = file.name.toLowerCase();

  if (!name.endsWith(".json") && !name.endsWith(".yml") && !name.endsWith(".yaml")) {
    uploadError = "Please select a .json, .yml, or .yaml file";
    uploadFile = null;
    return;
  }

  uploadFile = file;
  uploadError = "";
}

async function removeCustomTemplate(templateId: string) {
  if (!confirm("Are you sure you want to remove this icon template?")) {
    return;
  }

  try {
    await customization.deleteCustomIconTemplate(templateId);
    await loadIconTemplates();
    setStatus("Template removed successfully");
  } catch (error) {
    console.error("Failed to remove icon template:", error);
    setStatus("Failed to remove icon template");
  }
}

async function openIconsDirectory() {
  try {
    await customization.openIconsDirectory();
  } catch (error) {
    console.error("Failed to open icons directory:", error);
    setStatus("Failed to open icons directory");
  }
}

function updateSoundEnabled(enabled: boolean) {
  customization.setSoundEnabled(enabled);
  scheduleSave();
}

function updateMusicEnabled(enabled: boolean) {
  customization.setMusicEnabled(enabled);
  scheduleSave();
}

function updateMasterVolume(volume: number) {
  const sound = appearance?.sound_settings;
  if (!sound) return;

  sound.master_volume = volume;
  customization.setMasterVolume(volume);
  scheduleSave();
}

function updateSoundVolume(volume: number) {
  const sound = appearance?.sound_settings;
  if (!sound) return;

  sound.sound_volume = volume;
  customization.setSoundVolume(volume);
  scheduleSave();
}

function updateMusicVolume(volume: number) {
  const sound = appearance?.sound_settings;
  if (!sound) return;

  sound.music_volume = volume;
  customization.setMusicVolume(volume);
  scheduleSave();
}

async function selectSoundpack(packName: string) {
  const sound = appearance?.sound_settings;
  if (!sound) return;

  try {
    await customization.loadSoundpack(packName);
    sound.selected_soundpack = packName;
    scheduleSave();

    setStatus("Soundpack changed successfully");
  } catch (error) {
    console.error("Failed to change soundpack:", error);
    setStatus("Failed to change soundpack");
  }
}
</script>

{#if appearance}
  <div class="settings-tab">
    <header class="settings-header">
      <h2>Appearance Settings</h2>
      <p>Customize the look and feel of the launcher.</p>
    </header>

    <form onsubmit={(event) => event.preventDefault()}>
      <!-- CSS Theme Selection -->

      <div class="setting-item">
        <div class="setting-info">
          <label for="css-theme">CSS Theme</label>
          <p class="setting-description">Select a built-in or custom stylesheet. Choose Default to use the launcher's standard appearance.</p>
        </div>

        <div class="setting-control">
          <select
            id="css-theme"
            value={selectedCssThemeId ?? ""}
            disabled={cssThemesLoading}
            onchange={(event) => void selectTheme((event.currentTarget as HTMLSelectElement).value)}>
            <option value="">Default</option>

            {#if builtInThemes.length > 0}
              <optgroup label="Built-in Themes">
                {#each builtInThemes as theme (theme.id)}
                  <option value={theme.id}>{theme.name}</option>
                {/each}
              </optgroup>
            {/if}

            {#if hasCustomThemes}
              <optgroup label="Custom Themes">
                {#each customThemes as theme (theme.id)}
                  <option value={theme.id}>{theme.name}</option>
                {/each}
              </optgroup>
            {/if}
          </select>
        </div>
      </div>

      <!-- CSS Theme Management -->

      <div class="setting-item">
        <div class="setting-info">
          <label>CSS Theme Management</label>
          <p class="setting-description">Install custom CSS files, manage installed themes, or open the themes directory.</p>
        </div>

        <div class="setting-control">
          <button type="button" onclick={() => (showCssUpload = !showCssUpload)}>
            {showCssUpload ? "Cancel Upload" : "Upload Custom Theme"}
          </button>

          <button type="button" onclick={() => void openCssThemesDirectory()}> Open Themes Directory </button>
        </div>
      </div>

      {#if showCssUpload}
        <div class="setting-item">
          <div class="setting-info">
            <label for="css-file-input">Upload CSS Theme</label>
            <p class="setting-description">Drag and drop a .css file or select one from your computer.</p>
          </div>

          <div class="setting-control">
            <label
              for="css-file-input"
              class="upload-zone"
              class:drag-over={cssIsDragOver}
              class:error={!!cssUploadError}
              ondragover={handleCssDragOver}
              ondragleave={handleCssDragLeave}
              ondrop={handleCssDrop}>
              <input type="file" id="css-file-input" accept=".css,text/css" onchange={handleCssFileSelect} hidden />

              <div class="upload-placeholder">
                <h4>Drag and drop or click to select a CSS file</h4>
                <p>Accepted format: .css</p>

                {#if cssUploadFile}
                  <div class="file-info">
                    <span class="file-name">{cssUploadFile.name}</span>

                    <div class="file-actions">
                      <button
                        type="button"
                        disabled={uploadCssInProgress}
                        onclick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          void handleCssUpload();
                        }}>
                        {uploadCssInProgress ? "Uploading…" : "Upload"}
                      </button>

                      <button
                        type="button"
                        disabled={uploadCssInProgress}
                        onclick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          cssUploadFile = null;
                          cssUploadError = "";
                        }}>
                        Remove
                      </button>
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
            <label>Custom CSS Themes</label>
            <p class="setting-description">Remove custom themes you no longer need. Built-in themes cannot be deleted.</p>
          </div>

          <div class="setting-control custom-templates-list">
            <button type="button" class="dropdown-toggle" onclick={() => (showCustomThemes = !showCustomThemes)}>
              {showCustomThemes ? "Hide Custom Themes" : "Manage Custom Themes"}
            </button>

            {#if showCustomThemes}
              <div class="custom-templates-rows">
                {#each customThemes as theme (theme.id)}
                  <div class="custom-template-row">
                    <span class="template-name">{theme.name}</span>

                    <button
                      type="button"
                      class="icon-btn btn-danger"
                      title={`Remove ${theme.name}`}
                      aria-label={`Remove ${theme.name}`}
                      onclick={() => void removeCssTheme(theme.id)}>
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
          <p class="setting-description">Choose the launcher interface language.</p>
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
          <p class="setting-description">Choose an icon template or use the launcher's default icons.</p>
        </div>

        <div class="setting-control">
          <select
            id="selected-icon-template"
            value={appearance.icon_template ?? ""}
            onchange={(event) => {
              const value = (event.currentTarget as HTMLSelectElement).value;
              void selectIconTemplate(value || null);
            }}>
            <option value="">Default</option>

            {#each iconTemplates as template (template.id)}
              <option value={template.id}>{template.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Icon Template Management -->

      <div class="setting-item">
        <div class="setting-info">
          <label>Icon Template Management</label>
          <p class="setting-description">Install custom icon templates, remove them, or open the templates directory.</p>
        </div>

        <div class="setting-control template-management">
          <button type="button" onclick={() => (showIconUpload = !showIconUpload)}>
            {showIconUpload ? "Cancel Upload" : "Upload Custom Template"}
          </button>

          <button type="button" onclick={() => void openIconsDirectory()}> Open Icons Directory </button>
        </div>
      </div>

      {#if showIconUpload}
        <div class="setting-item">
          <div class="setting-info">
            <h4>Upload Icon Template</h4>
            <p class="setting-description">Select a .json template. YAML importing is not currently supported.</p>
          </div>

          <div class="setting-control">
            <label
              for="template-file-input"
              class="upload-zone"
              class:drag-over={isDragOver}
              class:error={!!uploadError}
              ondragover={handleDragOver}
              ondragleave={handleDragLeave}
              ondrop={handleDrop}>
              <input type="file" id="template-file-input" accept=".json,.yml,.yaml,application/json" onchange={handleFileSelect} hidden />

              <div class="upload-placeholder">
                <h4>Drag and drop or click to select a template</h4>
                <p>Accepted formats: .json, .yml, .yaml (JSON only can currently be imported)</p>

                {#if uploadFile}
                  <div class="file-info">
                    <span class="file-name">{uploadFile.name}</span>

                    <div class="file-actions">
                      <button
                        type="button"
                        disabled={uploadIconInProgress}
                        onclick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          void handleIconUpload();
                        }}>
                        {uploadIconInProgress ? "Uploading…" : "Upload"}
                      </button>

                      <button
                        type="button"
                        disabled={uploadIconInProgress}
                        onclick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          uploadFile = null;
                          uploadError = "";
                        }}>
                        Remove
                      </button>
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
            <label>Custom Icon Templates</label>
            <p class="setting-description">Manage installed custom icon templates.</p>
          </div>

          <div class="setting-control custom-templates-list">
            <button type="button" class="dropdown-toggle" onclick={() => (showCustomTemplates = !showCustomTemplates)}>
              {showCustomTemplates ? "Hide Custom Templates" : "Manage Custom Templates"}
            </button>

            {#if showCustomTemplates}
              <div class="custom-templates-rows">
                {#each customIconTemplates as template (template.id)}
                  <div class="custom-template-row">
                    <span class="template-name">{template.name}</span>

                    <button
                      type="button"
                      class="icon-btn btn-danger"
                      title={`Remove ${template.name}`}
                      aria-label={`Remove ${template.name}`}
                      onclick={() => void removeCustomTemplate(template.id)}>
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
            <p class="setting-description">Enable or disable interface sound effects.</p>
          </div>

          <div class="setting-control">
            <input
              id="sound-enabled"
              type="checkbox"
              bind:checked={sound.enabled}
              onchange={(event) => updateSoundEnabled((event.currentTarget as HTMLInputElement).checked)} />
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <label for="music-enabled">Background Music</label>
            <p class="setting-description">Enable or disable background music.</p>
          </div>

          <div class="setting-control">
            <input
              id="music-enabled"
              type="checkbox"
              bind:checked={sound.music_enabled}
              onchange={(event) => updateMusicEnabled((event.currentTarget as HTMLInputElement).checked)} />
          </div>
        </div>

        {#if sound.enabled || sound.music_enabled}
          <div class="setting-item">
            <div class="setting-info">
              <label for="master-volume">Master Volume</label>
              <p class="setting-description">Overall sound level: {sound.master_volume ?? 100}%</p>
            </div>

            <div class="setting-control slider-control">
              <input
                id="master-volume"
                type="range"
                min="0"
                max="100"
                step="5"
                value={sound.master_volume ?? 100}
                oninput={(event) => updateMasterVolume(Number((event.currentTarget as HTMLInputElement).value))} />
            </div>
          </div>

          {#if sound.enabled}
            <div class="setting-item">
              <div class="setting-info">
                <label for="sound-volume">Sound Effects Volume</label>
                <p class="setting-description">Interface sound level: {sound.sound_volume ?? 100}%</p>
              </div>

              <div class="setting-control slider-control">
                <input
                  id="sound-volume"
                  type="range"
                  min="0"
                  max="100"
                  step="5"
                  value={sound.sound_volume ?? 100}
                  oninput={(event) => updateSoundVolume(Number((event.currentTarget as HTMLInputElement).value))} />
              </div>
            </div>
          {/if}

          {#if sound.music_enabled}
            <div class="setting-item">
              <div class="setting-info">
                <label for="music-volume">Music Volume</label>
                <p class="setting-description">Background music level: {sound.music_volume ?? 50}%</p>
              </div>

              <div class="setting-control slider-control">
                <input
                  id="music-volume"
                  type="range"
                  min="0"
                  max="100"
                  step="5"
                  value={sound.music_volume ?? 50}
                  oninput={(event) => updateMusicVolume(Number((event.currentTarget as HTMLInputElement).value))} />
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
                onchange={(event) => void selectSoundpack((event.currentTarget as HTMLSelectElement).value)}>
                {#each customization.availableSoundpacks as pack (pack.name)}
                  <option value={pack.name}>{pack.displayName}</option>
                {/each}
              </select>
            </div>
          </div>
        {/if}
      {/if}
    </form>

    {#if saveStatus}
      <div
        class="warning-card"
        class:success={saveStatus.toLowerCase().includes("successfully") || saveStatus.toLowerCase().includes("restored")}
        class:error={saveStatus.toLowerCase().includes("failed")}
        role="status"
        aria-live="polite">
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

.settings-header {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.settings-header h2 {
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

.settings-header p {
  margin: 0;
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
  flex-wrap: wrap;
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

  &:hover:not(:disabled) {
    border-color: $color-accent-muted;
    background: $color-surface-3;
  }

  &:focus {
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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

  input[type="range"] {
    flex: 1 1 auto;
    width: 100%;
    min-width: 8rem;
    max-width: none;
    accent-color: $color-accent;
    cursor: pointer;
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
    transform 0.15s ease,
    opacity 0.2s ease;

  &:hover:not(:disabled) {
    background: $color-surface-3;
    border-color: $color-accent;
    color: $color-accent;
  }

  &:active:not(:disabled) {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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
  flex-wrap: wrap;
  gap: 1rem;
  padding: 0.75rem 1rem;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-3;

  .file-name {
    flex: 1 1 8rem;
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
    flex-wrap: wrap;
    gap: 0.5rem;
  }
}

.error-message {
  margin-top: 0.75rem;
  color: $color-error;
  font-size: 0.8rem;
  overflow-wrap: anywhere;
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

  .icon-btn {
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

    &:hover:not(:disabled) {
      border-color: $color-error;
      background: color-mix(in srgb, $color-error 20%, $color-surface-2);
      color: $color-error;
    }
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
  overflow-wrap: anywhere;

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
