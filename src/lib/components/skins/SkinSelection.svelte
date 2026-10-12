<script lang="ts">
import { app, Icon, SkinViewer3D, type AccountSkin, type SkinModel, type SkinUploadConfig } from "$lib";
import { open } from "@tauri-apps/plugin-dialog";
import { onMount } from "svelte";

const skinsService = app.skinsService;

let selectedSkinId = $state<string | null>(null);
let loading = $state(false);
let error = $state("");
let showEditModal = $state(false);
let editingSkin = $state<AccountSkin | null>(null);
let editName = $state("");
let editCapeId = $state("");
let editSlim = $state(false);

let showAddModal = $state(false);
let addFilePath = $state("");
let addName = $state("");
let addCapeId = $state("");
let addSlim = $state(false);

let accountSkins = $derived(skinsService.allSkins);
let capes = $derived(skinsService.capes);
let activeSkin = $derived(accountSkins.find(isSkinActive) ?? null);
let selectedSkin = $derived(accountSkins.find((skin) => skin.id === selectedSkinId) ?? activeSkin ?? accountSkins[0] ?? null);
let localSkinCount = $derived(accountSkins.filter(isLocalSkin).length);
let onlineSkinCount = $derived(accountSkins.filter((skin) => !isLocalSkin(skin)).length);
let accountAvailable = $derived(Boolean(app.authService.activeAccount));

onMount(() => {
  const handleUploadRequest = () => openUploadDialog();

  window.addEventListener("kable:open-skin-upload", handleUploadRequest);

  return () => {
    window.removeEventListener("kable:open-skin-upload", handleUploadRequest);
  };
});

function errorMessage(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function isLocalSkin(skin: AccountSkin): boolean {
  return skinsService.localSkins.some((localSkin) => localSkin.id === skin.id);
}

function isSkinActive(skin: AccountSkin): boolean {
  if (skin.is_current) return true;

  const currentUrl = skinsService.currentSkin?.url;
  return Boolean(currentUrl && skin.url && currentUrl === skin.url);
}

function skinName(skin: AccountSkin): string {
  return skin.name?.trim() || skin.id;
}

function skinModel(model: SkinModel | string): "classic" | "slim" {
  return String(model).toLowerCase() === "slim" ? "slim" : "classic";
}

function backendSkinId(id: string): string {
  return id.replace(/^local_/, "");
}

function selectSkin(skin: AccountSkin): void {
  selectedSkinId = skin.id;
  error = "";
}

function dismissError(): void {
  error = "";
  skinsService.error = null;
}

function openUploadDialog(): void {
  addFilePath = "";
  addName = "";
  addCapeId = "";
  addSlim = false;
  error = "";
  showAddModal = true;
}

function openEditModal(skin: AccountSkin): void {
  if (!isLocalSkin(skin)) return;

  editingSkin = skin;
  editName = skin.name ?? "";
  editCapeId = "";
  editSlim = skinModel(skin.model) === "slim";
  showEditModal = true;
  error = "";
}

async function applySelectedSkin(): Promise<void> {
  if (!selectedSkin || !isLocalSkin(selectedSkin)) return;

  if (!accountAvailable) {
    error = "Sign in to a Minecraft account before applying a skin.";
    return;
  }

  loading = true;
  error = "";

  try {
    const success = await skinsService.applySkin(backendSkinId(selectedSkin.id));

    if (!success) {
      throw new Error(skinsService.error ?? "Failed to apply skin.");
    }

    if (skinsService.error) error = skinsService.error;
  } catch (err) {
    error = `Failed to apply skin: ${errorMessage(err)}`;
  } finally {
    loading = false;
  }
}

async function saveEdit(): Promise<void> {
  if (!editingSkin || !editName.trim()) return;

  loading = true;
  error = "";

  try {
    const success = await skinsService.modifySkin(backendSkinId(editingSkin.id), {
      name: editName.trim(),
      capeId: editCapeId || undefined,
      slim: editSlim,
    });

    if (!success) {
      throw new Error(skinsService.error ?? "Failed to update skin.");
    }

    showEditModal = false;
    editingSkin = null;
  } catch (err) {
    error = `Failed to update skin: ${errorMessage(err)}`;
  } finally {
    loading = false;
  }
}

async function removeSkin(skin: AccountSkin, event: MouseEvent): Promise<void> {
  event.stopPropagation();

  if (!isLocalSkin(skin)) return;

  if (!confirm(`Remove "${skinName(skin)}" from your local skin library?\n\nThis cannot be undone.`)) {
    return;
  }

  loading = true;
  error = "";

  try {
    const success = await skinsService.removeLocalSkin(backendSkinId(skin.id));

    if (!success) {
      throw new Error(skinsService.error ?? "Failed to remove skin.");
    }

    if (selectedSkinId === skin.id) selectedSkinId = null;

    if (editingSkin?.id === skin.id) {
      showEditModal = false;
      editingSkin = null;
    }
  } catch (err) {
    error = `Failed to remove skin: ${errorMessage(err)}`;
  } finally {
    loading = false;
  }
}

async function selectFile(): Promise<void> {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Minecraft skin", extensions: ["png"] }],
    });

    if (typeof selected !== "string") return;

    addFilePath = selected;

    if (!addName) {
      const filename = selected.split(/[\\/]/).pop() ?? "";
      addName = filename.replace(/\.png$/i, "");
    }
  } catch (err) {
    error = `Failed to select skin file: ${errorMessage(err)}`;
  }
}

