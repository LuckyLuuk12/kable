<script lang="ts">
import { app, error, Icon, type Symlink, type SymlinkCreateRequest } from "$lib";
import { onMount } from "svelte";
import SymlinkModal from "./SymlinkModal.svelte";

let symlinks = $derived(app.symlinkService.persistentSymlinks);

let copiedPath = $state<string | null>(null);
let copyTimeout: ReturnType<typeof setTimeout> | null = null;

onMount(() => {
  if (!app.symlinkService.initialized) {
    void app.symlinkService.init();
  }

  return () => {
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = null;
    }
  };
});

function openCreateModal(): void {
  app.show(SymlinkModal, {
    open: true,
    symlink: null,
    busy: app.symlinkService.modifying,
    onSubmit: saveSymlink,
    onClose: () => app.dismissTop(),
  });
}

function openEditModal(symlink: Symlink): void {
  app.show(SymlinkModal, {
    open: true,
    symlink,
    busy: app.symlinkService.modifying,
    onSubmit: saveSymlink,
    onClose: () => app.dismissTop(),
  });
}

async function saveSymlink(request: SymlinkCreateRequest, existing?: Symlink): Promise<void> {
  if (existing) {
    const updated: Symlink = {
      ...existing,
      source: request.source,
      destination: request.destination,
    };

    const result = await app.symlinkService.update(existing, updated);

    if (!result) return;
  } else {
    const result = await app.symlinkService.create(request);

    if (!result) return;
  }

  app.dismissTop();
}

async function toggleSymlink(symlink: Symlink): Promise<void> {
  await app.symlinkService.toggle(symlink);
}

async function removeSymlink(symlink: Symlink): Promise<void> {
  if (symlink.is_temporary || app.symlinkService.modifying) return;

  const confirmed = window.confirm(`Remove this symlink?\n\nSource: ${symlink.source}\nDestination: ${symlink.destination}`);

  if (!confirmed) return;

  await app.symlinkService.remove(symlink);
}

async function copyPath(path: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(path);
    copiedPath = path;

    if (copyTimeout) clearTimeout(copyTimeout);

    copyTimeout = setTimeout(() => {
      copiedPath = null;
      copyTimeout = null;
    }, 2000);
  } catch (err) {
    error("Failed to copy path:", err);
  }
}

function getActionDisabledReason(symlink: Symlink): string | null {
  if (symlink.is_temporary) {
    return "This temporary symlink cannot be modified here.";
  }

  if (app.symlinkService.modifying) {
    return "Another symlink operation is in progress.";
  }

  return null;
}

function getToggleTitle(symlink: Symlink): string {
  const reason = getActionDisabledReason(symlink);

  if (reason) return reason;

  return symlink.enabled ? "Disable symlink" : "Enable symlink";
}
</script>

