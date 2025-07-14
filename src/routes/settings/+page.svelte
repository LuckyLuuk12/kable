<script lang="ts">
  import { SettingsManager } from '$lib';
  import { settings } from '$lib/settings';
  import type { LauncherSettings } from '$lib/types';
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';

  let isLoading = false;
  let saveStatus = '';

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

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
