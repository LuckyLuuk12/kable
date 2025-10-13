<script lang="ts">
  import { InstallationService } from '$lib/services/InstallationService';
  import type { KableInstallation } from '$lib/types';

  export let installation: KableInstallation;
  export let isOpen: boolean | undefined = undefined;
  
  // Initialize as undefined, will be set when modal opens
  let edited: KableInstallation | undefined = undefined;
  let javaArgsString: string = '';
  // JSON editor for parameters_map
  let parametersJson: string = '{}';
  let dialogRef: HTMLDialogElement;

  import { tick } from 'svelte';
  let showOptional = false;

  // Watch isOpen and ensure dialogRef is bound before showing/closing the dialog
  $: (async () => {
    // wait for DOM updates so dialogRef is available
    await tick();
    if (isOpen != undefined && isOpen) {
      // Reinitialize edited fields from the current installation when opening
      if (installation) {
        edited = structuredClone(installation);
        javaArgsString = edited.java_args?.join(' ') || '';
        parametersJson = JSON.stringify(edited.parameters_map || {}, null, 2);
      }
      showOptional = false;
      showDialog();
    } else if (isOpen != undefined && !isOpen) {
      close();
    }
  })();
  function showDialog() {
    dialogRef?.showModal();
  }
  function close() {
    dialogRef?.close();
  }

  function handleInput(e: Event, field: keyof KableInstallation) {
    if (!edited) return; // Safety check
    const target = e.target as HTMLInputElement;
    edited = { ...edited, [field]: target.value };
  }

  function handleJavaArgsInput(e: Event) {
    const target = e.target as HTMLInputElement;
    javaArgsString = target.value;
  }

  async function pickFolder(field: keyof KableInstallation) {
    // Trigger the hidden folder input for the requested field
    const inputId = `folder-input-${String(field)}`;
    const input = document.getElementById(inputId) as HTMLInputElement | null;
    input?.click();
  }

  async function pickIconFile() {
    // Trigger the hidden icon file input
    const input = document.getElementById('icon-file-input') as HTMLInputElement | null;
    input?.click();
  }

  // Handler for when a folder is selected via the hidden input
  function handleFolderSelect(e: Event, field: keyof KableInstallation) {
    if (!edited) return; // Safety check
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;
    // Use webkitRelativePath to determine the selected folder root when available
    const first = files[0];
    let folderPath: string | null = null;
    const anyFirst = first as any;
    // Some desktop webviews and Tauri variants expose a `.path` on File objects
    if (anyFirst.path) {
      // use the parent directory of the first file
      const p: string = anyFirst.path as string;
      const normalized = p.replace(/\\/g, '/');
      const parts = normalized.split('/');
      if (parts.length > 1) parts.pop();
      folderPath = parts.join('/');
    } else if (anyFirst.webkitRelativePath) {
      const rel = anyFirst.webkitRelativePath as string;
      const parts = rel.split('/');
      if (parts.length > 0) folderPath = parts[0];
    }
    if (!folderPath) folderPath = first.name || null;
    if (folderPath) edited = { ...edited, [field]: folderPath } as KableInstallation;
    // clear the input value so re-selecting the same folder triggers change
    target.value = '';
  }

  // Handler for icon file selection
  function handleIconFileSelect(e: Event) {
    if (!edited) return; // Safety check
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    // Read file as data URL (base64) and store as the icon string
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string | null;
      if (result) {
        edited = { ...edited, icon: result } as KableInstallation;
      }
    };
    reader.onerror = (err) => {
      console.warn('Failed to read icon file', err);
    };
    reader.readAsDataURL(file);
    // clear the input value so selecting same file again triggers change
    target.value = '';
  }

  async function confirmEdit() {
    if (!edited) return; // Safety check
    
    edited.java_args = javaArgsString.split(' ').filter(arg => arg.length > 0);
    // merge parameters from JSON editor if valid
    try {
      const parsed = JSON.parse(parametersJson || '{}');
      if (parsed && typeof parsed === 'object') {
        edited.parameters_map = parsed;
      }
    } catch (e) {
      // if parsing fails, keep existing map and log
      console.warn('Failed to parse parameters JSON, keeping original map', e);
    }

    console.log('[Modal] About to update installation:', { id: edited.id, edited });
    await InstallationService.updateInstallation(edited.id, edited);
    close();
  }

  function cancelEdit() {
    close();
  }
</script>