function skinNumber(id: string): number {
  const match = backendSkinId(id).match(/^skin_(\d+)$/);
  return match ? Number(match[1]) : -1;
}

async function uploadSkin(): Promise<void> {
  const name = addName.trim();
  if (!addFilePath || !name) return;

  loading = true;
  error = "";

  try {
    const existingIds = new Set(skinsService.localSkins.map((skin) => skin.id));

    const config: SkinUploadConfig = {
      model: addSlim ? "Slim" : "Classic",
      file_path: addFilePath,
    };

    const response = await skinsService.addLocalSkin(config);

    if (!response) {
      throw new Error(skinsService.error ?? "Failed to save skin.");
    }

    const newSkin =
      skinsService.localSkins.find((skin) => !existingIds.has(skin.id)) ?? [...skinsService.localSkins].sort((a, b) => skinNumber(b.id) - skinNumber(a.id))[0];

    if (!newSkin) {
      throw new Error("The skin was saved, but its local entry could not be identified.");
    }

    const modified = await skinsService.modifySkin(newSkin.id, {
      name,
      capeId: addCapeId || undefined,
      slim: addSlim,
    });

    if (!modified) {
      throw new Error(skinsService.error ?? "The skin was saved, but its metadata could not be updated.");
    }

    selectedSkinId = newSkin.id;
    showAddModal = false;
  } catch (err) {
    error = `Failed to upload skin: ${errorMessage(err)}`;
  } finally {
    loading = false;
  }
}

function closeModals(): void {
  if (loading) return;

  showEditModal = false;
  showAddModal = false;
  editingSkin = null;
}

function handleModalKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") closeModals();
}
</script>

