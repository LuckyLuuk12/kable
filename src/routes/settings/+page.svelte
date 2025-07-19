<script lang="ts">
  import { SettingsManager, Icon, settings, IconManager, selectedTemplate, availableTemplates, Settings, type LaunchOptions } from '$lib';
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

  // No need to initialize managers here - they're already initialized in the layout

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

  // async function updateSetting(key: keyof LauncherSettings, value: any) {
  //   try {
  //     isLoading = true;
  //     await SettingsManager.updateSetting(key, value);
  //     saveStatus = 'Saved successfully';
  //     setTimeout(() => saveStatus = '', 2000);
  //   } catch (error) {
  //     console.error('Failed to update setting:', error);
  //     saveStatus = 'Failed to save';
  //     setTimeout(() => saveStatus = '', 2000);
  //   } finally {
  //     isLoading = false;
  //   }
  // }

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
    <Settings />
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
