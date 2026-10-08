<script lang="ts">
import { app, Icon, type KableProfile } from "$lib";
import { clickSound, successSound } from "$lib/actions";
import { MODAL_CONTEXT, type ModalContext } from "$lib/utils/modal";
import { getContext } from "svelte";

const modal = getContext<ModalContext>(MODAL_CONTEXT);

let {
  profile,
}: {
  profile: KableProfile;
} = $props();

// svelte-ignore state_referenced_locally
let installation = $state(app.profilesService.memoryClone(profile));
let javaArgsString = $state(installation.settings.java_args?.join(" ") ?? "");
let parametersJson = $state(JSON.stringify(installation.settings.parameters_map ?? {}, null, 2));
let showOptional = $state(false);
let isSaving = $state(false);
let error = $state<string | null>(null);

let jsonPreviewInstallation = $derived.by(() => {
  const preview = app.profilesService.memoryClone(installation);

  preview.settings.java_args = javaArgsString.split(" ").filter((arg) => arg.length > 0);

  try {
    const parsed = JSON.parse(parametersJson || "{}");

    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      preview.settings.parameters_map = parsed;
    }
  } catch {
    preview.settings.parameters_map = {
      ...preview.settings.parameters_map,
    };
  }

  return preview;
});

let jsonPreview = $derived(JSON.stringify(jsonPreviewInstallation, null, 2));

let highlightedJson = $derived(highlightJson(jsonPreview));

function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
}

function highlightJson(json: string): string {
  const tokenRegex = /"(?:\\.|[^"\\])*"(?=\s*:)|"(?:\\.|[^"\\])*"|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|\btrue\b|\bfalse\b|\bnull\b/g;

  let result = "";
  let lastIndex = 0;

  for (const match of json.matchAll(tokenRegex)) {
    const index = match.index ?? 0;

    result += escapeHtml(json.slice(lastIndex, index));

    const token = match[0];
    const escapedToken = escapeHtml(token);

    if (/^"(?:\\.|[^"\\])*"(?=\s*:)$/.test(token)) {
      result += `<span class="json-key">${escapedToken}</span>`;
    } else if (token.startsWith('"')) {
      result += `<span class="json-string">${escapedToken}</span>`;
    } else if (token === "true" || token === "false") {
      result += `<span class="json-boolean">${token}</span>`;
    } else if (token === "null") {
      result += `<span class="json-null">${token}</span>`;
    } else {
      result += `<span class="json-number">${token}</span>`;
    }

    lastIndex = index + token.length;
  }

  result += escapeHtml(json.slice(lastIndex));

  return result;
}

function handleIconFile(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  const file = input.files?.[0];

  if (!file) return;

  if (!file.type.startsWith("image/")) {
    error = "Please select an image file.";
    input.value = "";
    return;
  }

  const reader = new FileReader();

  reader.onload = () => {
    if (typeof reader.result === "string") {
      installation.metadata.icon = reader.result;
      error = null;
    }
  };

  reader.onerror = () => {
    error = "Failed to read the selected icon.";
  };

  reader.readAsDataURL(file);
}

async function confirmEdit() {
  error = null;

  installation.settings.java_args = javaArgsString.split(" ").filter((arg) => arg.length > 0);

  try {
    const parsed = JSON.parse(parametersJson || "{}");

    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      installation.settings.parameters_map = parsed;
    } else {
      error = "Parameters must contain a JSON object.";
      return;
    }
  } catch {
    error = "Parameters must contain valid JSON.";
    return;
  }

  isSaving = true;

  try {
    await app.profilesService.modify(installation, installation);
    modal.resolve(true);
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to update installation.";
  } finally {
    isSaving = false;
  }
}

function cancelEdit() {
  modal.dismiss();
}
</script>