<section class="symlink-manager">
  <header class="section-header">
    <div class="section-heading">
      <div class="heading-title">
        <h1>Symlink Manager</h1>

        {#if app.symlinkService.modifying}
          <span class="modifying-indicator" role="status">
            <Icon name="load" forceType="svg" size="sm" />
            Updating symlinks…
          </span>
        {/if}
      </div>

      <p>Manage custom symbolic links used by Kable.</p>
      <p class="hint">Runtime symlinks are managed automatically by the backend.</p>
    </div>

    <div class="header-actions">
      <button
        class="action-btn"
        type="button"
        onclick={() => app.symlinkService.refreshPersistent()}
        disabled={app.symlinkService.loading || app.symlinkService.modifying}
        title="Refresh symlinks"
        aria-label="Refresh symlinks">
        <Icon name="refresh" forceType="svg" size="sm" />
      </button>

      <button class="primary-btn" type="button" onclick={openCreateModal} disabled={app.symlinkService.modifying}>
        <Icon name="plus" forceType="svg" size="sm" />
        Create Symlink
      </button>
    </div>
  </header>

  {#if app.symlinkService.error}
    <div class="error-banner" role="alert">
      <Icon name="alert" forceType="svg" size="sm" />
      <span>{app.symlinkService.error}</span>
    </div>
  {/if}

  {#if app.symlinkService.loading}
    <div class="state-panel">
      <Icon name="load" forceType="svg" />
      <p>Loading symlinks...</p>
    </div>
  {:else if symlinks.length === 0}
    <div class="state-panel">
      <Icon name="link" size="xl" />
      <h2>No custom symlinks</h2>
      <p>Create a symlink to link a source file or directory to a destination path.</p>

      <button class="primary-btn" type="button" onclick={openCreateModal} disabled={app.symlinkService.modifying}>
        <Icon name="plus" forceType="svg" size="sm" />
        Create Symlink
      </button>
    </div>
  {:else}
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Status</th>
            <th>Source</th>
            <th>Destination</th>
            <th>Origin</th>
            <th>Actions</th>
          </tr>
        </thead>

        <tbody>
          {#each symlinks as symlink (symlink.id)}
            {@const disabledReason = getActionDisabledReason(symlink)}

            <tr class:symlink-disabled={!symlink.enabled}>
              <td>
                <span class="status" class:active={symlink.enabled} class:disabled={!symlink.enabled}>
                  {symlink.enabled ? "Enabled" : "Disabled"}
                </span>
              </td>

              <td class="path-cell">
                <button class="path-btn" type="button" title={`Copy source path: ${symlink.source}`} onclick={() => copyPath(symlink.source)}>
                  <span>{symlink.source}</span>
                  <Icon name="copy" forceType="svg" size="sm" />
                </button>
              </td>

              <td class="path-cell">
                <button class="path-btn" type="button" title={`Copy destination path: ${symlink.destination}`} onclick={() => copyPath(symlink.destination)}>
                  <span>{symlink.destination}</span>
                  <Icon name="copy" forceType="svg" size="sm" />
                </button>
              </td>

              <td>
                <div class="origin-labels">
                  <span class="origin" class:launcher={symlink.from_launcher}>
                    {symlink.from_launcher ? "Kable" : "Custom"}
                  </span>

                  {#if symlink.is_temporary}
                    <span class="temporary-label" title="Temporary symlinks are managed automatically and cannot be modified here."> Temporary </span>
                  {/if}

                  {#if app.symlinkService.modifying}
                    <span class="action-status" title="A symlink operation is in progress."> Busy </span>
                  {/if}
                </div>
              </td>

              <td>
                <div class="actions">
                  <button
                    class="icon-btn"
                    type="button"
                    onclick={() => openEditModal(symlink)}
                    title={disabledReason ?? "Edit symlink"}
                    aria-label="Edit symlink"
                    disabled={disabledReason !== null}>
                    <Icon name="edit" forceType="svg" />
                  </button>

                  <button
                    class="icon-btn"
                    type="button"
                    onclick={() => toggleSymlink(symlink)}
                    title={getToggleTitle(symlink)}
                    aria-label={symlink.enabled ? "Disable symlink" : "Enable symlink"}
                    disabled={disabledReason !== null}>
                    <Icon name={symlink.enabled ? "eye-off" : "eye"} forceType="svg" />
                  </button>

                  <button
                    class="icon-btn danger"
                    type="button"
                    onclick={() => removeSymlink(symlink)}
                    title={disabledReason ?? "Remove symlink"}
                    aria-label="Remove symlink"
                    disabled={disabledReason !== null}>
                    <Icon name="delete" forceType="svg" />
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

{#if copiedPath}
  <div class="copy-notification" role="status">
    <Icon name="check" forceType="svg" size="sm" />
    Path copied to clipboard
  </div>
{/if}

<style lang="scss">
.symlink-manager {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  min-width: 0;
  color: $color-text;
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: $space-lg;
  padding: $space-lg;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
}

.section-heading {
  min-width: 0;

  p {
    margin: 0;
    color: $color-text-muted;
    font-size: 0.85rem;
  }

  .hint {
    margin-top: $space-xs;
    font-size: 0.75rem;
  }
}

.heading-title {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: $space-md;
  margin-bottom: $space-xs;

  h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }
}

.modifying-indicator {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  color: $color-text-muted;
  font-size: 0.75rem;
  font-weight: 500;
}

.header-actions,
.actions,
.origin-labels {
  display: flex;
  align-items: center;
  gap: $space-sm;
}

.header-actions {
  flex-shrink: 0;
}

.primary-btn,
.action-btn,
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  font: inherit;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
}

.primary-btn {
  padding: $space-sm $space-md;
  background: $color-accent;
  color: white;
  font-size: 0.85rem;
  font-weight: 600;

  &:hover:not(:disabled) {
    filter: brightness(1.08);
  }
}

.action-btn,
.icon-btn {
  padding: $space-sm;
  background: transparent;
  color: $color-text-muted;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-hover;
    color: $color-text;
  }
}

.icon-btn {
  flex: 0 0 auto;

  &.danger {
    color: $color-error;
  }
}

.error-banner {
  display: flex;
  align-items: center;
  gap: $space-sm;
  padding: $space-md;
  border: 1px solid color-mix(in srgb, $color-error, transparent 65%);
  border-radius: $radius-md;
  background: color-mix(in srgb, $color-error, transparent 92%);
  color: $color-error;
  font-size: 0.85rem;
  overflow-wrap: anywhere;
}

.state-panel {
  display: flex;
  min-height: 220px;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  padding: $space-xl;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;
  color: $color-text-muted;
  text-align: center;

  p,
  h2 {
    margin: 0;
  }

  h2 {
    color: $color-text;
    font-size: 1rem;
  }

  .primary-btn {
    margin-top: $space-sm;
  }
}

.table-wrapper {
  max-width: 100%;
  overflow: auto;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-1;

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  th {
    padding: $space-sm $space-md;
    border-bottom: 1px solid $color-border;
    background: $color-surface-2;
    color: $color-text-muted;
    font-size: 0.7rem;
    font-weight: 600;
    text-align: left;
    text-transform: uppercase;
    white-space: nowrap;
  }

  td {
    padding: $space-sm $space-md;
    border-bottom: 1px solid $color-border-muted;
    vertical-align: middle;
  }

  tbody tr:last-child td {
    border-bottom: 0;
  }

  tbody tr:hover {
    background: $color-hover;
  }

  tbody tr.symlink-disabled {
    opacity: 0.75;
  }
}

.path-cell {
  min-width: 180px;
  max-width: 360px;
  font-family: monospace;
}

.path-btn {
  display: flex;
  align-items: center;
  gap: $space-sm;
  width: 100%;
  overflow: hidden;
  padding: $space-xs;
  border: 0;
  border-radius: $radius-sm;
  background: transparent;
  color: $color-text;
  font: inherit;
  text-align: left;
  cursor: copy;

  span {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &:hover {
    background: $color-hover;
    color: $color-accent;
  }
}

.status,
.origin,
.temporary-label,
.action-status {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: $radius-sm;
  font-size: 0.7rem;
  font-weight: 600;
  white-space: nowrap;
}

.status.active {
  background: color-mix(in srgb, $color-success, transparent 85%);
  color: $color-success;
}

.status.disabled {
  background: color-mix(in srgb, $color-warning, transparent 85%);
  color: $color-warning;
}

.origin {
  background: $color-surface-2;
  color: $color-text-muted;

  &.launcher {
    background: color-mix(in srgb, $color-accent, transparent 85%);
    color: $color-accent;
  }
}

.temporary-label {
  background: color-mix(in srgb, $color-warning, transparent 85%);
  color: $color-warning;
  cursor: help;
}

.action-status {
  background: $color-surface-2;
  color: $color-text-muted;
}

.copy-notification {
  position: fixed;
  right: $space-xl;
  bottom: $space-xl;
  z-index: 1001;
  display: flex;
  align-items: center;
  gap: $space-sm;
  padding: $space-md $space-lg;
  border: 1px solid color-mix(in srgb, $color-success, transparent 65%);
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-success;
  box-shadow: 0 4px 16px rgb(0 0 0 / 25%);
}

@media (max-width: 900px) {
  .section-header {
    flex-direction: column;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .table-wrapper table {
    min-width: 760px;
  }
}
</style>
