<script lang="ts">
  let showCustomTemplates = false;
  import { settings } from "$lib/stores";
  import { IconManager, availableTemplates } from "$lib";
  import { onMount } from "svelte";

  let isWideScreen = true;
  function checkScreen() {
    isWideScreen = window.innerWidth >= 700;
  }
  onMount(() => {
    checkScreen();
    window.addEventListener('resize', checkScreen);
    return () => window.removeEventListener('resize', checkScreen);
  });

  let showIconUpload = false;
  let uploadError = '';
  let uploadFile: File | null = null;
  let isDragOver = false;
  let saveStatus = '';

  async function selectIconTemplate(templateName: string) {
    try {
      await IconManager.setActiveTemplate(templateName);
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
      const template = await IconManager.validateTemplate(content, format);
      await IconManager.installCustomTemplate(template);
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
      await IconManager.removeCustomTemplate(templateName);
      saveStatus = 'Template removed successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      saveStatus = 'Failed to remove template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function openIconsDirectory() {
    try {
      await IconManager.openIconsDirectory();
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
    <div class="setting-item">
      <div class="setting-info">
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
    </div>

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
          <label>Upload Zone</label>
          <p class="setting-description">Drag & drop or click to select a template file (.json, .yml, .yaml)</p>
        </div>
        <div class="setting-control">
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
                    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M6 6L14 14M14 6L6 14" stroke="white" stroke-width="2" stroke-linecap="round"/>
                    </svg>
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
  background: $container;
  border-radius: $border-radius-large;
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
  color: transparent;
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
  border-bottom: 1px solid $dark-200;
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
  color: $text;
  margin-bottom: 0.1rem;
}
.setting-description {
  font-size: 0.95rem;
  color: $placeholder;
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
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
input[type="text"] {
  width: 100%;
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: $border-radius;
  border: 1px solid $dark-200;
  color: $text;
}
select {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: $border-radius;
  border: 1px solid $dark-200;
  color: $text;
}
.template-management {
  display: flex;
  gap: 1rem;
}
.upload-zone {
  border: 2px dashed $primary;
  border-radius: $border-radius-large;
  padding: 1.5rem;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s;
  &.drag-over {
    border-color: $secondary;
  }
  &.error {
    border-color: $red;
  }
}
.upload-placeholder {
  h4 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: $primary;
  }
  p {
    margin: 0 0 1rem 0;
    color: $placeholder;
  }
  .file-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
    .file-name {
      font-weight: 500;
      color: $text;
    }
    .file-actions {
      display: flex;
      gap: 0.5rem;
    }
  }
  .error-message {
    color: $red;
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
  border-radius: $border-radius;
  border: 1px solid $primary;
  color: $primary;
  margin-bottom: 0.5rem;
  width: 100%;
  max-width: 320px;
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s;
  &:hover {
    border-color: $secondary;
    color: $secondary;
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
  background: $card;
  border: 1px solid $dark-200;
  border-radius: $border-radius-large;
  padding: 0.75rem 1rem;
}
.custom-template-row .template-name {
  font-weight: 500;
  color: $primary;
  flex: 1;
}
.custom-template-row .icon-btn {
  background: $red;
  border: none;
  border-radius: $border-radius;
  padding: 0.3em 0.7em;
  display: flex;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s;
}
.custom-template-row .icon-btn:hover {
  background: $red-600;
}
.custom-template-row .icon-btn svg {
  display: block;
}
</style>