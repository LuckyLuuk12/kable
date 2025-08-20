<script lang="ts">
  import { settings } from "$lib/stores";
  import { Icon, IconService, availableTemplates, SettingsService } from "$lib";
  import { onMount } from "svelte";
  
  let showCustomTemplates = false;
  let showIconUpload = false;
  let uploadError = '';
  let uploadFile: File | null = null;
  let isDragOver = false;
  let saveStatus = '';

  // CSS themes functionality
  let showCssUpload = false;
  let cssUploadError = '';
  let cssUploadFile: File | null = null;
  let cssIsDragOver = false;
  let cssThemes: string[] = ['default'];
  let showCustomThemes = false;

  // Built-in themes that shouldn't be removable
  const builtInThemes = ['default', 'KasaiSora-Theme', 'Modrinth-Theme'];
  
  // Helper function to get only custom (user-uploaded) themes
  $: customThemes = cssThemes.filter(theme => !builtInThemes.includes(theme));
  $: hasCustomThemes = customThemes.length > 0;

  // Load available CSS themes on mount
  onMount(async () => {
    await loadCssThemes();
  });

  async function loadCssThemes() {
    try {
      cssThemes = await SettingsService.getCssThemes();
    } catch (error) {
      console.error('Failed to load CSS themes:', error);
      cssThemes = ['default'];
    }
  }

  async function selectCssTheme(themeName: string) {
    try {
      await SettingsService.setSelectedCssTheme(themeName);
      $settings.appearance.selected_css_theme = themeName;
      
      // Trigger theme reload in the layout
      if (typeof window !== 'undefined' && (window as any).reloadCustomCSS) {
        await (window as any).reloadCustomCSS();
      }
      
      saveStatus = 'CSS theme updated successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      saveStatus = 'Failed to update CSS theme';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function handleCssUpload() {
    if (!cssUploadFile) return;
    try {
      cssUploadError = '';
      const content = await cssUploadFile.text();
      const themeName = cssUploadFile.name.replace('.css', '');
      await SettingsService.saveCssTheme(themeName, content);
      await loadCssThemes(); // Refresh the themes list
      showCssUpload = false;
      saveStatus = `CSS theme "${themeName}" uploaded successfully`;
      setTimeout(() => saveStatus = '', 3000);
      cssUploadFile = null;
    } catch (error) {
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
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.endsWith('.css')) {
        cssUploadFile = file;
        cssUploadError = '';
      } else {
        cssUploadError = 'Please select a .css file';
      }
    }
  }
  function handleCssFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      if (file.name.endsWith('.css')) {
        cssUploadFile = file;
        cssUploadError = '';
      } else {
        cssUploadError = 'Please select a .css file';
      }
    }
  }

  async function removeCssTheme(themeName: string) {
    // Prevent removal of built-in themes
    if (builtInThemes.includes(themeName)) {
      saveStatus = 'Cannot remove built-in themes';
      setTimeout(() => saveStatus = '', 2000);
      return;
    }
    
    if (!confirm(`Are you sure you want to remove the "${themeName}" CSS theme?`)) return;
    try {
      await SettingsService.deleteCssTheme(themeName);
      await loadCssThemes(); // Refresh the themes list
      saveStatus = 'CSS theme removed successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      saveStatus = 'Failed to remove CSS theme';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function openCssThemesDirectory() {
    try {
      await SettingsService.openCssThemesDirectory();
    } catch (error) {
      saveStatus = 'Failed to open CSS themes directory';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function selectIconTemplate(templateName: string) {
    try {
      await IconService.setActiveTemplate?.(templateName);
      $settings.appearance.selected_icon_template = templateName;
      saveStatus = 'Icon template updated successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      saveStatus = 'Failed to update template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }
  async function handleIconUpload() {
    if (!uploadFile) return;
    try {
      uploadError = '';
      const content = await uploadFile.text();
      const format = uploadFile.name.endsWith('.yml') || uploadFile.name.endsWith('.yaml') ? 'yaml' : 'json';
      const template = await IconService.validateTemplate?.(content, format);
      await IconService.installCustomTemplate?.(template);
      showIconUpload = false;
      saveStatus = `Template "${template.displayName || uploadFile.name}" installed successfully`;
      setTimeout(() => saveStatus = '', 3000);
      uploadFile = null;
    } catch (error) {
      uploadError = `Upload failed: ${error}`;
    }
  }
  // ...existing code...

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }
  function handleDragLeave() { isDragOver = false; }
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.endsWith('.json') || file.name.endsWith('.yml') || file.name.endsWith('.yaml')) {
      }
    }
  }
  function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      uploadFile = file;
      uploadError = '';
    }
  }
  async function removeCustomTemplate(templateName: string) {
    if (!confirm('Are you sure you want to remove this icon template?')) return;
    try {
      await IconService.removeCustomTemplate?.(templateName);
      saveStatus = 'Template removed successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      saveStatus = 'Failed to remove template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function openIconsDirectory() {
    try {
      await IconService.openIconsDirectory?.();
    } catch (error) {
      saveStatus = 'Failed to open icons directory';
      setTimeout(() => saveStatus = '', 2000);
    }
  }
</script>


<div class="settings-tab">
  <h2>Appearance Settings</h2>
  <p>Customize the look and feel of the launcher.</p>
  <form>
    <!-- CSS Theme Section -->
    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>CSS Theme</label>
        <p class="setting-description">Choose or upload a custom CSS theme</p>
      </div>
      <div class="setting-control">
        <select bind:value={$settings.appearance.selected_css_theme} on:change={(e) => selectCssTheme((e.target as HTMLSelectElement).value)}>
          {#each cssThemes as theme}
            <option value={theme}>{theme === 'default' ? 'Default (No Custom CSS)' : theme}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>CSS Theme Management</label>
        <p class="setting-description">Upload, remove, or open CSS themes folder</p>
      </div>
      <div class="setting-control">
        <button type="button" on:click={() => showCssUpload = !showCssUpload}>
          {showCssUpload ? 'Cancel Upload' : 'Upload Custom Theme'}
        </button>
        <button type="button" on:click={openCssThemesDirectory}>
          Open Themes Directory
        </button>
      </div>
    </div>

    {#if showCssUpload}
      <div class="setting-item">
        <div class="setting-info">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label>Upload Zone</label>
          <p class="setting-description">Drag & drop or click to select a CSS theme file (.css)</p>
        </div>
        <div class="setting-control">
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div class="upload-zone {cssIsDragOver ? 'drag-over' : ''} {cssUploadError ? 'error' : ''}"
            role="form"
            tabindex="-1"
            on:dragover={handleCssDragOver}
            on:dragleave={handleCssDragLeave}
            on:drop={handleCssDrop}
            on:keydown={(e) => e.key === 'Enter' && document.getElementById('css-file-input')?.click()}
            on:click={() => document.getElementById('css-file-input')?.click()}>
            <input type="file" id="css-file-input" accept=".css" on:change={handleCssFileSelect} style="display:none;" />
            <div class="upload-placeholder">
              <h4>Drag & drop or click to select a CSS file</h4>
              <p>Accepted: .css</p>
              {#if cssUploadFile}
                <div class="file-info">
                  <span class="file-name">{cssUploadFile.name}</span>
                  <div class="file-actions">
                    <button type="button" on:click={handleCssUpload}>Upload</button>
                    <button type="button" on:click={() => cssUploadFile = null}>Remove</button>
                  </div>
                </div>
              {/if}
              {#if cssUploadError}
                <div class="error-message">{cssUploadError}</div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}

    {#if hasCustomThemes}
      <div class="setting-item">
        <div class="setting-info">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label>Custom CSS Themes</label>
          <p class="setting-description">Manage your custom CSS themes (built-in themes cannot be removed)</p>
        </div>
        <div class="setting-control custom-templates-list">
          <button type="button" class="dropdown-toggle" on:click={() => showCustomThemes = !showCustomThemes}>
            {showCustomThemes ? 'Hide Custom Themes' : 'Manage Custom Themes'}
          </button>
          {#if showCustomThemes}
            <div class="custom-templates-rows">
              {#each customThemes as theme}
                <div class="custom-template-row">
                  <span class="template-name">{theme}</span>
                  <button type="button" class="icon-btn btn-danger" title="Remove Custom Theme" on:click={() => removeCssTheme(theme)}>
                    <Icon name="delete" />
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
    <!-- TODO: OUDATED THEME SELECTION 
    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control ->
        <label>Theme</label>
        <p class="setting-description">Choose your preferred theme</p>
      </div>
      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label><input type="radio" name="theme" value="light" bind:group={$settings.appearance.theme} /> Light</label>
            <label><input type="radio" name="theme" value="dark" bind:group={$settings.appearance.theme} /> Dark</label>
            <label><input type="radio" name="theme" value="system" bind:group={$settings.appearance.theme} /> System</label>
          </div>
        {:else}
          <select id="theme" bind:value={$settings.appearance.theme}>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>
          </select>
        {/if}
      </div>
    </div>-->

    <div class="setting-item">
      <div class="setting-info">
        <label for="language">Language</label>
        <p class="setting-description">e.g. en, nl, fr</p>
      </div>
      <div class="setting-control">
        <input type="text" id="language" bind:value={$settings.appearance.language} placeholder="e.g. en, nl, fr" />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="extra-spacing">Extra Spacing (px)</label>
        <p class="setting-description">Adjust spacing in UI elements</p>
      </div>
      <div class="setting-control slider-control">
        <input type="range" id="extra-spacing-slider" min="0" max="128" bind:value={$settings.appearance.extra_spacing} />
        <input type="number" id="extra-spacing" min="0" max="128" bind:value={$settings.appearance.extra_spacing} />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="sidebar-width">Sidebar Width (px)</label>
        <p class="setting-description">Width of the sidebar in pixels</p>
      </div>
      <div class="setting-control slider-control">
        <input type="range" id="sidebar-width-slider" min="200" max="1000" bind:value={$settings.appearance.sidebar_width} />
        <input type="number" id="sidebar-width" min="200" max="1000" bind:value={$settings.appearance.sidebar_width} />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="selected-icon-template">Icon Template</label>
        <p class="setting-description">Choose or upload a custom icon template</p>
      </div>
      <div class="setting-control">
        <select id="selected-icon-template" bind:value={$settings.appearance.selected_icon_template} on:change={(e) => selectIconTemplate((e.target as HTMLSelectElement).value)}>
          {#each $availableTemplates as template}
            <option value={template.name}>{template.displayName || template.name}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>Icon Template Management</label>
        <p class="setting-description">Upload, remove, or open icon templates folder</p>
      </div>
      <div class="setting-control template-management">
        <button type="button" on:click={() => showIconUpload = !showIconUpload}>
          {showIconUpload ? 'Cancel Upload' : 'Upload Custom Template'}
        </button>
        <button type="button" on:click={openIconsDirectory}>
          Open Icons Directory
        </button>
      </div>
    </div>

    {#if showIconUpload}
      <div class="setting-item">
        <div class="setting-info">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label>Upload Zone</label>
          <p class="setting-description">Drag & drop or click to select a template file (.json, .yml, .yaml)</p>
        </div>
        <div class="setting-control">
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div class="upload-zone {isDragOver ? 'drag-over' : ''} {uploadError ? 'error' : ''}"
            role="form"
            tabindex="-1"
            on:dragover={handleDragOver}
            on:dragleave={handleDragLeave}
            on:drop={handleDrop}
            on:keydown={(e) => e.key === 'Enter' && document.getElementById('template-file-input')?.click()}
            on:click={() => document.getElementById('template-file-input')?.click()}>
            <input type="file" id="template-file-input" accept=".json,.yml,.yaml" on:change={handleFileSelect} style="display:none;" />
            <div class="upload-placeholder">
              <h4>Drag & drop or click to select a template file</h4>
              <p>Accepted: .json, .yml, .yaml</p>
              {#if uploadFile}
                <div class="file-info">
                  <span class="file-name">{uploadFile.name}</span>
                  <div class="file-actions">
                    <button type="button" on:click={handleIconUpload}>Upload</button>
                    <button type="button" on:click={() => uploadFile = null}>Remove</button>
                  </div>
                </div>
              {/if}
              {#if uploadError}
                <div class="error-message">{uploadError}</div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}

    {#if $availableTemplates.some(t => t.type === 'custom')}
      <div class="setting-item">
        <div class="setting-info">
          <!-- svelte-ignore a11y_label_has_associated_control -->
          <label>Custom Icon Templates</label>
          <p class="setting-description">Manage your custom icon templates</p>
        </div>
        <div class="setting-control custom-templates-list">
          <button type="button" class="dropdown-toggle" on:click={() => showCustomTemplates = !showCustomTemplates}>
            {showCustomTemplates ? 'Hide Custom Templates' : 'Manage Custom Templates'}
          </button>
          {#if showCustomTemplates}
            <div class="custom-templates-rows">
              {#each $availableTemplates.filter(t => t.type === 'custom') as template}
                <div class="custom-template-row">
                  <span class="template-name">{template.displayName || template.name}</span>
                      <button type="button" class="icon-btn btn-danger" title="Remove" on:click={() => removeCustomTemplate(template.name)}>
                        <Icon name="delete" />
                      </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </form>
  {#if saveStatus}
    <div class="warning-card" class:success={saveStatus.includes('success')} class:error={saveStatus.includes('Failed')}>
      {saveStatus}
    </div>
  {/if}
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.settings-tab {
  background: var(--container);
  border-radius: var(--border-radius-large);
  box-shadow: 0 0.125rem 0.5rem rgba(0,0,0,0.08);
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
  background: linear-gradient(to right, $primary, $secondary);
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
  border: 2px dashed var(--primary);
  border-radius: var(--border-radius-large);
  padding: 1.5rem;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s;
  &.drag-over {
    border-color: var(--secondary);
  }
  &.error {
    border-color: var(--red);
  }
}
.upload-placeholder {
  h4 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: var(--primary);
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
  border: 1px solid var(--primary);
  color: var(--primary);
  margin-bottom: 0.5rem;
  width: 100%;
  max-width: 320px;
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s;
  &:hover {
    border-color: var(--secondary);
    color: var(--secondary);
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
  color: var(--primary);
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