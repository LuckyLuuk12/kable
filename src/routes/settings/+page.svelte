<script lang="ts">
  import { SettingsManager } from '$lib';
  import { settings } from '$lib/settings';
  import type { LauncherSettings } from '$lib/types';
  import Icon from '$lib/components/Icon.svelte';
  import { IconManager, selectedTemplate, availableTemplates, isIconsLoading } from '$lib/managers/IconManager';
  import { onMount } from 'svelte';

  let isLoading = false;
  let saveStatus = '';

  // Icon template management
  let showIconUpload = false;
  let uploadError = '';
  let uploadFile: File | null = null;
  let isDragOver = false;

  // Local validation functions
  function validateMemory(value: string): number | null {
    const num = parseInt(value);
    if (isNaN(num) || num < 512 || num > 262144) return null;
    return Math.floor(num / 512) * 512; // Round to nearest 512MB
  }

  function validateNumber(value: string, min: number, max: number): number | null {
    const num = parseInt(value);
    if (isNaN(num) || num < min || num > max) return null;
    return num;
  }

  function validatePath(value: string): string {
    return value.trim();
  }

  onMount(async () => {
    await SettingsManager.initialize();
    await IconManager.initialize();
  });

  async function selectIconTemplate(templateName: string) {
    try {
      await IconManager.setActiveTemplate(templateName);
      saveStatus = 'Icon template updated successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      console.error('Failed to set icon template:', error);
      saveStatus = 'Failed to update template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function handleIconUpload() {
    if (!uploadFile) return;
    
    try {
      uploadError = '';
      isLoading = true;
      
      const content = await uploadFile.text();
      const format = uploadFile.name.endsWith('.yml') || uploadFile.name.endsWith('.yaml') ? 'yaml' : 'json';
      
      // Validate template
      const template = await IconManager.validateTemplate(content, format);
      
      // Install template
      await IconManager.installCustomTemplate(template);
      
      // Clear upload state
      uploadFile = null;
      showIconUpload = false;
      
      saveStatus = `Template "${template.displayName}" installed successfully`;
      setTimeout(() => saveStatus = '', 3000);
      
    } catch (error) {
      uploadError = `Upload failed: ${error}`;
      console.error('Template upload failed:', error);
    } finally {
      isLoading = false;
    }
  }

  async function removeTemplate(templateName: string) {
    try {
      await IconManager.removeCustomTemplate(templateName);
      saveStatus = 'Template removed successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      console.error('Failed to remove template:', error);
      saveStatus = 'Failed to remove template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  // File drag and drop handlers
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
    
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.endsWith('.json') || file.name.endsWith('.yml') || file.name.endsWith('.yaml')) {
        uploadFile = file;
        uploadError = '';
      } else {
        uploadError = 'Please upload a JSON or YAML file';
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
      console.error('Failed to remove template:', error);
      saveStatus = 'Failed to remove template';
      setTimeout(() => saveStatus = '', 2000);
    }
  }

  async function openIconsDirectory() {
    try {
      await IconManager.openIconsDirectory();
    } catch (error) {
      console.error('Failed to open icons directory:', error);
    }
  }

  async function updateSetting(key: keyof LauncherSettings, value: any) {
    try {
      isLoading = true;
      await SettingsManager.updateSetting(key, value);
      saveStatus = 'Saved successfully';
      setTimeout(() => saveStatus = '', 2000);
    } catch (error) {
      console.error('Failed to update setting:', error);
      saveStatus = 'Failed to save';
      setTimeout(() => saveStatus = '', 2000);
    } finally {
      isLoading = false;
    }
  }

  async function selectMinecraftDirectory() {
    try {
      // For now, just show a message that this feature will be implemented
      alert('Directory selection will be implemented in a future update. Please manually enter the path.');
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  }

  async function selectJavaPath() {
    try {
      // For now, just show a message that this feature will be implemented  
      alert('Java path selection will be implemented in a future update. Please manually enter the path.');
    } catch (error) {
      console.error('Failed to select Java path:', error);
    }
  }
</script>

<div class="settings-page">
  <div class="page-header">
    <h1>Settings</h1>
    <p>Configure your launcher preferences</p>
    {#if saveStatus}
      <div class="warning-card" class:success={saveStatus.includes('success')} class:error={saveStatus.includes('Failed')}>
        {saveStatus}
      </div>
    {/if}
  </div>

  {#if $settings}
    <div class="settings-sections">
      <!-- General Settings -->
      <section class="settings-section">
        <h2><Icon name="target" /> General</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="minecraft-dir">Minecraft Directory</label>
              <p class="setting-description">Where your Minecraft installations are stored</p>
            </div>
            <div class="setting-control">
              <div class="path-input">
                <input 
                  id="minecraft-dir"
                  type="text" 
                  bind:value={$settings.minecraft_path}
                  on:blur={(e) => updateSetting('minecraft_path', validatePath((e.target as HTMLInputElement).value))}
                  placeholder="Path to .minecraft directory"
                  class="path-field"
                  style="flex: 1;"
                />
                <button on:click={selectMinecraftDirectory} class="btn btn-secondary">
                  <Icon name="folder" /> Browse
                </button>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="java-path">Java Path</label>
              <p class="setting-description">Java executable to use for running Minecraft</p>
            </div>
            <div class="setting-control">
              <div class="path-input">
                <input 
                  id="java-path"
                  type="text" 
                  bind:value={$settings.java_path}
                  on:blur={(e) => updateSetting('java_path', validatePath((e.target as HTMLInputElement).value))}
                  placeholder="Auto-detect Java"
                  class="path-field"
                  style="flex: 1;"
                />
                <button on:click={selectJavaPath} class="btn btn-secondary">
                  <Icon name="coffee" /> Browse
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Performance Settings -->
      <section class="settings-section">
        <h2><Icon name="zap" /> Performance</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="memory-allocation">Default Memory Allocation</label>
              <p class="setting-description">Default RAM allocated to new installations (can be overridden per installation)</p>
            </div>
            <div class="setting-control">
              <div class="memory-control">
                <input 
                  id="memory-allocation"
                  type="range" 
                  min="512" 
                  max="262144" 
                  step="512"
                  bind:value={$settings.default_memory}
                  class="memory-slider"
                />
                <div class="memory-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.default_memory}
                    on:blur={(e) => {
                      const validated = validateMemory((e.target as HTMLInputElement).value);
                      if (validated !== null) {
                        updateSetting('default_memory', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.default_memory || 0).toString();
                      }
                    }}
                    class="memory-text-input"
                  />
                  <span class="memory-unit">MB</span>
                  <span class="memory-gb">({Math.round(($settings.default_memory || 0) / 1024 * 10) / 10}GB)</span>
                </div>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="max-memory">Maximum Memory Limit</label>
              <p class="setting-description">Maximum RAM that can be allocated to Minecraft</p>
            </div>
            <div class="setting-control">
              <div class="memory-control">
                <input 
                  id="max-memory"
                  type="range" 
                  min={$settings.default_memory || 1024}
                  max="262144" 
                  step="512"
                  bind:value={$settings.max_memory}
                  class="memory-slider"
                />
                <div class="memory-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.max_memory}
                    on:blur={(e) => {
                      const validated = validateMemory((e.target as HTMLInputElement).value);
                      if (validated !== null && validated >= ($settings.default_memory || 1024)) {
                        updateSetting('max_memory', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.max_memory || 0).toString();
                      }
                    }}
                    class="memory-text-input"
                  />
                  <span class="memory-unit">MB</span>
                  <span class="memory-gb">({Math.round(($settings.max_memory || 0) / 1024 * 10) / 10}GB)</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Appearance Settings -->
      <section class="settings-section">
        <h2><Icon name="palette" /> Appearance</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="theme">Theme</label>
              <p class="setting-description">Choose your preferred color scheme</p>
            </div>
            <div class="setting-control">
              <select 
                id="theme"
                value={$settings.theme}
                on:change={(e) => updateSetting('theme', (e.target as HTMLSelectElement).value)}
                class="theme-select"
              >
                <option value="dark"><Icon name="moon" size="sm" /> Dark</option>
                <option value="light"><Icon name="sun" size="sm" /> Light</option>
                <option value="auto"><Icon name="refresh" size="sm" /> Auto</option>
              </select>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="close-behavior">Close Launcher on Game Start</label>
              <p class="setting-description">Close the launcher when Minecraft starts</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="close-behavior"
                  type="checkbox" 
                  checked={$settings.close_launcher_on_game_start}
                  on:change={(e) => updateSetting('close_launcher_on_game_start', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="animation-speed">Animation Speed</label>
              <p class="setting-description">Speed of UI animations</p>
            </div>
            <div class="setting-control">
              <select 
                id="animation-speed"
                value={$settings.animation_speed}
                on:change={(e) => updateSetting('animation_speed', (e.target as HTMLSelectElement).value)}
                class="animation-select"
              >
                <option value="disabled">Disabled</option>
                <option value="slow">Slow</option>
                <option value="normal">Normal</option>
                <option value="fast">Fast</option>
              </select>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="card-spacing">Card Spacing</label>
              <p class="setting-description">Spacing between cards in grid layouts</p>
            </div>
            <div class="setting-control">
              <div class="slider-control">
                <input 
                  id="card-spacing"
                  type="range" 
                  min="0" 
                  max="128" 
                  step="1"
                  bind:value={$settings.card_spacing}
                  class="spacing-slider"
                />
                <div class="value-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.card_spacing}
                    on:blur={(e) => {
                      const validated = validateNumber((e.target as HTMLInputElement).value, 0, 128);
                      if (validated !== null) {
                        updateSetting('card_spacing', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.card_spacing || 0).toString();
                      }
                    }}
                    class="value-text-input"
                  />
                  <span class="spacing-value">px</span>
                </div>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="sidebar-width">Sidebar Width</label>
              <p class="setting-description">Width of the navigation sidebar</p>
            </div>
            <div class="setting-control">
              <div class="slider-control">
                <input 
                  id="sidebar-width"
                  type="range" 
                  min="100" 
                  max="800" 
                  step="25"
                  bind:value={$settings.sidebar_width}
                  class="width-slider"
                />
                <div class="value-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.sidebar_width}
                    on:blur={(e) => {
                      const validated = validateNumber((e.target as HTMLInputElement).value, 100, 800);
                      if (validated !== null) {
                        updateSetting('sidebar_width', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.sidebar_width || 0).toString();
                      }
                    }}
                    class="value-text-input"
                  />
                  <span class="width-value">px</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Icon Template Selection -->
          <div class="setting-item">
            <div class="setting-info">
              <label for="icon-template">Icon Template</label>
              <p class="setting-description">Choose your preferred icon style</p>
            </div>
            <div class="setting-control">
              <select 
                id="icon-template"
                value={$selectedTemplate}
                on:change={(e) => selectIconTemplate((e.target as HTMLSelectElement).value)}
                class="template-select"
              >
                {#each $availableTemplates as template}
                  <option value={template.name}>
                    {template.displayName} {template.type === 'custom' ? '(Custom)' : ''}
                  </option>
                {/each}
              </select>
            </div>
          </div>

          <!-- Custom Template Management -->
          <div class="setting-item">
            <div class="setting-info">
              <span class="section-label">Custom Icon Templates</span>
              <p class="setting-description">Upload and manage custom icon templates</p>
            </div>
            <div class="setting-control template-management">
              <button 
                class="btn btn-outline" 
                on:click={() => showIconUpload = !showIconUpload}
                type="button"
              >
                <Icon name="upload" size="sm" />
                Upload Template
              </button>
              
              <button 
                class="btn btn-outline" 
                on:click={openIconsDirectory}
                type="button"
              >
                <Icon name="folder" size="sm" />
                Open Icons Folder
              </button>
            </div>
          </div>

          {#if showIconUpload}
            <div class="setting-item template-upload">
              <div 
                class="upload-zone" 
                class:error={uploadError}
                class:drag-over={isDragOver}
                on:dragover={handleDragOver}
                on:dragleave={handleDragLeave}
                on:drop={handleDrop}
                role="button"
                tabindex="0"
                on:click={() => document.getElementById('template-file-input')?.click()}
                on:keydown={(e) => e.key === 'Enter' && document.getElementById('template-file-input')?.click()}
              >
                <input 
                  id="template-file-input"
                  type="file" 
                  accept=".json,.yml,.yaml"
                  on:change={handleFileSelect}
                  class="file-input"
                  style="display: none;"
                />
                
                {#if uploadFile}
                  <div class="file-info">
                    <Icon name="file" size="sm" />
                    <span class="file-name">{uploadFile.name}</span>
                    <div class="file-actions">
                      <button 
                        class="btn btn-primary btn-sm" 
                        on:click|stopPropagation={handleIconUpload}
                        disabled={isLoading}
                        type="button"
                      >
                        {#if isLoading}
                          <Icon name="loading" size="sm" />
                          Installing...
                        {:else}
                          <Icon name="install" size="sm" />
                          Install Template
                        {/if}
                      </button>
                      <button 
                        class="btn btn-outline btn-sm" 
                        on:click|stopPropagation={() => { uploadFile = null; uploadError = ''; }}
                        type="button"
                      >
                        <Icon name="x" size="sm" />
                        Cancel
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="upload-placeholder">
                    <Icon name="upload" size="lg" />
                    <h4>Upload Icon Template</h4>
                    <p>Drop a template file here or click to browse</p>
                    <small>Supports JSON (.json) and YAML (.yml, .yaml) files</small>
                  </div>
                {/if}
                
                {#if uploadError}
                  <div class="error-message">
                    <Icon name="error" size="sm" />
                    <span>{uploadError}</span>
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Custom Templates List -->
          {#if $availableTemplates.some(t => t.type === 'custom')}
            <div class="setting-item">
              <div class="custom-templates-list">
                <h4>Installed Custom Templates</h4>
                <div class="templates-grid">
                  {#each $availableTemplates.filter(t => t.type === 'custom') as template}
                    <div class="template-card" class:active={$selectedTemplate === template.name}>
                      <div class="template-info">
                        <span class="template-name">{template.displayName}</span>
                        <small class="template-id">({template.name})</small>
                      </div>
                      <div class="template-actions">
                        {#if $selectedTemplate !== template.name}
                          <button 
                            class="btn btn-outline btn-sm" 
                            on:click={() => selectIconTemplate(template.name)}
                            type="button"
                          >
                            <Icon name="play" size="sm" />
                            Use
                          </button>
                        {:else}
                          <span class="active-indicator">
                            <Icon name="success" size="sm" />
                            Active
                          </span>
                        {/if}
                        <button 
                          class="btn btn-danger btn-sm" 
                          on:click={() => removeTemplate(template.name)}
                          type="button"
                        >
                          <Icon name="delete" size="sm" />
                          Remove
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      </section>

      <!-- Advanced Settings -->
      <section class="settings-section">
        <h2><Icon name="wrench" /> Advanced</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="auto-update">Auto-update Launcher</label>
              <p class="setting-description">Automatically download launcher updates</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="auto-update"
                  type="checkbox" 
                  checked={$settings.auto_update_launcher}
                  on:change={(e) => updateSetting('auto_update_launcher', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="keep-open">Keep Launcher Open</label>
              <p class="setting-description">Keep launcher running after starting Minecraft</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="keep-open"
                  type="checkbox" 
                  checked={$settings.keep_launcher_open}
                  on:change={(e) => updateSetting('keep_launcher_open', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="show-logs">Show Logs on Launch</label>
              <p class="setting-description">Display game logs when Minecraft starts</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="show-logs"
                  type="checkbox" 
                  checked={$settings.show_logs_on_launch}
                  on:change={(e) => updateSetting('show_logs_on_launch', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="experimental">Experimental Features</label>
              <p class="setting-description">Enable experimental launcher features</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="experimental"
                  type="checkbox" 
                  checked={$settings.enable_experimental_features}
                  on:change={(e) => updateSetting('enable_experimental_features', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </section>

      <!-- Network Settings -->
      <section class="settings-section">
        <h2><Icon name="wifi" /> Network & Downloads</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="parallel-downloads">Parallel Downloads</label>
              <p class="setting-description">Number of simultaneous downloads</p>
            </div>
            <div class="setting-control">
              <div class="slider-control">
                <input 
                  id="parallel-downloads"
                  type="range" 
                  min="1" 
                  max="10"
                  step="1"
                  bind:value={$settings.parallel_downloads}
                  class="number-slider"
                />
                <div class="value-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.parallel_downloads}
                    on:blur={(e) => {
                      const validated = validateNumber((e.target as HTMLInputElement).value, 1, 10);
                      if (validated !== null) {
                        updateSetting('parallel_downloads', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.parallel_downloads || 1).toString();
                      }
                    }}
                    class="value-text-input"
                  />
                  <span class="number-unit">downloads</span>
                </div>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="connection-timeout">Connection Timeout (seconds)</label>
              <p class="setting-description">Network timeout for downloads and API calls</p>
            </div>
            <div class="setting-control">
              <div class="slider-control">
                <input 
                  id="connection-timeout"
                  type="range" 
                  min="5" 
                  max="120"
                  step="5"
                  bind:value={$settings.connection_timeout}
                  class="number-slider"
                />
                <div class="value-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.connection_timeout}
                    on:blur={(e) => {
                      const validated = validateNumber((e.target as HTMLInputElement).value, 5, 120);
                      if (validated !== null) {
                        updateSetting('connection_timeout', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.connection_timeout || 5).toString();
                      }
                    }}
                    class="value-text-input"
                  />
                  <span class="number-unit">seconds</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Content Management Settings -->
      <section class="settings-section">
        <h2><Icon name="database" /> Content Management</h2>
        
        <div class="setting-group">
          <div class="setting-item">
            <div class="setting-info">
              <label for="auto-backup">Auto-backup Worlds</label>
              <p class="setting-description">Automatically create backups before modifying worlds</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="auto-backup"
                  type="checkbox" 
                  checked={$settings.auto_backup_worlds}
                  on:change={(e) => updateSetting('auto_backup_worlds', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="max-backups">Maximum World Backups</label>
              <p class="setting-description">How many backups to keep per world</p>
            </div>
            <div class="setting-control">
              <div class="slider-control">
                <input 
                  id="max-backups"
                  type="range" 
                  min="1" 
                  max="50"
                  step="1"
                  bind:value={$settings.max_world_backups}
                  class="number-slider"
                />
                <div class="value-input-row">
                  <input 
                    type="text"
                    bind:value={$settings.max_world_backups}
                    on:blur={(e) => {
                      const validated = validateNumber((e.target as HTMLInputElement).value, 1, 50);
                      if (validated !== null) {
                        updateSetting('max_world_backups', validated);
                      } else {
                        (e.target as HTMLInputElement).value = ($settings.max_world_backups || 1).toString();
                      }
                    }}
                    class="value-text-input"
                  />
                  <span class="number-unit">backups</span>
                </div>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="shader-quality">Shader Quality Preset</label>
              <p class="setting-description">Default quality preset for shader packs</p>
            </div>
            <div class="setting-control">
              <select 
                id="shader-quality"
                value={$settings.shader_quality_preset}
                on:change={(e) => updateSetting('shader_quality_preset', (e.target as HTMLSelectElement).value)}
                class="animation-select"
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="ultra">Ultra</option>
              </select>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="shader-caching">Enable Shader Caching</label>
              <p class="setting-description">Cache compiled shaders for faster loading</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="shader-caching"
                  type="checkbox" 
                  checked={$settings.enable_shader_caching}
                  on:change={(e) => updateSetting('enable_shader_caching', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </section>
    </div>
  {:else}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading settings...</p>
    </div>
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .settings-page {
    max-width: 800px;
    margin: 0 auto;
  }

  .warning-card {
    position: fixed;
    bottom: 1rem;
    width: fit-content;
    @extend .card !optional;
    padding: 1rem;
    margin-bottom: 1.5rem;
    color: $text;
    background: $container;
    border-radius: 0.75rem;
    font-size: 0.875rem;

    &.success {
      border-color: $green;
      background-color: $green-900;
    }
    
    &.error {
      border-color: $red-700;
      background-color: $red-900;
    }
  }

  .settings-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .settings-section {
    @extend .card !optional;
    padding: 2rem;
    
    h2 {
      margin: 0 0 1.5rem 0;
      color: $text;
      font-size: 1.25rem;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 2rem;
    padding: 1.5rem;
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 0.75rem;
    
    @media (max-width: 768px) {
      flex-direction: column;
      gap: 1rem;
    }
  }

  .setting-info {
    flex: 1;
    
    label {
      display: block;
      font-weight: 600;
      color: $text;
      margin-bottom: 0.25rem;
      font-size: 1rem;
    }
    
    .section-label {
      display: block;
      font-weight: 600;
      color: $text;
      margin-bottom: 0.25rem;
      font-size: 1rem;
    }
    
    .setting-description {
      margin: 0;
      color: $placeholder;
      font-size: 0.875rem;
      line-height: 1.4;
    }
  }

  .setting-control {
    min-width: 250px;
    
    @media (max-width: 768px) {
      min-width: unset;
      width: 100%;
    }
  }

  .path-input {
    display: flex;
    gap: 0.5rem;
  }

  .memory-control {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    
    .memory-input-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      
      .memory-text-input {
        background: transparent;
        border: none;
        color: $primary;
        font-weight: 600;
        font-size: 0.9rem;
        text-align: right;
        min-width: 60px;
        max-width: 80px;
        padding: 0;
        border-radius: 4px;
        transition: all 0.2s ease;
        
        &:hover {
          color: rgba($primary, 0.8);
        }
        
        &:focus {
          color: rgba($primary, 0.75);
        }
        
        &::placeholder {
          color: transparent;
        }
      }
      
      .memory-gb {
        color: $placeholder;
        font-size: 0.875rem;
        min-width: 60px;
      }
      
      .memory-unit {
        color: $primary;
        font-weight: 600;
        font-size: 0.9rem;
      }
    }
  }

  .slider-control {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    
    .value-input-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      
      .value-text-input {
        background: transparent;
        border: none;
        color: $primary;
        font-weight: 600;
        font-size: 0.9rem;
        text-align: right;
        padding: 0;
        border-radius: 4px;
        transition: all 0.2s ease;
        
        &:hover {
          color: rgba($primary, 0.8);
        }
        
        &:focus {
          color: rgba($primary, 0.75);
        }
        
        &::placeholder {
          color: transparent;
        }
      }
    }
  }

  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 52px;
    height: 28px;
    
    input {
      opacity: 0;
      width: 0;
      height: 0;
      
      &:checked + .toggle-slider {
        background-color: $primary;
        
        &:before {
          transform: translateX(24px);
        }
      }
    }
    
    .toggle-slider {
      position: absolute;
      cursor: pointer;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: $input;
      transition: 0.3s;
      border-radius: 28px;
      
      &:before {
        position: absolute;
        content: "";
        height: 20px;
        width: 20px;
        left: 4px;
        bottom: 4px;
        background-color: white;
        transition: 0.3s;
        border-radius: 50%;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
      }
    }
  }

  .loading-state {
    @extend .empty-state !optional;
    padding: 4rem;
    
    .spinner {
      width: 40px;
      height: 40px;
      border: 4px solid $input;
      border-top: 4px solid $primary;
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin-bottom: 1rem;
    }
  }

  .spacing-value, .width-value {
    font-weight: 600;
    color: $primary;
    font-size: 0.9rem;
    // min-width: 50px;
    text-align: left;
  }

  /* Icon Template Management Styles */
  .template-management {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .template-upload {
    margin-top: 0.5rem;
    padding: 0;
  }

  .upload-zone {
    border: 2px dashed var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    text-align: center;
    transition: all 0.2s ease;
    background: var(--background-secondary);
    cursor: pointer;
    position: relative;
    
    &:hover {
      border-color: var(--accent-color);
      background: var(--background-hover);
    }
    
    &.drag-over {
      border-color: var(--accent-color);
      background: rgba(74, 144, 226, 0.1);
      transform: scale(1.02);
    }
    
    &.error {
      border-color: var(--error-color);
      background: rgba(220, 53, 69, 0.1);
    }
  }

  .upload-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    color: var(--text-secondary);
    
    h4 {
      margin: 0;
      color: var(--text-primary);
      font-size: 1.1rem;
      font-weight: 600;
    }
    
    p {
      margin: 0;
      font-weight: 500;
    }
    
    small {
      opacity: 0.7;
      font-size: 0.85rem;
    }
  }

  .file-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    
    .file-name {
      font-weight: 500;
      color: var(--text-primary);
      font-size: 1rem;
    }
    
    .file-actions {
      display: flex;
      gap: 0.5rem;
      align-items: center;
    }
  }

  .error-message {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    color: var(--error-color);
    margin-top: 1rem;
    font-size: 0.9rem;
    padding: 0.5rem;
    background: rgba(220, 53, 69, 0.1);
    border-radius: 4px;
  }

  .custom-templates-list {
    width: 100%;
    
    h4 {
      margin: 0 0 1rem 0;
      color: var(--text-primary);
      font-size: 1rem;
      font-weight: 600;
    }
  }

  .templates-grid {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .template-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--background-secondary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    transition: all 0.2s ease;
    
    &:hover {
      background: var(--background-hover);
      border-color: var(--accent-color);
    }
    
    &.active {
      border-color: var(--accent-color);
      background: rgba(74, 144, 226, 0.1);
    }
  }

  .template-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    
    .template-name {
      font-weight: 500;
      color: var(--text-primary);
    }
    
    .template-id {
      color: var(--text-secondary);
      font-size: 0.8rem;
    }
  }

  .template-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .active-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--accent-color);
    font-weight: 500;
    font-size: 0.9rem;
  }

  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    
    :global(.icon) {
      font-size: 0.75rem;
    }
  }

  .btn-danger {
    background: var(--error-color);
    border-color: var(--error-color);
    color: white;
    
    &:hover {
      background: #c82333;
      border-color: #bd2130;
    }
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
