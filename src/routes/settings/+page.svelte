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
    max-width: 100%;
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

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