<dialog bind:this={dialogRef} class="edit-installation-modal">
  <h2>Edit Installation{#if installation && installation.name} — {installation.name}{/if}</h2>
  {#if edited}
  <form on:submit|preventDefault={confirmEdit} class="two-column-form">
    <div class="left-column">
      <label>
        Name:
        <input type="text" bind:value={edited.name} placeholder={installation?.name || ''} on:input={(e) => handleInput(e, 'name')} />
      </label>

      <label>
        Icon:
        <div class="file-row">
          <input type="text" bind:value={edited.icon} placeholder={installation?.icon || ''} on:input={(e) => handleInput(e, 'icon')} />
          <button type="button" class="btn" on:click={pickIconFile}>Choose...</button>
        </div>
      </label>

      <label>
        Description (optional):
        <textarea bind:value={edited.description} placeholder={installation?.description || ''}></textarea>
      </label>

      <label class="favorite-row">
        <span>Favorite:</span>
        <input type="checkbox" bind:checked={edited.favorite} />
      </label>
    </div>

    <div class="right-column">
      <details bind:open={showOptional} class="optional-section">
        <summary>Optional settings</summary>
        <div class="optional-content">
          <label>
            Java Args:
            <input type="text" bind:value={javaArgsString} on:input={handleJavaArgsInput} />
          </label>

          <label>
            Dedicated Mods Folder (optional):
            <div class="file-row">
              <input type="text" bind:value={edited.dedicated_mods_folder} on:input={(e) => handleInput(e, 'dedicated_mods_folder')} />
              <button type="button" class="btn" on:click={() => pickFolder('dedicated_mods_folder')}>Browse...</button>
            </div>
          </label>

          <label>
            Dedicated Resource Pack Folder (optional):
            <div class="file-row">
              <input type="text" bind:value={edited.dedicated_resource_pack_folder} on:input={(e) => handleInput(e, 'dedicated_resource_pack_folder')} />
              <button type="button" class="btn" on:click={() => pickFolder('dedicated_resource_pack_folder')}>Browse...</button>
            </div>
          </label>

          <label>
            Dedicated Shaders Folder (optional):
            <div class="file-row">
              <input type="text" bind:value={edited.dedicated_shaders_folder} on:input={(e) => handleInput(e, 'dedicated_shaders_folder')} />
              <button type="button" class="btn" on:click={() => pickFolder('dedicated_shaders_folder')}>Browse...</button>
            </div>
          </label>

          <label>
            Parameters (JSON object):
            <textarea bind:value={parametersJson} rows="6"></textarea>
          </label>
        </div>
      </details>
    </div>

    <div class="actions" style="grid-column: 1 / -1;">
      <button type="submit" class="btn btn-primary">Confirm</button>
      <button type="button" class="btn btn-secondary" on:click={cancelEdit}>Cancel</button>
    </div>
  </form>
  {/if}
  <!-- Hidden inputs for folder and icon selection (used instead of tauri dialog) -->
  <input id="icon-file-input" type="file" accept="image/png,image/jpeg,image/svg+xml,image/x-icon,image/webp" style="display:none;" on:change={handleIconFileSelect} />
  <!-- Folder inputs: use webkitdirectory to allow picking a folder and read its relative paths -->
  <input id="folder-input-dedicated_mods_folder" type="file" webkitdirectory style="display:none;" on:change={(e) => handleFolderSelect(e, 'dedicated_mods_folder')} />
  <input id="folder-input-dedicated_resource_pack_folder" type="file" webkitdirectory style="display:none;" on:change={(e) => handleFolderSelect(e, 'dedicated_resource_pack_folder')} />
  <input id="folder-input-dedicated_shaders_folder" type="file" webkitdirectory style="display:none;" on:change={(e) => handleFolderSelect(e, 'dedicated_shaders_folder')} />
</dialog>

<style lang="scss">
.edit-installation-modal {
  padding: 2rem;
  background: var(--container);
  border-radius: var(--border-radius);
  max-width: 80vw;
  margin: 0 auto;
  h2 {
    margin-bottom: 1rem;
    color: var(--text);
  }
  form {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    align-items: start;
    label {
      color: var(--text);
      font-size: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
    .file-row {
      display: flex;
      gap: 0.5rem;
      input[type="text"] {
        flex: 1;
      }
      .btn {
        padding: 0.25rem 0.75rem;
        border-radius: 0.375rem;
        border: 1px solid rgba(0,0,0,0.1);
        background: var(--surface);
        cursor: pointer;
      }
    }
    .actions {
      display: flex;
      gap: 1rem;
      justify-content: flex-end;
      button {
        padding: 0.5rem 1.5rem;
        border-radius: var(--border-radius);
        border: none;
        font-size: 1rem;
        cursor: pointer;
      }
    }
    .left-column {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    .right-column {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    .favorite-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
    .optional-section summary {
      cursor: pointer;
      font-weight: 600;
      margin-bottom: 0.5rem;
      color: var(--text);
    }
    .optional-content {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
    }
  }
}
</style>
