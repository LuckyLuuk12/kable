<!-- @component
◄!--
@component
SkinSelectionMenu - Manages Minecraft account skins and capes

Provides interface for viewing, adding, editing, and applying skins and capes
to the current Microsoft account. Includes 3D preview and management tools.

@example
```svelte
◄SkinSelectionMenu /►
```
-->
<script lang="ts">
import { onMount } from "svelte";
import { Icon, SkinViewer3D, SkinsService } from "$lib";
import { clickSound, successSound, errorSound } from "$lib/actions";
import type { AccountSkin, AccountCape } from "$lib";

// State
let accountSkins: AccountSkin[] = [];
let capes: AccountCape[] = [];
let activeCape: AccountCape | null = null;
let loading = false;
let error = "";
let hoveredSkinId: string | null = null;

// Edit modal
let showEditModal = false;
let editingSkin: AccountSkin | null = null;
let editName = "";
let editCapeId = "";
let editSlim = false;

// Add modal
let showAddModal = false;
let addFilePath = "";
let addName = "";
let addCapeId = "";
let addSlim = false;

onMount(async () => {
  await loadData();
});

async function loadData(skipCapes = false) {
  loading = true;
  error = "";
  try {
    // Fetch skins and active cape
    const [skinsRes, activeCapeRes] = await Promise.allSettled([
      SkinsService.getAllSkins(),
      SkinsService.getActiveCape(),
    ]);

    if (skinsRes.status === "fulfilled") {
      // Force reactivity by creating new array reference
      accountSkins = [...skinsRes.value];
    } else {
      console.error("Failed to load skins:", skinsRes.reason);
      error = "Failed to load skins. Please try again.";
    }

    if (activeCapeRes.status === "fulfilled") {
      activeCape = activeCapeRes.value;
    } else {
      console.warn("Failed to load active cape:", activeCapeRes.reason);
    }

    // Only fetch capes list if we don't have it yet (and not skipping)
    if (!skipCapes && capes.length === 0) {
      try {
        capes = await SkinsService.getCapes();
      } catch (err: any) {
        console.warn("Failed to load capes list:", err);
        // Don't show error for capes - it's not critical
      }
    }
  } catch (err: any) {
    console.error("Failed to load data:", err);
    error = `Failed to load: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

async function handleApplySkin(skinId: string) {
  loading = true;
  error = "";
  try {
    await SkinsService.applySkin(skinId);
    // Only reload skins and active cape, skip refetching capes list
    await loadData(true);
    // Force reactivity update
    accountSkins = [...accountSkins];
  } catch (err: any) {
    console.error("Apply skin error:", err);
    error = `Failed to apply skin: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

async function handleApplyCape(capeId: string | null) {
  loading = true;
  error = "";
  try {
    await SkinsService.applyCape(capeId);

    // Update active cape locally without refetching everything
    if (capeId === null) {
      activeCape = null;
    } else {
      const found = capes.find((c) => c.id === capeId);
      if (found) activeCape = found;
    }

    // Only verify the active cape status, don't reload all capes
    try {
      const updatedCape = await SkinsService.getActiveCape();
      activeCape = updatedCape;
    } catch (err) {
      console.warn("Failed to verify active cape:", err);
      // Keep local state if verification fails
    }
  } catch (err: any) {
    error = `Failed to apply cape: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

function openEditModal(skin: AccountSkin) {
  editingSkin = skin;
  editName = skin.name || "";
  editCapeId = "";
  editSlim = String(skin.model).toLowerCase() === "slim";
  showEditModal = true;
}

async function saveEdit() {
  if (!editingSkin) return;
  loading = true;
  error = "";
  try {
    // Strip 'local_' prefix if present (backend expects just 'skin_1', not 'local_skin_1')
    const skinId = editingSkin.id.replace(/^local_/, "");
    await SkinsService.modifySkin(skinId, editName, editCapeId, editSlim);
    // Only reload skins, skip refetching capes
    await loadData(true);
    // Force reactivity update
    accountSkins = [...accountSkins];
    showEditModal = false;
    editingSkin = null;
  } catch (err: any) {
    console.error("Edit error:", err);
    error = `Failed to update: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

async function removeSkin(skinId: string, e: Event) {
  e.stopPropagation();
  const skinName =
    accountSkins.find((s) => s.id === skinId)?.name || "this skin";
  if (
    !confirm(
      `Are you sure you want to remove "${skinName}"?\n\nThis action cannot be undone.`,
    )
  )
    return;
  loading = true;
  error = "";
  try {
    // Strip 'local_' prefix if present (backend expects just 'skin_1', not 'local_skin_1')
    const backendSkinId = skinId.replace(/^local_/, "");
    await SkinsService.removeSkin(backendSkinId);
    // Only reload skins, skip refetching capes
    await loadData(true);
    // Force reactivity update
    accountSkins = [...accountSkins];
  } catch (err: any) {
    console.error("Remove error:", err);
    error = `Failed to remove: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

async function openUploadDialog() {
  // Reset modal state and open immediately
  addFilePath = "";
  addName = "";
  addCapeId = "";
  addSlim = false; // Default to Classic/Normal
  showAddModal = true;
}

async function selectFile() {
  const path = await SkinsService.selectSkinFile();
  if (path) {
    addFilePath = path;
    // Auto-populate name from filename if empty
    if (!addName) {
      const filename = path.split(/[\\\/]/).pop() || "";
      addName = filename.replace(/\.(png|jpg|jpeg)$/i, "");
    }
  }
}

async function uploadSkin() {
  if (!addFilePath || !addName) return;
  loading = true;
  error = "";
  try {
    // Upload skin first
    await SkinsService.uploadSkin({
      model: addSlim ? "Slim" : "Classic",
      file_path: addFilePath,
    });

    // Get only LOCAL skins to find the newly added one
    const localSkins = await SkinsService.getLocalSkins();

    // Find the most recently added local skin (they have IDs like 'local_skin_13')
    // The newest one will have the highest skin number
    const localSkinIds = localSkins
      .map((s) => s.id)
      .filter((id) => id.startsWith("local_skin_"))
      .sort((a, b) => {
        const numA = parseInt(a.replace("local_skin_", "")) || 0;
        const numB = parseInt(b.replace("local_skin_", "")) || 0;
        return numB - numA; // Descending order
      });

    const newestLocalSkinId = localSkinIds[0];

    if (newestLocalSkinId) {
      // Strip 'local_' prefix for backend (backend expects 'skin_13', not 'local_skin_13')
      const backendSkinId = newestLocalSkinId.replace(/^local_/, "");
      await SkinsService.modifySkin(backendSkinId, addName, addCapeId, addSlim);
    }

    // Reload data to reflect the changes
    await loadData(true);

    // Close modal and show success
    showAddModal = false;
    // Force reactivity update
    accountSkins = [...accountSkins];
  } catch (err: any) {
    console.error("Upload error:", err);
    error = `Failed to upload: ${err?.message || err}`;
  } finally {
    loading = false;
  }
}

function getModel(m: string): "classic" | "slim" | "auto" {
  return m?.toLowerCase() === "slim" ? "slim" : "classic";
}
</script>

<div class="skins-container">
  <div class="header">
    <div class="title-row">
      <div class="title">
        <h1>Skins & Capes</h1>
        <span class="subtitle">Customize your appearance</span>
      </div>
      <button
        use:clickSound
        class="upload-btn"
        on:click={openUploadDialog}
        disabled={loading}
      >
        <Icon name="upload" size="sm" forceType="svg" />
        Upload Skin
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-msg">
      <Icon name="alert" size="sm" />
      {error}
      <button use:clickSound on:click={() => (error = "")}
        ><Icon name="close" size="sm" /></button
      >
    </div>
  {/if}

  {#if loading && !accountSkins.length}
    <div class="loading">
      <Icon name="refresh" size="lg" />
      <span>Loading...</span>
    </div>
  {:else}
    <div class="content-grid">
      <!-- Skins Section (Left) -->
      <section class="skins-section">
        <div class="section-header">
          <h2>🖼️ Skins</h2>
          <span class="count">{accountSkins.length}</span>
        </div>
        {#if accountSkins.length === 0}
          <div class="empty">
            <Icon name="image" size="xl" />
            <h3>No Skins</h3>
            <p>Upload your first skin</p>
            <button use:clickSound on:click={openUploadDialog}
              ><Icon name="upload" size="sm" />Upload</button
            >
          </div>
        {:else}
          <div class="skins-list">
            {#each accountSkins as skin (skin.id)}
              <div
                class="skin-card"
                role="button"
                tabindex="0"
                class:current={SkinsService.isSkinActive(skin)}
                on:mouseenter={() => (hoveredSkinId = skin.id)}
                on:mouseleave={() => (hoveredSkinId = null)}
              >
                <div class="preview">
                  {#if skin.url}
                    <SkinViewer3D
                      skinUrl={skin.url}
                      height={140}
                      model={getModel(skin.model)}
                      animation={hoveredSkinId === skin.id ? "walk" : "idle"}
                    />
                  {:else}
                    <Icon name="user" size="lg" />
                  {/if}
                </div>
                <div class="details">
                  <div class="title-row">
                    <h4>{SkinsService.getSkinDisplayName(skin)}</h4>
                  </div>
                  <div class="meta">
                    <span class="tag">{skin.model}</span>
                    {#if SkinsService.isSkinActive(skin)}<span
                        class="badge active-badge">Active</span
                      >{/if}
                    {#if skin.uploaded_date}
                      <span class="date"
                        ><Icon name="calendar" size="sm" />{new Date(
                          skin.uploaded_date * 1000,
                        ).toLocaleDateString()}</span
                      >
                    {/if}
                  </div>
                  <div class="actions">
                    {#if !SkinsService.isSkinActive(skin)}
                      <button
                        use:successSound
                        class="apply"
                        on:click={() => handleApplySkin(skin.id)}
                        disabled={loading}
                        ><Icon
                          name="check"
                          size="sm"
                          forceType="svg"
                        />Apply</button
                      >
                    {/if}
                    <button
                      use:clickSound
                      on:click={() => openEditModal(skin)}
                      disabled={loading}
                      title="Edit"><Icon name="edit" size="sm" /></button
                    >
                    {#if !SkinsService.isSkinActive(skin)}
                      <button
                        use:errorSound
                        class="danger"
                        on:click={(e) => removeSkin(skin.id, e)}
                        disabled={loading}
                        title="Remove"><Icon name="trash" size="sm" /></button
                      >
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Capes Section (Right) -->
      {#if capes.length > 0}
        <section class="capes-section">
          <div class="section-header">
            <h2>🎽 Capes</h2>
            <span class="count">{capes.length}</span>
          </div>
          <div class="capes-grid">
            <button
              use:successSound
              class="cape-card"
              class:active={!activeCape}
              on:click={() => handleApplyCape(null)}
              disabled={loading}
            >
              <div class="cape-preview no-cape">
                <Icon name="close" size="md" />
              </div>
              <h4>No Cape</h4>
              {#if !activeCape}<span class="badge">Active</span>{/if}
            </button>
            {#each capes as cape}
              <button
                use:successSound
                class="cape-card"
                class:active={SkinsService.isCapeActive(cape)}
                on:click={() => handleApplyCape(cape.id)}
                disabled={loading}
              >
                <div class="cape-preview">
                  {#if cape.url}<img
                      src={cape.url}
                      alt={cape.alias || cape.id}
                    />{:else}<Icon name="image" size="md" />{/if}
                </div>
                <h4>{SkinsService.getCapeDisplayName(cape)}</h4>
                {#if SkinsService.isCapeActive(cape)}<span class="badge"
                    >Active</span
                  >{/if}
              </button>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}
</div>

{#if showEditModal && editingSkin}
  <div
    class="modal-bg"
    on:click={() => (showEditModal = false)}
    on:keypress={(e) => e.key === "Escape" && (showEditModal = false)}
    role="button"
    tabindex="0"
  >
    <div
      class="modal"
      on:click|stopPropagation
      on:keypress={(e) => e.key === "Escape" && (showEditModal = false)}
      role="button"
      tabindex="0"
    >
      <div class="modal-header">
        <h3>Edit Skin</h3>
        <button use:clickSound on:click={() => (showEditModal = false)}
          ><Icon name="close" size="sm" /></button
        >
      </div>
      <div class="modal-body">
        <label
          >Name <input
            type="text"
            bind:value={editName}
            placeholder="Skin name"
          /></label
        >
        <label
          >Cape <select bind:value={editCapeId}
            ><option value="">None</option>{#each capes as c}<option
                value={c.id}>{SkinsService.getCapeDisplayName(c)}</option
              >{/each}</select
          ></label
        >
        <label
          >Model <div class="radio-group">
            <label
              ><input
                type="radio"
                bind:group={editSlim}
                value={false}
              />Classic</label
            ><label
              ><input
                type="radio"
                bind:group={editSlim}
                value={true}
              />Slim</label
            >
          </div></label
        >
      </div>
      <div class="modal-footer">
        <button use:clickSound on:click={() => (showEditModal = false)}
          >Cancel</button
        >
        <button
          use:successSound
          class="primary"
          on:click={saveEdit}
          disabled={!editName}><Icon name="check" size="sm" />Save</button
        >
      </div>
    </div>
  </div>
{/if}

{#if showAddModal}
  <div
    class="modal-bg"
    on:click={() => (showAddModal = false)}
    on:keypress={(e) => e.key === "Escape" && (showAddModal = false)}
    role="button"
    tabindex="0"
  >
    <div
      class="modal"
      on:click|stopPropagation
      on:keypress={(e) => e.key === "Escape" && (showAddModal = false)}
      role="button"
      tabindex="0"
    >
      <div class="modal-header">
        <h3>Upload New Skin</h3>
        <button use:clickSound on:click={() => (showAddModal = false)}
          ><Icon name="close" size="sm" /></button
        >
      </div>
      <div class="modal-body">
        <label class="file-select-label">
          Skin File
          <button
            type="button"
            class="file-select-btn"
            use:clickSound
            on:click={selectFile}
          >
            <Icon name="file" size="sm" />
            {addFilePath ? "Change File" : "Select PNG File"}
          </button>
          {#if addFilePath}
            <div class="file-info-inline">
              <Icon name="check" size="sm" />
              <span>{addFilePath.split(/[\\\\/]/).pop()}</span>
            </div>
          {/if}
        </label>
        <label>
          Name
          <input
            type="text"
            bind:value={addName}
            placeholder="Enter skin name"
            required
          />
        </label>
        <label>
          Model Type
          <div class="radio-group">
            <label>
              <input
                type="radio"
                bind:group={addSlim}
                value={false}
                checked
              />Normal (Classic)
            </label>
            <label>
              <input type="radio" bind:group={addSlim} value={true} />Slim
              (Alex)
            </label>
          </div>
        </label>
        <label>
          Cape (Optional)
          <select bind:value={addCapeId}>
            <option value="">No Cape</option>
            {#each capes as c}
              <option value={c.id}>{SkinsService.getCapeDisplayName(c)}</option>
            {/each}
          </select>
        </label>
      </div>
      <div class="modal-footer">
        <button use:clickSound on:click={() => (showAddModal = false)}
          >Cancel</button
        >
        <button
          use:successSound
          class="primary"
          on:click={uploadSkin}
          disabled={!addName || !addFilePath}
          ><Icon name="upload" size="sm" />Upload</button
        >
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
.skins-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 10%, transparent);
  flex-shrink: 0;

  .title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;

    .title {
      display: flex;
      align-items: baseline;
      gap: 0.75rem;

      h1 {
        font-size: 1.5rem;
        font-weight: 700;
        background: linear-gradient(135deg, var(--primary), var(--secondary));
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        margin: 0;
      }

      .subtitle {
        color: var(--text-secondary);
        font-size: 0.85rem;
      }
    }

    .upload-btn {
      padding: 0.6rem 1.2rem;
      background: linear-gradient(135deg, var(--primary), var(--secondary));
      color: var(--text-white);
      border: none;
      border-radius: 0.5rem;
      font-weight: 600;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      transition: all 0.2s;
      box-shadow: 0 2px 8px color-mix(in srgb, var(--primary), 25%, transparent);

      &:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px
          color-mix(in srgb, var(--primary), 35%, transparent);
      }
      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }
  }
}

.error-msg {
  margin: 0.75rem 1rem 0;
  padding: 0.75rem 1rem;
  background: color-mix(in srgb, var(--red), 10%, transparent);
  border: 1px solid var(--red);
  border-radius: 0.5rem;
  color: var(--red);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;

  button {
    background: none;
    border: none;
    color: var(--red);
    cursor: pointer;
    padding: 0.25rem;
    margin-left: auto;
  }
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  gap: 1rem;
  color: var(--text-secondary);
  flex: 1;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 1rem;
  padding: 1rem;
  overflow: hidden;
  flex: 1;
  min-height: 0;
}

section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  overflow: hidden;
  min-height: 0;
}

.skins-section {
  min-width: 0;
}

.capes-section {
  width: 440px;
  flex-shrink: 0;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border);

  h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .count {
    color: var(--text-secondary);
    font-size: 0.85rem;
    padding: 0.2rem 0.6rem;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 1rem;
  }
}

.capes-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.75rem;
  overflow-y: auto;
  padding-right: 0.25rem;

  &::-webkit-scrollbar {
    width: 6px;
  }

  &::-webkit-scrollbar-track {
    background: var(--background);
    border-radius: 3px;
  }

  &::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;

    &:hover {
      background: var(--primary);
    }
  }

  .cape-card {
    background: var(--card);
    border: 2px solid var(--border);
    border-radius: 0.5rem;
    padding: 0;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    text-align: center;

    &:hover:not(:disabled) {
      border-color: var(--primary);
      transform: translateY(-2px);
      box-shadow: 0 4px 12px
        color-mix(in srgb, var(--primary), 20%, transparent);
    }

    &.active {
      border-color: var(--green);
      background: color-mix(in srgb, var(--green), 5%, transparent);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .cape-preview {
      width: 100%;
      aspect-ratio: 1;
      background: var(--background);
      display: flex;
      align-items: center;
      justify-content: center;
      overflow: hidden;

      &.no-cape {
        background: color-mix(in srgb, var(--red), 10%, transparent);
        color: var(--red);
      }

      img {
        width: 100%;
        height: 100%;
        object-fit: contain;
      }
    }

    h4 {
      font-size: 0.85rem;
      font-weight: 600;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      padding: 0.25rem 0.75rem 0.75rem;
    }

    .badge {
      padding: 0.15rem 0.5rem;
      margin: 0 0.75rem 0.75rem;
      border-radius: 0.25rem;
      font-size: 0.7rem;
      font-weight: 600;
      background: var(--green);
      color: var(--text-white);
      display: inline-block;
      align-self: center;
    }
  }
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 3rem 1.5rem;
  gap: 0.75rem;
  border: 2px dashed var(--border);
  border-radius: 1rem;
  text-align: center;

  h3 {
    margin: 0;
    font-size: 1.25rem;
  }
  p {
    margin: 0;
    color: var(--text-secondary);
  }

  button {
    margin-top: 0.75rem;
    padding: 0.6rem 1.2rem;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    color: var(--text-white);
    border: none;
    border-radius: 0.5rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
}

.skins-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  overflow-y: auto;
  padding-right: 0.25rem;
  max-width: 100%;

  &::-webkit-scrollbar {
    width: 6px;
  }

  &::-webkit-scrollbar-track {
    background: var(--background);
    border-radius: 3px;
  }

  &::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;

    &:hover {
      background: var(--primary);
    }
  }

  .skin-card {
    background: var(--card);
    border: 2px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
    transition: all 0.2s;
    flex: 1 1 fit-content;
    display: flex;
    flex-direction: row;

    &:hover {
      border-color: var(--primary);
      box-shadow: 0 4px 12px
        color-mix(in srgb, var(--primary), 15%, transparent);
    }

    &.current {
      border-color: var(--green);
      background: color-mix(in srgb, var(--green), 3%, transparent);
    }

    .preview {
      width: 140px;
      height: 140px;
      flex-shrink: 0;
      background: linear-gradient(
        135deg,
        color-mix(in srgb, var(--primary), 5%, transparent),
        color-mix(in srgb, var(--secondary), 5%, transparent)
      );
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text-secondary);
    }

    .details {
      flex: 1;
      padding: 0.75rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      justify-content: center;

      .title-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 0.5rem;

        h4 {
          margin: 0;
          font-size: 0.9rem;
          font-weight: 600;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          flex: 1;
        }
      }

      .meta {
        display: flex;
        gap: 0.4rem;
        flex-wrap: wrap;
        font-size: 0.75rem;

        .tag {
          padding: 0.15rem 0.4rem;
          background: var(--background);
          border-radius: 0.25rem;
          color: var(--text-secondary);
        }

        .active-badge {
          padding: 0.15rem 0.4rem;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-weight: 600;
          background: var(--green);
          color: var(--text-white);
        }

        .date {
          display: flex;
          align-items: center;
          gap: 0.2rem;
          color: var(--text-secondary);
        }
      }

      .actions {
        display: flex;
        gap: 0.4rem;

        button {
          padding: 0.4rem 0.5rem;
          background: var(--background);
          border: 1px solid var(--border);
          border-radius: 0.4rem;
          cursor: pointer;
          transition: all 0.15s;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 0.3rem;
          font-weight: 500;
          font-size: 0.8rem;
          color: var(--text);

          &:hover:not(:disabled) {
            background: var(--card);
            border-color: var(--primary);
            color: var(--primary);
          }

          &.apply {
            background: var(--green);
            color: var(--text-white);
            border: none;

            &:hover:not(:disabled) {
              background: color-mix(in srgb, var(--green), 20%, black);
              box-shadow: 0 2px 6px
                color-mix(in srgb, var(--green), 30%, transparent);
              color: var(--text-white);
            }
          }

          &.danger:hover:not(:disabled) {
            background: color-mix(in srgb, var(--red), 10%, transparent);
            border-color: var(--red);
            color: var(--red);
          }

          &:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }
        }
      }
    }
  }
}

.modal-bg {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 2rem;
}

.modal {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 1rem;
  max-width: 500px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);

  .modal-header {
    padding: 1.5rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;

    h3 {
      margin: 0;
      font-size: 1.25rem;
      font-weight: 600;
    }

    button {
      background: none;
      border: none;
      color: var(--text);
      cursor: pointer;
      padding: 0.25rem;
      &:hover {
        color: var(--primary);
      }
    }
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;

    label {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      font-weight: 600;
      font-size: 0.9rem;

      &.file-select-label {
        .file-select-btn {
          padding: 0.75rem;
          background: var(--background);
          border: 2px dashed var(--border);
          border-radius: 0.5rem;
          color: var(--text);
          font-size: 0.95rem;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 0.5rem;
          font-weight: 500;
          transition: all 0.2s;

          &:hover {
            border-color: var(--primary);
            background: color-mix(in srgb, var(--primary), 5%, transparent);
            color: var(--primary);
          }
        }

        .file-info-inline {
          padding: 0.5rem 0.75rem;
          background: color-mix(in srgb, var(--green), 10%, transparent);
          border: 1px solid var(--green);
          border-radius: 0.5rem;
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: var(--green);
          font-size: 0.85rem;
          font-weight: 500;
          margin-top: 0.5rem;

          span {
            flex: 1;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
          }
        }
      }
      font-weight: 600;
      font-size: 0.9rem;

      input,
      select {
        padding: 0.75rem;
        background: var(--background);
        border: 1px solid var(--border);
        border-radius: 0.5rem;
        color: var(--text);
        font-size: 0.95rem;

        &:focus {
          outline: none;
          border-color: var(--primary);
        }
      }
    }

    .radio-group {
      display: flex;
      gap: 1rem;

      label {
        flex: 1;
        flex-direction: row;
        padding: 0.75rem;
        background: var(--background);
        border: 2px solid var(--border);
        border-radius: 0.5rem;
        cursor: pointer;
        align-items: center;
        gap: 0.5rem;
        font-weight: 500;

        &:has(input:checked) {
          border-color: var(--primary);
          background: color-mix(in srgb, var(--primary), 10%, transparent);
        }

        input {
          cursor: pointer;
        }
      }
    }
  }

  .modal-footer {
    padding: 1.5rem;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;

    button {
      padding: 0.75rem 1.5rem;
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      font-weight: 600;
      cursor: pointer;
      background: var(--background);
      color: var(--text);

      &.primary {
        background: linear-gradient(135deg, var(--primary), var(--secondary));
        color: var(--text-white);
        border: none;
        box-shadow: 0 2px 8px
          color-mix(in srgb, var(--primary), 25%, transparent);
        display: flex;
        align-items: center;
        gap: 0.5rem;

        &:hover:not(:disabled) {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px
            color-mix(in srgb, var(--primary), 35%, transparent);
        }
      }

      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }
  }
}

@media (max-width: 1024px) {
  .content-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
  }

  .capes-section {
    width: 100%;
    max-height: 350px;
  }

  .skins-section {
    min-height: 0;
  }
}

@media (max-width: 768px) {
  .header .title-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;

    .upload-btn {
      width: 100%;
      justify-content: center;
    }
  }

  .skins-list .skin-card {
    flex-direction: column;

    .preview {
      width: 100%;
      height: 160px;
    }

    .details {
      padding: 0.75rem;
    }
  }

  .capes-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .capes-section {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .capes-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