<div class="skin-selection">
  {#if error || skinsService.error}
    <div class="error-message" role="alert">
      <Icon name="alert" forceType="svg" size="sm" />
      <span>{error || skinsService.error}</span>
      <button type="button" class="icon-button" onclick={dismissError} aria-label="Dismiss error">
        <Icon name="close" forceType="svg" size="sm" />
      </button>
    </div>
  {/if}

  <div class="skins-workspace">
    <section class="selected-panel" aria-label="Selected skin">
      {#if selectedSkin}
        <div class="preview-toolbar">
          <div class="preview-heading">
            <h2 title={skinName(selectedSkin)}>{skinName(selectedSkin)}</h2>
            <span class="preview-subtitle">
              {isLocalSkin(selectedSkin) ? "Local skin" : "Account skin"}
            </span>
          </div>

          {#if isSkinActive(selectedSkin)}
            <span class="status-badge">
              <span class="status-dot"></span>
              Active
            </span>
          {/if}
        </div>

        <div class="large-preview">
          {#if selectedSkin.url}
            <SkinViewer3D skinUrl={selectedSkin.url} model={skinModel(selectedSkin.model)} animationMode="all" zoom={0.9} />
          {:else}
            <div class="preview-placeholder">
              <Icon name="user" forceType="svg" size="xl" />
              <span>No preview available</span>
            </div>
          {/if}
        </div>

        <div class="selected-details">
          <div class="skin-properties">
            <span class="property">
              {skinModel(selectedSkin.model) === "slim" ? "Slim" : "Classic"}
            </span>
            <span class="property">
              {isLocalSkin(selectedSkin) ? "Local skin" : "Account skin"}
            </span>
            {#if selectedSkin.uploaded_date}
              <span class="property">
                Added {new Date(selectedSkin.uploaded_date * 1000).toLocaleDateString()}
              </span>
            {/if}
          </div>

          <div class="selected-actions">
            {#if isLocalSkin(selectedSkin)}
              {#if !isSkinActive(selectedSkin)}
                <button
                  type="button"
                  class="action-button"
                  onclick={applySelectedSkin}
                  disabled={loading || !accountAvailable}
                  title={!accountAvailable ? "Sign in to apply a skin" : "Apply this skin"}>
                  <Icon name="check" forceType="svg" size="sm" />
                  Apply skin
                </button>
              {:else}
                <span class="active-label">
                  <Icon name="check" forceType="svg" size="sm" />
                  Currently applied
                </span>
              {/if}

              <button type="button" class="secondary-button" onclick={() => openEditModal(selectedSkin)} disabled={loading}>
                <Icon name="edit" forceType="svg" size="sm" />
                Edit
              </button>

              <button
                type="button"
                class="icon-button danger-button"
                onclick={(event) => removeSkin(selectedSkin!, event)}
                disabled={loading}
                aria-label="Remove selected skin"
                title="Remove skin">
                <Icon name="trash" forceType="svg" size="sm" />
              </button>
            {:else}
              <span class="muted-message">Account skin</span>
            {/if}
          </div>
        </div>
      {:else}
        <div class="empty-preview">
          <Icon name="user" forceType="svg" size="xl" />
          <h2>No skins yet</h2>
          <p>Upload a PNG skin to start your collection.</p>
          <button type="button" class="action-button" onclick={openUploadDialog}>
            <Icon name="upload" forceType="svg" size="sm" />
            Upload skin
          </button>
        </div>
      {/if}
    </section>

    <section class="library-panel" aria-label="Skin library">
      <header class="library-header">
        <div>
          <h2>Library</h2>
          <span class="library-summary">{localSkinCount} local · {onlineSkinCount} account</span>
        </div>
        <span class="library-count">{accountSkins.length}</span>
      </header>

      {#if accountSkins.length === 0}
        <div class="empty-library">
          <Icon name="image" forceType="svg" size="xl" />
          <p>Your skin library is empty.</p>
        </div>
      {:else}
        <div class="skin-list">
          {#each accountSkins as skin (skin.id)}
            <button
              type="button"
              class="skin-list-item"
              class:selected={selectedSkin?.id === skin.id}
              onclick={() => selectSkin(skin)}
              disabled={loading}
              aria-pressed={selectedSkin?.id === skin.id}>
              <div class="skin-thumbnail">
                {#if skin.url}
                  <SkinViewer3D skinUrl={skin.url} height={76} model={skinModel(skin.model)} animationMode="idle-only" />
                {:else}
                  <Icon name="user" forceType="svg" size="md" />
                {/if}
              </div>

              <div class="skin-list-details">
                <span class="skin-list-name" title={skinName(skin)}>{skinName(skin)}</span>
                <span class="skin-list-meta">
                  {skinModel(skin.model) === "slim" ? "Slim" : "Classic"}
                  · {isLocalSkin(skin) ? "Local" : "Account"}
                </span>
                {#if isSkinActive(skin)}
                  <span class="list-active-label">Active</span>
                {/if}
              </div>

              <Icon name={selectedSkin?.id === skin.id ? "check" : "chevron-right"} forceType="svg" size="sm" />
            </button>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

{#if showEditModal && editingSkin}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) closeModals();
    }}
    onkeydown={handleModalKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="modal" role="dialog" aria-modal="true" aria-labelledby="edit-skin-title">
      <header class="modal-header">
        <h2 id="edit-skin-title">Edit skin</h2>
        <button type="button" class="icon-button" onclick={closeModals} aria-label="Close dialog">
          <Icon name="close" forceType="svg" size="sm" />
        </button>
      </header>

      <div class="modal-body">
        <label class="field">
          <span>Name</span>
          <input type="text" bind:value={editName} placeholder="Skin name" maxlength="100" />
        </label>

        <label class="field">
          <span>Cape metadata</span>
          <select bind:value={editCapeId}>
            <option value="">None</option>
            {#each capes as cape (cape.id)}
              <option value={cape.id}>{cape.alias?.trim() || cape.id}</option>
            {/each}
          </select>
          <span class="field-hint">Local metadata only. This does not equip the cape.</span>
        </label>

        <fieldset class="field">
          <legend>Model</legend>
          <div class="radio-group">
            <label><input type="radio" bind:group={editSlim} value={false} /> Classic</label>
            <label><input type="radio" bind:group={editSlim} value={true} /> Slim</label>
          </div>
        </fieldset>
      </div>

      <footer class="modal-footer">
        <button type="button" class="secondary-button" onclick={closeModals} disabled={loading}> Cancel </button>
        <button type="button" class="action-button" onclick={saveEdit} disabled={!editName.trim() || loading}>
          <Icon name="check" forceType="svg" size="sm" />
          Save changes
        </button>
      </footer>
    </section>
  </div>
{/if}

{#if showAddModal}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) closeModals();
    }}
    onkeydown={handleModalKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="modal" role="dialog" aria-modal="true" aria-labelledby="add-skin-title">
      <header class="modal-header">
        <h2 id="add-skin-title">Upload skin</h2>
        <button type="button" class="icon-button" onclick={closeModals} aria-label="Close dialog">
          <Icon name="close" forceType="svg" size="sm" />
        </button>
      </header>

      <div class="modal-body">
        <div class="field">
          <span>PNG file</span>
          <button type="button" class="file-picker" onclick={selectFile} disabled={loading}>
            <Icon name="file" forceType="svg" size="sm" />
            {addFilePath ? "Change file" : "Choose PNG file"}
          </button>

          {#if addFilePath}
            <span class="selected-file">
              <Icon name="check" forceType="svg" size="sm" />
              {addFilePath.split(/[\\/]/).pop()}
            </span>
          {/if}
        </div>

        <label class="field">
          <span>Name</span>
          <input type="text" bind:value={addName} placeholder="Skin name" maxlength="100" />
        </label>

        <fieldset class="field">
          <legend>Model</legend>
          <div class="radio-group">
            <label><input type="radio" bind:group={addSlim} value={false} /> Classic</label>
            <label><input type="radio" bind:group={addSlim} value={true} /> Slim</label>
          </div>
        </fieldset>

        <label class="field">
          <span>Cape metadata (optional)</span>
          <select bind:value={addCapeId}>
            <option value="">None</option>
            {#each capes as cape (cape.id)}
              <option value={cape.id}>{cape.alias?.trim() || cape.id}</option>
            {/each}
          </select>
          <span class="field-hint">Saved with this local skin. Equipping a cape is separate.</span>
        </label>
      </div>

      <footer class="modal-footer">
        <button type="button" class="secondary-button" onclick={closeModals} disabled={loading}> Cancel </button>
        <button type="button" class="action-button" onclick={uploadSkin} disabled={!addName.trim() || !addFilePath || loading}>
          <Icon name="upload" forceType="svg" size="sm" />
          Upload skin
        </button>
      </footer>
    </section>
  </div>
{/if}

<style lang="scss">
.skin-selection {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: $color-text;
}

.error-message {
  display: flex;
  align-items: center;
  gap: $space-md;
  margin: $space-md $space-lg 0;
  padding: $space-sm $space-md;
  border: 1px solid $color-error;
  border-radius: $radius-md;
  background: $color-error-muted;
  color: $color-error;
  font-size: 0.85rem;

  > span {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }
}

.skins-workspace {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(280px, 0.8fr);
  gap: $space-lg;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  padding: $space-lg;
}

.selected-panel,
.library-panel {
  min-width: 0;
  min-height: 0;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.selected-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-toolbar,
.library-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  flex: 0 0 auto;
  padding: $space-md $space-lg;
  border-bottom: 1px solid $color-border-muted;
}

.preview-heading {
  min-width: 0;

  h2 {
    margin: 0;
    overflow: hidden;
    font-size: 1rem;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.preview-subtitle,
.library-summary {
  display: block;
  margin-top: $space-xs;
  color: $color-text-muted;
  font-size: 0.75rem;
}

.status-badge,
.active-label {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  color: $color-success;
  font-size: 0.78rem;
  white-space: nowrap;
}

.status-badge {
  padding: $space-1 $space-sm;
  border: 1px solid $color-success;
  border-radius: $radius-md;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: $color-success;
}

.large-preview {
  // display: flex;
  // align-items: center;
  // justify-content: center;
  flex: 1;
  min-height: auto;
  overflow: hidden;
  // padding: $space-md;
  background: $color-surface-0;
}

.preview-placeholder,
.empty-preview,
.empty-library {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  color: $color-text-muted;
  text-align: center;
}

.selected-details {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  padding: $space-md $space-lg;
  border-top: 1px solid $color-border-muted;
}

.skin-properties,
.selected-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: $space-sm;
}

.property {
  padding: $space-1 $space-sm;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  color: $color-text-muted;
  font-size: 0.72rem;
}

.muted-message {
  color: $color-text-muted;
  font-size: 0.8rem;
}

.library-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.library-header h2 {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.library-count {
  color: $color-text-muted;
  font-size: 0.8rem;
}

.skin-list {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: $space-xs;
  min-height: 0;
  overflow-y: auto;
  padding: $space-sm;
}

.skin-list-item {
  display: flex;
  align-items: center;
  gap: $space-md;
  width: 100%;
  min-width: 0;
  padding: $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text;
  text-align: left;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $color-hover;
  }

  &.selected {
    border-color: $color-accent;
    background: $color-selected;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: -2px;
  }

  &:disabled {
    opacity: 0.6;
    cursor: wait;
  }
}

.skin-thumbnail {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 64px;
  width: 64px;
  height: 76px;
  overflow: hidden;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-0;
  color: $color-text-muted;
}

.skin-list-details {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: $space-xs;
  min-width: 0;
}

.skin-list-name {
  overflow: hidden;
  font-size: 0.82rem;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skin-list-meta,
.list-active-label {
  font-size: 0.72rem;
}

.skin-list-meta {
  color: $color-text-muted;
}

.list-active-label {
  color: $color-success;
}

.empty-preview,
.empty-library {
  flex: 1;
  padding: $space-2xl;

  h2,
  p {
    margin: 0;
  }

  h2 {
    color: $color-text;
    font-size: 1rem;
  }

  p {
    font-size: 0.85rem;
  }
}

.icon-button,
.action-button,
.secondary-button,
.file-picker {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  min-height: 32px;
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-accent;
    background: $color-hover;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.icon-button {
  width: 32px;
  padding: $space-sm;
  border-color: transparent;
}

.danger-button:hover:not(:disabled) {
  border-color: $color-error;
  background: $color-error-muted;
  color: $color-error;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: $layout-z-modal;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: $space-lg;
  background: $color-backdrop;
}

.modal {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow: hidden;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-1;
  box-shadow: 0 16px 48px $color-overlay;
}

.modal-header,
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  padding: $space-md $space-lg;
}

.modal-header {
  border-bottom: 1px solid $color-border-muted;

  h2 {
    margin: 0;
    font-size: 1rem;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  overflow-y: auto;
  padding: $space-lg;
}

.field {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
  color: $color-text;
  font-size: 0.85rem;

  > span:first-child,
  legend {
    font-weight: 600;
  }

  input[type="text"],
  select {
    width: 100%;
    min-height: 38px;
    padding: $space-sm $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-0;
    color: $color-text;
    font: inherit;

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: 1px;
    }
  }
}

.field-hint {
  color: $color-text-muted;
  font-size: 0.75rem;
  font-weight: 400;
}

.file-picker {
  min-height: 40px;
  border-style: dashed;
}

.selected-file {
  display: flex;
  align-items: center;
  gap: $space-sm;
  overflow-wrap: anywhere;
  color: $color-success;
  font-size: 0.75rem;
}

.radio-group {
  display: flex;
  gap: $space-md;

  label {
    display: flex;
    align-items: center;
    gap: $space-sm;
    flex: 1;
    padding: $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-0;
    cursor: pointer;

    &:has(input:checked) {
      border-color: $color-accent;
      background: $color-selected;
    }
  }

  input {
    accent-color: $color-accent;
  }
}

.modal-footer {
  justify-content: flex-end;
  border-top: 1px solid $color-border-muted;
}

@media (max-width: 900px) {
  .skins-workspace {
    grid-template-columns: minmax(0, 1fr) minmax(230px, 0.8fr);
    gap: $space-md;
    padding: $space-md;
  }
}

@media (max-width: 680px) {
  .skins-workspace {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(320px, 1fr) minmax(220px, 0.8fr);
    overflow-y: auto;
  }

  .selected-panel {
    min-height: 320px;
  }

  .library-panel {
    min-height: 220px;
  }
}
</style>
