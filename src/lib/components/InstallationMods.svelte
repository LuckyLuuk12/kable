<script lang="ts">
import { installations, selectedInstallation, InstallationService } from '$lib';
import { onMount } from 'svelte';
import { get } from 'svelte/store';
import type { KableInstallation, ModJarInfo } from '$lib';

let currentInstallation: KableInstallation | null = null;
let selectedId: string = '';
let mods: ModJarInfo[] = [];
let loading = false;
let error: string | null = null;

// Reactively update currentInstallation and mods when selectedId changes
$: {
  const inst = get(installations).find(i => i.id === selectedId) || null;
  currentInstallation = inst;
  selectedInstallation.set(inst);
  if (currentInstallation) loadMods(currentInstallation);
  else mods = [];
}

async function loadMods(installation: KableInstallation) {
  loading = true;
  error = null;
  try {
    mods = await InstallationService.getModInfo(installation);
  } catch (e: any) {
    error = e?.message || 'Failed to load mods info';
    mods = [];
  } finally {
    loading = false;
  }
}

onMount(() => {
  const inst = get(selectedInstallation);
  selectedId = inst?.id || '';
});
</script>

<div class="installation-mods">
  <h2>Installations</h2>
  <select
    bind:value={selectedId}
  >
    <option value="" disabled>Select installation...</option>
    {#each $installations as inst}
      <option value={inst.id}>{inst.name}</option>
    {/each}
  </select>

  {#if currentInstallation}
    <h3>Mods for {currentInstallation.name}</h3>
    {#if loading}
      <p>Loading mods...</p>
    {:else if error}
      <p class="error">{error}</p>
    {:else if mods.length > 0}
      <div class="mods-list">
        {#each mods as mod}
          <div class="mod-box">
            <div class="mod-title">{mod.mod_name || mod.file_name}</div>
            <div class="mod-details">
              <span>File: <b>{mod.file_name}</b></span><br>
              {#if mod.mod_version}<span>Version: <b>{mod.mod_version}</b></span><br>{/if}
              {#if mod.loader}<span>Loader: <b>{mod.loader}</b></span><br>{/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <p>No mods installed for this installation.</p>
    {/if}
  {:else}
    <p>No installation selected.</p>
  {/if}
</div>

<style lang="scss">
</style>
