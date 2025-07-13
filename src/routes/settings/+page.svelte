<script lang="ts">
  import { SettingsManager } from '$lib';
  import { settings } from '$lib/settings';
  import type { LauncherSettings } from '$lib/types';
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';

  let isLoading = false;
  let saveStatus = '';

  onMount(async () => {
    await SettingsManager.initialize();
  });

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
                  value={$settings.minecraft_path || ''} 
                  on:blur={(e) => updateSetting('minecraft_path', (e.target as HTMLInputElement).value)}
                  placeholder="Path to .minecraft directory"
                  class="path-field"
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
                  value={$settings.java_path || ''} 
                  on:blur={(e) => updateSetting('java_path', (e.target as HTMLInputElement).value)}
                  placeholder="Auto-detect Java"
                  class="path-field"
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
              <label for="memory-allocation">Memory Allocation</label>
              <p class="setting-description">RAM allocated to Minecraft (in MB)</p>
            </div>
            <div class="setting-control">
              <div class="memory-control">
                <input 
                  id="memory-allocation"
                  type="range" 
                  min="512" 
                  max="262144" 
                  step="512"
                  value={$settings.default_memory}
                  on:input={(e) => updateSetting('default_memory', parseInt((e.target as HTMLInputElement).value))}
                  class="memory-slider"
                />
                <div class="memory-display">
                  <span class="memory-value">{$settings.default_memory}MB</span>
                  <span class="memory-gb">({Math.round($settings.default_memory / 1024 * 10) / 10}GB)</span>
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
                  value={$settings.max_memory}
                  on:input={(e) => updateSetting('max_memory', parseInt((e.target as HTMLInputElement).value))}
                  class="memory-slider"
                />
                <div class="memory-display">
                  <span class="memory-value">{$settings.max_memory}MB</span>
                  <span class="memory-gb">({Math.round($settings.max_memory / 1024 * 10) / 10}GB)</span>
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
              <input 
                id="parallel-downloads"
                type="number" 
                min="1" 
                max="10"
                value={$settings.parallel_downloads}
                on:blur={(e) => updateSetting('parallel_downloads', parseInt((e.target as HTMLInputElement).value))}
                class="number-input"
              />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="connection-timeout">Connection Timeout (seconds)</label>
              <p class="setting-description">Network timeout for downloads and API calls</p>
            </div>
            <div class="setting-control">
              <input 
                id="connection-timeout"
                type="number" 
                min="5" 
                max="120"
                value={$settings.connection_timeout}
                on:blur={(e) => updateSetting('connection_timeout', parseInt((e.target as HTMLInputElement).value))}
                class="number-input"
              />
            </div>
          </div>
        </div>
      </section>

      <!-- UI Settings -->
      <section class="settings-section">
        <h2><Icon name="layout" /> Interface</h2>
        
        <div class="setting-group">
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
              <input 
                id="card-spacing"
                type="range" 
                min="0" 
                max="128" 
                step="1"
                value={$settings.card_spacing}
                on:input={(e) => updateSetting('card_spacing', parseInt((e.target as HTMLInputElement).value))}
                class="spacing-slider"
              />
              <span class="spacing-value">{$settings.card_spacing}px</span>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="sidebar-width">Sidebar Width</label>
              <p class="setting-description">Width of the navigation sidebar</p>
            </div>
            <div class="setting-control">
              <input 
                id="sidebar-width"
                type="range" 
                min="100" 
                max="800" 
                step="25"
                value={$settings.sidebar_width}
                on:input={(e) => updateSetting('sidebar_width', parseInt((e.target as HTMLInputElement).value))}
                class="width-slider"
              />
              <span class="width-value">{$settings.sidebar_width}px</span>
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
              <input 
                id="max-backups"
                type="number" 
                min="1" 
                max="50"
                value={$settings.max_world_backups}
                on:blur={(e) => updateSetting('max_world_backups', parseInt((e.target as HTMLInputElement).value))}
                class="number-input"
              />
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
  .settings-page {
    max-width: 800px;
    margin: 0 auto;
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
      color: var(--text);
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
    background: var(--background);
    border: 1px solid var(--border);
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
      color: var(--text);
      margin-bottom: 0.25rem;
      font-size: 1rem;
    }
    
    .setting-description {
      margin: 0;
      color: var(--text-muted);
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
    
    .path-field {
      flex: 1;
      padding: 0.5rem 0.75rem;
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      background: var(--background);
      color: var(--text);
      font-size: 0.875rem;
    }
  }

  .memory-control {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    
    .memory-slider {
      width: 100%;
      height: 6px;
      border-radius: 3px;
      background: var(--surface-variant);
      outline: none;
      cursor: pointer;
      
      &::-webkit-slider-thumb {
        appearance: none;
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: var(--primary);
        cursor: pointer;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
        
        &:hover {
          transform: scale(1.1);
        }
      }
      
      &::-moz-range-thumb {
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: var(--primary);
        cursor: pointer;
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
      }
    }
    
    .memory-display {
      display: flex;
      justify-content: space-between;
      align-items: center;
      
      .memory-value {
        font-weight: 600;
        color: var(--primary);
        font-size: 1rem;
      }
      
      .memory-gb {
        color: var(--text-muted);
        font-size: 0.875rem;
      }
    }
  }

  .jvm-args {
    width: 100%;
    min-height: 80px;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--background);
    color: var(--text);
    font-family: 'Fira Code', monospace;
    font-size: 0.875rem;
    resize: vertical;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
    }
  }

  .theme-select, .behavior-select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--background);
    color: var(--text);
    font-size: 0.875rem;
    cursor: pointer;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
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
        background-color: var(--primary);
        
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
      background-color: var(--surface-variant);
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
      border: 4px solid var(--surface-variant);
      border-top: 4px solid var(--primary);
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin-bottom: 1rem;
    }
  }

  .number-input {
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.9rem;
    width: 80px;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
      box-shadow: 0 0 0 2px rgba(var(--primary-rgb), 0.2);
    }
  }

  .animation-select {
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.9rem;
    min-width: 120px;
    
    &:focus {
      outline: none;
      border-color: var(--primary);
      box-shadow: 0 0 0 2px rgba(var(--primary-rgb), 0.2);
    }
  }

  .spacing-slider, .width-slider {
    flex: 1;
    margin-right: 12px;
  }

  .spacing-value, .width-value {
    font-weight: 600;
    color: var(--primary);
    font-size: 0.9rem;
    min-width: 50px;
    text-align: center;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
