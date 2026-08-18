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
  await Promise.all([loadCssThemes(), loadIconTemplates()]);
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
  background: var(--container);
  border-radius: var(--border-radius-large);
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
  padding: 2rem 2.5rem;
  margin-bottom: 2rem;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.settings-tab h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
  color: var(--text-transparent);
  background-clip: text;
  -webkit-background-clip: text;
  -moz-background-clip: text;
  letter-spacing: 0.02em;
}
form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.setting-item {
  display: flex;
  align-items: flex-start;
  gap: 2rem;
  padding: 1rem 0;
  border-bottom: 1px solid var(--dark-200);
}
.setting-item:last-child {
  border-bottom: none;
}
.setting-info {
  flex: 1 1 16.25rem;
  min-width: 13.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.setting-info label {
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--text);
  margin-bottom: 0.1rem;
}
.setting-description {
  font-size: 0.95rem;
  color: var(--placeholder);
  margin-bottom: 0.2rem;
  line-height: 1.4;
}
.setting-control {
  flex: 1 1 11.25rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  min-width: 10rem;
}
.slider-control {
  gap: 0.7rem;
}

input[type="text"] {
  width: 100%;
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);
  color: var(--text);
}
select {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);
  color: var(--text);
}
.template-management {
  display: flex;
  gap: 1rem;
}
.upload-zone {
  border: 2px dashed $color-accent;
  border-radius: var(--border-radius-large);
  padding: 1.5rem;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s;
  &.drag-over {
    border-color: $color-accent-secondary;
  }
  &.error {
    border-color: var(--red);
  }
}
.upload-placeholder {
  h4 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: $color-accent;
  }
  p {
    margin: 0 0 1rem 0;
    color: var(--placeholder);
  }
  .file-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
    .file-name {
      font-weight: 500;
      color: var(--text);
    }
    .file-actions {
      display: flex;
      gap: 0.5rem;
    }
  }
  .error-message {
    color: var(--red);
    font-size: 0.95rem;
    margin-top: 0.5rem;
  }
}
.custom-templates-list {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.dropdown-toggle {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid $color-accent;
  color: $color-accent;
  margin-bottom: 0.5rem;
  width: 100%;
  max-width: 320px;
  cursor: pointer;
  transition:
    background 0.2s,
    border-color 0.2s;
  &:hover {
    border-color: $color-accent-secondary;
    color: $color-accent-secondary;
  }
}
.custom-templates-rows {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
}
.custom-template-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: var(--card);
  border: 1px solid var(--dark-200);
  border-radius: var(--border-radius-large);
  padding: 0.75rem 1rem;
}
.custom-template-row .template-name {
  font-weight: 500;
  color: $color-accent;
  flex: 1;
}
.custom-template-row .icon-btn {
  background: var(--red);
  border: none;
  border-radius: var(--border-radius);
  padding: 0.3em 0.7em;
  display: flex;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s;
}
.custom-template-row .icon-btn:hover {
  background: var(--red-600);
}
</style>