<div class="edit-installation-modal">
  <div class="modal-header">
    <div>
      <h2>Edit Installation</h2>

      {#if installation.metadata.name}
        <div class="profile-name">{installation.metadata.name}</div>
      {/if}
    </div>
  </div>

  <div class="header-divider"></div>

  {#if error}
    <div class="error-message">
      {error}
    </div>
  {/if}

  <form
    class="three-column-form"
    onsubmit={(e) => {
      e.preventDefault();
      confirmEdit();
    }}>
    <section class="settings-column">
      <div class="section-header">
        <div>
          <h3>Basic settings</h3>
          <p>Identity and appearance of this installation.</p>
        </div>
      </div>

      <div class="section-divider"></div>

      <div class="section-content">
        <label>
          <span class="label-text">Name</span>
          <input type="text" bind:value={installation.metadata.name} placeholder="Installation name" />
        </label>

        <label>
          <span class="label-text">Icon</span>

          <div class="icon-input">
            {#if installation.metadata.icon}
              <div class="icon-preview">
                <img src={installation.metadata.icon} alt="Installation icon preview" />
              </div>
            {/if}

            <div class="icon-input-controls">
              <input type="text" bind:value={installation.metadata.icon} placeholder="Icon URL or data URI" />

              <label class="file-button">
                <span>Select</span>
                <input type="file" accept="image/*" onchange={handleIconFile} />
              </label>
            </div>
          </div>
        </label>

        <label>
          <span class="label-text">Description</span>
          <textarea bind:value={installation.metadata.description} placeholder="Optional description" rows="5"></textarea>
        </label>

        <div class="favorite-setting">
          <span class="label-text">Favorite</span>

          <button
            type="button"
            class:favorite-active={installation.metadata.favorite}
            class="favorite-button"
            onclick={() => {
              installation.metadata.favorite = !installation.metadata.favorite;
            }}
            aria-label={installation.metadata.favorite ? "Remove from favorites" : "Add to favorites"}
            title={installation.metadata.favorite ? "Remove from favorites" : "Add to favorites"}>
            {#if installation.metadata.favorite}
              <Icon name="star" size="md" />
            {:else}
              <Icon name="star" forceType="svg" size="md" />
            {/if}
          </button>
        </div>
      </div>
    </section>

    <section class="settings-column">
      <div class="section-header">
        <div>
          <h3>Optional settings</h3>
          <p>Advanced launch configuration for this installation.</p>
        </div>

        <button
          type="button"
          class="collapse-button"
          class:expanded={showOptional}
          onclick={() => {
            showOptional = !showOptional;
          }}
          aria-label={showOptional ? "Collapse settings" : "Expand settings"}>
          <Icon name="chevron-down" forceType="svg" size="sm" />
        </button>
      </div>

      <div class="section-divider"></div>

      {#if showOptional}
        <div class="section-content optional-content">
          <label>
            <span class="label-text">Java arguments</span>
            <input type="text" bind:value={javaArgsString} placeholder="e.g. -Xmx4G -XX:+UseG1GC" />
          </label>

          <label>
            <span class="label-text">Parameters</span>
            <span class="field-description"> JSON object passed to the installation. </span>

            <textarea class="parameters-input" bind:value={parametersJson} rows="10" spellcheck="false"></textarea>
          </label>
        </div>
      {:else}
        <div class="collapsed-message">Optional launch settings are hidden.</div>
      {/if}
    </section>

    <section class="preview-column">
      <div class="section-header">
        <div>
          <h3>Profile JSON</h3>
          <p>Live read-only representation of the profile.</p>
        </div>

        <span class="readonly-label">Read-only</span>
      </div>

      <div class="section-divider"></div>

      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <pre class="json-preview"><code>{@html highlightedJson}</code></pre>
    </section>

    <div class="actions">
      <button use:successSound type="submit" class="btn btn-primary" disabled={isSaving}>
        {#if isSaving}
          Saving...
        {:else}
          Confirm
        {/if}
      </button>

      <button use:clickSound type="button" class="btn btn-secondary" onclick={cancelEdit} disabled={isSaving}> Cancel </button>
    </div>
  </form>
</div>

<style lang="scss">
.edit-installation-modal {
  padding: 1.5rem;
  background: $color-surface-1;
  border-radius: $radius-md;
  width: 80vw;
  max-width: 1400px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 0.5rem 2rem rgba(0, 0, 0, 0.3);
  color: $color-text;

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 3rem;

    h2 {
      margin: 0;
      color: $color-text;
      font-size: 1.4rem;
      font-weight: 650;
    }

    .profile-name {
      margin-top: 0.25rem;
      color: $color-text-muted;
      font-size: 0.9rem;
    }
  }

  .header-divider {
    height: 1px;
    margin: 1rem 0 1.25rem;
    background: color-mix(in srgb, $color-text-muted 25%, transparent);
  }

  .error-message {
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border: 1px solid color-mix(in srgb, $color-error 35%, transparent);
    border-radius: $radius-md;
    background: color-mix(in srgb, $color-error 10%, transparent);
    color: $color-error;
  }

  .three-column-form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1.2fr);
    gap: 1.25rem;
    align-items: start;

    .settings-column,
    .preview-column {
      min-width: 0;
      min-height: 0;
      padding: 1rem;
      border: 1px solid color-mix(in srgb, $color-text-muted 18%, transparent);
      border-radius: $radius-md;
      background: color-mix(in srgb, $color-surface-2 35%, transparent);
    }

    .section-header {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: 1rem;

      h3 {
        margin: 0;
        color: $color-text;
        font-size: 1rem;
        font-weight: 650;
      }

      p {
        margin: 0.3rem 0 0;
        color: $color-text-muted;
        font-size: 0.75rem;
        line-height: 1.4;
      }
    }

    .section-divider {
      height: 1px;
      margin: 0.85rem 0 1rem;
      background: color-mix(in srgb, $color-text-muted 15%, transparent);
    }

    .section-content {
      display: flex;
      flex-direction: column;
      gap: 1rem;
      min-width: 0;
    }

    label {
      display: flex;
      flex-direction: column;
      gap: 0.4rem;
      min-width: 0;
      color: $color-text;
      font-size: 0.9rem;
    }

    .label-text {
      color: $color-text;
      font-size: 0.85rem;
      font-weight: 550;
    }

    .field-description {
      color: $color-text-muted;
      font-size: 0.75rem;
      line-height: 1.35;
    }

    input[type="text"],
    textarea {
      display: block;
      width: 100%;
      min-width: 0;
      box-sizing: border-box;
      border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
      border-radius: $radius-md;
      background: $color-surface-1;
      color: $color-text;
      font: inherit;
      outline: none;
      transition:
        border-color 120ms ease,
        box-shadow 120ms ease,
        background 120ms ease;

      &:hover {
        border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
      }

      &:focus {
        border-color: color-mix(in srgb, $color-text 55%, transparent);
        box-shadow: 0 0 0 2px color-mix(in srgb, $color-text 10%, transparent);
      }

      &::placeholder {
        color: $color-text-muted;
        opacity: 0.7;
      }
    }

    input[type="text"] {
      height: 2.4rem;
      padding: 0.5rem 0.7rem;
    }

    textarea {
      resize: vertical;
      padding: 0.65rem 0.7rem;
      line-height: 1.45;
    }

    .parameters-input {
      min-height: 12rem;
      resize: vertical;
      font-family: "Cascadia Code", "Fira Code", monospace;
      font-size: 0.8rem;
    }

    .icon-input {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      min-width: 0;
    }

    .icon-preview {
      width: 3rem;
      height: 3rem;
      overflow: hidden;
      border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
      border-radius: $radius-md;
      background: $color-surface-1;

      img {
        width: 100%;
        height: 100%;
        display: block;
        object-fit: contain;
      }
    }

    .icon-input-controls {
      display: flex;
      gap: 0.5rem;
      min-width: 0;

      > input {
        flex: 1;
        min-width: 0;
      }
    }

    .file-button {
      position: relative;
      display: inline-flex;
      flex: 0 0 auto;
      align-items: center;
      justify-content: center;
      min-width: 5rem;
      padding: 0 0.8rem;
      border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
      border-radius: $radius-md;
      background: $color-surface-1;
      color: $color-text;
      cursor: pointer;
      font-size: 0.8rem;
      transition:
        background 120ms ease,
        border-color 120ms ease;

      &:hover {
        border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
        background: $color-surface-2;
      }

      input[type="file"] {
        position: absolute;
        width: 1px;
        height: 1px;
        overflow: hidden;
        opacity: 0;
        pointer-events: none;
      }
    }

    .favorite-setting {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      min-height: 2.5rem;
    }

    .favorite-button {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 2.25rem;
      height: 2.25rem;
      padding: 0;
      border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
      border-radius: $radius-md;
      background: $color-surface-1;
      color: $color-text-muted;
      cursor: pointer;
      transition:
        color 120ms ease,
        background 120ms ease,
        border-color 120ms ease,
        transform 120ms ease;

      &:hover {
        color: $color-text;
        border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
        background: $color-surface-2;
      }

      &:active {
        transform: scale(0.94);
      }

      &.favorite-active {
        color: $color-warning;
        border-color: color-mix(in srgb, $color-warning 40%, transparent);
        background: color-mix(in srgb, $color-warning 10%, transparent);
      }
    }

    .collapse-button {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 2rem;
      height: 2rem;
      padding: 0;
      border: none;
      border-radius: $radius-md;
      background: transparent;
      color: $color-text-muted;
      cursor: pointer;
      transition:
        color 120ms ease,
        background 120ms ease,
        transform 120ms ease;

      &:hover {
        background: $color-surface-2;
        color: $color-text;
      }

      &.expanded {
        transform: rotate(180deg);
      }
    }

    .collapsed-message {
      padding: 0.5rem 0;
      color: $color-text-muted;
      font-size: 0.8rem;
      font-style: italic;
    }

    .readonly-label {
      flex: 0 0 auto;
      padding: 0.2rem 0.45rem;
      border: 1px solid color-mix(in srgb, $color-text-muted 20%, transparent);
      border-radius: 999px;
      color: $color-text-muted;
      font-size: 0.65rem;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .json-preview {
      width: 100%;
      min-width: 0;
      height: 34rem;
      margin: 0;
      padding: 0.85rem;
      box-sizing: border-box;
      overflow: auto;
      border: 1px solid color-mix(in srgb, $color-text-muted 18%, transparent);
      border-radius: $radius-md;
      background: $color-surface-1;

      font-family: "Cascadia Code", "Fira Code", monospace;
      font-size: 0.76rem;
      line-height: 1.55;
      white-space: pre;
      tab-size: 2;

      :global(.json-key) {
        color: #8ab4f8;
      }

      :global(.json-string) {
        color: #a8d08d;
      }

      :global(.json-number) {
        color: #f6c177;
      }

      :global(.json-boolean) {
        color: #d19a66;
      }

      :global(.json-null) {
        color: #c586c0;
      }
    }

    .actions {
      grid-column: 1 / -1;
      display: flex;
      justify-content: flex-end;
      gap: 0.75rem;
      padding-top: 0.25rem;
      border-top: 1px solid color-mix(in srgb, $color-text-muted 18%, transparent);

      button {
        min-width: 6.5rem;
        height: 2.4rem;
        padding: 0.45rem 1.25rem;
        border: 1px solid transparent;
        border-radius: $radius-md;
        font-size: 0.9rem;
        font-weight: 550;
        cursor: pointer;
        transition:
          background 120ms ease,
          border-color 120ms ease,
          opacity 120ms ease,
          transform 120ms ease;

        &:active:not(:disabled) {
          transform: translateY(1px);
        }

        &:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      }

      .btn-primary {
        border-color: color-mix(in srgb, $color-success 70%, transparent);
        background: color-mix(in srgb, $color-success 85%, transparent);
        color: $color-text;

        &:hover:not(:disabled) {
          background: $color-success;
        }
      }

      .btn-secondary {
        border-color: color-mix(in srgb, $color-error 70%, transparent);
        background: color-mix(in srgb, $color-error 12%, transparent);
        color: $color-error;

        &:hover:not(:disabled) {
          background: color-mix(in srgb, $color-error 20%, transparent);
        }
      }
    }
  }
}

@media (max-width: 1100px) {
  .edit-installation-modal {
    width: 90vw;

    .three-column-form {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);

      .preview-column {
        grid-column: 1 / -1;
      }

      .json-preview {
        height: 30rem;
      }
    }
  }
}

@media (max-width: 700px) {
  .edit-installation-modal {
    width: 95vw;
    padding: 1rem;

    .three-column-form {
      grid-template-columns: 1fr;

      .preview-column {
        grid-column: auto;
      }

      .json-preview {
        height: 24rem;
      }

      .icon-input-controls {
        flex-direction: column;

        .file-button {
          min-height: 2.4rem;
        }
      }

      .actions {
        flex-direction: column-reverse;

        button {
          width: 100%;
        }
      }
    }
  }
}
</style>
