<script lang="ts">
  import { SettingsManager } from '$lib';
  import { settings } from '$lib/settings';
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';

  let isLoading = false;
  let saveStatus = '';

  onMount(async () => {
    await SettingsManager.initialize();
  });

  async function updateSetting(key: string, value: any) {
    try {
      isLoading = true;
      // This would need to be implemented in SettingsManager
      console.log(`Update setting ${key} to ${value}`);
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
      // This would need to be implemented in the Rust backend
      console.log('Directory selection not yet implemented');
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  }

  async function selectJavaPath() {
    try {
      // This would need to be implemented in the Rust backend
      console.log('Java path selection not yet implemented');
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
                  value={$settings.minecraft_directory} 
                  readonly
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
                  value={$settings.java_path || 'Auto-detect'} 
                  readonly
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
                  min="1024" 
                  max="16384" 
                  step="512"
                  value={$settings.memory_allocation}
                  on:input={(e) => updateSetting('memory_allocation', parseInt((e.target as HTMLInputElement).value))}
                  class="memory-slider"
                />
                <div class="memory-display">
                  <span class="memory-value">{$settings.memory_allocation}MB</span>
                  <span class="memory-gb">({Math.round($settings.memory_allocation / 1024 * 10) / 10}GB)</span>
                </div>
              </div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="launch-options">Additional JVM Arguments</label>
              <p class="setting-description">Advanced Java options for performance tuning</p>
            </div>
            <div class="setting-control">
              <textarea 
                id="launch-options"
                value={$settings.jvm_args || ''}
                on:blur={(e) => updateSetting('jvm_args', (e.target as HTMLTextAreaElement).value)}
                placeholder="-XX:+UnlockExperimentalVMOptions -XX:+UseG1GC"
                class="jvm-args"
              ></textarea>
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
              <label for="close-behavior">Close Button Behavior</label>
              <p class="setting-description">What happens when you close the launcher</p>
            </div>
            <div class="setting-control">
              <select 
                id="close-behavior"
                value={$settings.close_behavior || 'minimize'}
                on:change={(e) => updateSetting('close_behavior', (e.target as HTMLSelectElement).value)}
                class="behavior-select"
              >
                <option value="close"><Icon name="door" size="sm" /> Exit completely</option>
                <option value="minimize"><Icon name="package" size="sm" /> Minimize to tray</option>
                <option value="hide"><Icon name="eye-off" size="sm" /> Hide window</option>
              </select>
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
                  checked={$settings.auto_update ?? true}
                  on:change={(e) => updateSetting('auto_update', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="analytics">Send Anonymous Analytics</label>
              <p class="setting-description">Help improve the launcher by sharing usage data</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="analytics"
                  type="checkbox" 
                  checked={$settings.analytics ?? false}
                  on:change={(e) => updateSetting('analytics', (e.target as HTMLInputElement).checked)}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <label for="debug-mode">Debug Mode</label>
              <p class="setting-description">Enable detailed logging for troubleshooting</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input 
                  id="debug-mode"
                  type="checkbox" 
                  checked={$settings.debug_mode ?? false}
                  on:change={(e) => updateSetting('debug_mode', (e.target as HTMLInputElement).checked)}
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

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
