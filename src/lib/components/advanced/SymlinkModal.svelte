<script lang="ts">
import { Icon, type Symlink, type SymlinkCreateRequest } from "$lib";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  open: boolean;
  symlink?: Symlink | null;
  busy?: boolean;
  onSubmit: (request: SymlinkCreateRequest, existing?: Symlink) => Promise<void> | void;
  onClose: () => void;
}

let { open: isOpen, symlink = null, busy = false, onSubmit, onClose }: Props = $props();

let source = $state("");
let destination = $state("");
let validationError = $state<string | null>(null);

let isEditing = $derived(symlink !== null);

$effect(() => {
  if (!isOpen) return;

  source = symlink?.source ?? "";
  destination = symlink?.destination ?? "";
  validationError = null;
});

async function pickSourceFile(): Promise<void> {
  try {
    const path = await open({
      directory: false,
      multiple: false,
      title: "Select Source File",
    });

    if (typeof path !== "string") return;

    source = path;
    validationError = null;
  } catch (error) {
    console.error("Failed to select source file:", error);
    validationError = "Failed to select source file.";
  }
}

async function pickSourceFolder(): Promise<void> {
  try {
    const path = await open({
      directory: true,
      multiple: false,
      title: "Select Source Folder",
    });

    if (typeof path !== "string") return;

    source = path;
    validationError = null;
  } catch (error) {
    console.error("Failed to select source folder:", error);
    validationError = "Failed to select source folder.";
  }
}

async function pickDestinationFolder(): Promise<void> {
  try {
    const path = await open({
      directory: true,
      multiple: false,
      title: "Select Destination Folder",
    });

    if (typeof path !== "string") return;

    destination = path;
    validationError = null;
  } catch (error) {
    console.error("Failed to select destination folder:", error);
    validationError = "Failed to select destination folder.";
  }
}

async function submit(): Promise<void> {
  const normalizedSource = source.trim();
  const normalizedDestination = destination.trim();

  if (!normalizedSource || !normalizedDestination) {
    validationError = "Both source and destination paths are required.";
    return;
  }

  if (normalizedSource === normalizedDestination) {
    validationError = "Source and destination paths must be different.";
    return;
  }

  validationError = null;

  await onSubmit(
    {
      source: normalizedSource,
      destination: normalizedDestination,
    },
    symlink ?? undefined,
  );
}
</script>

{#if isOpen}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget && !busy) onClose();
    }}
    onkeydown={(event) => {
      if (event.key === "Escape" && !busy) onClose();
    }}>
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section class="modal" role="dialog" aria-modal="true" aria-labelledby="symlink-modal-title">
      <header class="modal-header">
        <div>
          <h2 id="symlink-modal-title">
            {isEditing ? "Edit symlink" : "Create symlink"}
          </h2>
          <p>
            {isEditing ? "Update the source and destination paths." : "Configure the source and destination paths for the symlink."}
          </p>
        </div>

        <button class="close-button" type="button" aria-label="Close" disabled={busy} onclick={onClose}> × </button>
      </header>

      <form
        onsubmit={(event) => {
          event.preventDefault();
          void submit();
        }}>
        <div class="form-field">
          <label for="symlink-source">Source path</label>

          <div class="path-input">
            <input
              id="symlink-source"
              type="text"
              bind:value={source}
              placeholder="Absolute path to the source"
              autocomplete="off"
              spellcheck="false"
              required
              disabled={busy} />
            <button class="picker-button" type="button" title="Select source file" aria-label="Select source file" disabled={busy} onclick={() => void pickSourceFile()}>
              <Icon name="file" forceType="svg" />
            </button>
            <button
              class="picker-button"
              type="button"
              title="Select source folder"
              aria-label="Select source folder"
              disabled={busy}
              onclick={() => void pickSourceFolder()}>
              <Icon name="folder" forceType="svg" />
            </button>
          </div>

          <p class="field-hint">The existing file or directory to link to. Use the folder picker or enter a file path manually.</p>
        </div>

        <div class="form-field">
          <label for="symlink-destination">Destination path</label>

          <div class="path-input">
            <input
              id="symlink-destination"
              type="text"
              bind:value={destination}
              placeholder="Absolute path for the symlink"
              autocomplete="off"
              spellcheck="false"
              required
              disabled={busy} />

            <button
              class="picker-button"
              type="button"
              title="Select destination folder"
              aria-label="Select destination folder"
              disabled={busy}
              onclick={() => void pickDestinationFolder()}>
              <Icon name="folder" forceType="svg" />
            </button>
          </div>

          <p class="field-hint">
            The full path where the symlink will be created. After selecting a folder, edit the path if the symlink should have a different name or location.
          </p>
        </div>

        {#if validationError}
          <p class="validation-error" role="alert">{validationError}</p>
        {/if}

        <footer class="modal-actions">
          <button class="secondary-button" type="button" disabled={busy} onclick={onClose}> Cancel </button>

          <button class="primary-button" type="submit" disabled={busy || !source.trim() || !destination.trim()}>
            {busy ? "Saving…" : isEditing ? "Save changes" : "Create symlink"}
          </button>
        </footer>
      </form>
    </section>
  </div>
{/if}

<style lang="scss">
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: $space-lg;
  background: rgb(0 0 0 / 60%);
}

.modal {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  width: 100%;
  max-width: 560px;
  max-height: calc(100vh - 2 * $space-lg);
  overflow-y: auto;
  padding: $space-lg;
  border: 1px solid $color-border;
  border-radius: 8px;
  background: $color-background;
  color: $color-text;
  box-shadow: 0 16px 48px rgb(0 0 0 / 30%);
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: $space-md;

  h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  p {
    margin: $space-xs 0 0;
    color: $color-text-muted;
    font-size: 0.85rem;
  }
}

.close-button {
  display: grid;
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  place-items: center;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: $color-text-muted;
  font-size: 1.5rem;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $color-hover;
    color: $color-text;
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

form {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: $space-xs;

  label {
    color: $color-text;
    font-size: 0.875rem;
    font-weight: 500;
  }
}

.path-input {
  display: flex;
  align-items: stretch;
  gap: $space-xs;
  min-width: 0;

  input {
    flex: 1;
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: $space-sm $space-md;
    border: 1px solid $color-border;
    border-radius: 4px;
    outline: none;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;
    font-size: 0.875rem;

    &:focus {
      border-color: $color-focus;
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }
}

.picker-button {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 40px;
  border: 1px solid $color-border;
  border-radius: 4px;
  background: $color-surface-2;
  color: $color-text-muted;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:hover:not(:disabled) {
    border-color: $color-accent;
    background: $color-hover;
    color: $color-accent;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

.field-hint {
  margin: 0;
  color: $color-text-muted;
  font-size: 0.8rem;
  line-height: 1.4;
}

.validation-error {
  margin: 0;
  color: $color-error;
  font-size: 0.875rem;
  overflow-wrap: anywhere;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
  padding-top: $space-sm;
  border-top: 1px solid $color-border;
}

.primary-button,
.secondary-button {
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: 4px;
  font: inherit;
  font-size: 0.875rem;
  cursor: pointer;

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.primary-button {
  border-color: $color-accent;
  background: $color-accent;
  color: white;
}

.secondary-button {
  background: $color-surface-2;
  color: $color-text;

  &:hover:not(:disabled) {
    background: $color-hover;
  }
}
</style>
