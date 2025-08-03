<script lang="ts">
import { selectedInstallation, installations } from '$lib';
import { get } from 'svelte/store';
import type { KableInstallation } from '$lib';

// You may want to fetch the mods for the selected installation from the backend in the future
// For now, we assume each installation has a mods property (array of mod IDs or objects)

let currentInstallation: KableInstallation | null = null;

$:
  currentInstallation = $selectedInstallation;

function selectInstallation(installation: KableInstallation) {
  selectedInstallation.set(installation);
}
</script>

<div class="installation-mods">
  <h2>Installations</h2>
  <select on:change={(e) => selectInstallation($installations[e.target?.selectedIndex])}>
    {#each $installations as inst, i}
      <option value={inst.id} selected={currentInstallation && inst.id === currentInstallation.id}>
        {inst.name}
      </option>
    {/each}
  </select>

  {#if currentInstallation}
    <h3>Mods for {currentInstallation.name}</h3>
    {#if currentInstallation.mods && currentInstallation.mods.length > 0}
      <ul>
        {#each currentInstallation.mods as mod}
          <li>{mod}</li> <!-- Replace with mod name/details if available -->
        {/each}
      </ul>
    {:else}
      <p>No mods installed for this installation.</p>
    {/if}
  {:else}
    <p>No installation selected.</p>
  {/if}
</div>

<style lang="scss">
.installation-mods {
  padding: 1rem;
  border: 1px solid #ccc;
  border-radius: 8px;
  background: #fafafa;
  max-width: 400px;
}
select {
  margin-bottom: 1rem;
}
ul {
  margin: 0;
  padding-left: 1.5rem;
}
</style>
