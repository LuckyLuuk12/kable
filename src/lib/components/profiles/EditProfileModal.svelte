<script lang="ts">
import { app, Icon, type KableProfile } from "$lib";
import { clickSound, successSound } from "$lib/actions";
import { MODAL_CONTEXT, type ModalContext } from "$lib/utils/modal";
import { getContext, onMount } from "svelte";
import RamAllocation from "./RamAllocation.svelte";

const modal = getContext<ModalContext>(MODAL_CONTEXT);

let { profileProp }: { profileProp: KableProfile } = $props();

// svelte-ignore state_referenced_locally
let profile = $state(app.profilesService.memoryClone(profileProp));

let javaArgsString = $state(profile.settings.java_args?.join(" ") ?? "");
let parametersJson = $state(JSON.stringify(profile.settings.parameters_map ?? {}, null, 2));
let launchWithAccountSelection = $state(profile.settings.launch_with_account ?? "");

let isSaving = $state(false);
let copiedJson = $state(false);
let error = $state<string | null>(null);

onMount(() => {
  void app.authService.init().catch((e: unknown) => {
    error = e instanceof Error ? e.message : "Failed to load authenticated accounts.";
  });
});

let hasSelectedAccount = $derived(launchWithAccountSelection !== "" && app.authService.accounts.some((account) => account.local_id === launchWithAccountSelection));

let jsonPreviewProfile = $derived.by(() => {
  const preview = app.profilesService.memoryClone(profile);

  preview.settings.java_args = javaArgsString.split(" ").filter((arg) => arg.length > 0);
  preview.settings.launch_with_account = launchWithAccountSelection || null;

  try {
    const parsed = JSON.parse(parametersJson || "{}");

    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      preview.settings.parameters_map = parsed;
    }
  } catch {
    // Preserve the last valid parameters while the JSON editor is invalid.
  }

  return preview;
});

let jsonPreview = $derived(JSON.stringify(jsonPreviewProfile, null, 2));
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
      profile.metadata.icon = reader.result;
      error = null;
    }
  };

  reader.onerror = () => {
    error = "Failed to read the selected icon.";
  };

  reader.readAsDataURL(file);
}

async function copyJson() {
  try {
    await navigator.clipboard.writeText(jsonPreview);
    copiedJson = true;
    error = null;
  } catch {
    error = "Could not copy JSON. Check clipboard permissions.";
  }
}

async function confirmEdit() {
  error = null;

  profile.settings.java_args = javaArgsString.split(" ").filter((arg) => arg.length > 0);
  profile.settings.launch_with_account = launchWithAccountSelection || null;

  try {
    const parsed = JSON.parse(parametersJson || "{}");

    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
      error = "Parameters must contain a JSON object.";
      return;
    }

    profile.settings.parameters_map = parsed;
  } catch {
    error = "Parameters must contain valid JSON.";
    return;
  }

  isSaving = true;

  try {
    await app.profilesService.modify(profile, profile);
    app.dismissTop();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to update profile.";
  } finally {
    isSaving = false;
  }
}

function cancelEdit() {
  modal.dismiss();
}
</script>

<div class="edit-profile-modal">
  <div class="modal-header">
    <h2>Edit profile</h2>
  </div>

  {#if error}
    <div class="error-message" role="alert">
      {error}
    </div>
  {/if}

  <form
    class="profile-grid"
    onsubmit={(e) => {
      e.preventDefault();
      void confirmEdit();
    }}>
    <section class="basic-settings panel">
      <div class="profile-identity">
        <label class="icon-picker" title="Choose profile icon">
          {#if profile.metadata.icon}
            <img src={profile.metadata.icon} alt="Profile icon" />
          {:else}
            <span class="empty-icon">
              <Icon name="image" forceType="svg" size="md" />
            </span>
          {/if}

          <input type="file" accept="image/*" aria-label="Choose profile icon" onchange={handleIconFile} />
        </label>

        <input class="profile-name-input" type="text" bind:value={profile.metadata.name} placeholder="Profile name" aria-label="Profile name" />

        <button
          type="button"
          class="favorite-button"
          class:favorite-active={profile.metadata.favorite}
          onclick={() => {
            profile.metadata.favorite = !profile.metadata.favorite;
          }}
          aria-label={profile.metadata.favorite ? "Remove from favorites" : "Add to favorites"}
          title={profile.metadata.favorite ? "Remove from favorites" : "Add to favorites"}>
          {#if profile.metadata.favorite}
            <Icon name="star" size="md" />
          {:else}
            <Icon name="star" forceType="svg" size="md" />
          {/if}
        </button>
      </div>

      <label class="description-field">
        <span class="label-text">Description</span>
        <textarea bind:value={profile.metadata.description} placeholder="Add a description..." rows="4"></textarea>
      </label>
    </section>

    <section class="optional-settings panel">
      <div class="optional-heading">
        <span class="label-text">Launch configuration</span>
      </div>

      <div class="optional-content">
        <label class="form-field">
          <span class="label-text">Launch with account</span>

          <select bind:value={launchWithAccountSelection} disabled={app.authService.loading || isSaving}>
            <option value="">Active Account</option>

            {#if launchWithAccountSelection && !hasSelectedAccount}
              <option value={launchWithAccountSelection}>
                Unavailable account ({launchWithAccountSelection})
              </option>
            {/if}

            {#each app.authService.accounts as account (account.local_id)}
              <option value={account.local_id}>
                {app.authService.getAccountDisplayName(account)}
                {#if account.local_id === app.authService.activeAccount?.local_id}
                  — Active
                {/if}
              </option>
            {/each}
          </select>

          {#if app.authService.loading}
            <span class="field-description">Loading accounts…</span>
          {:else if app.authService.accounts.length === 0}
            <span class="field-description"> No authenticated accounts. Sign in to assign one. </span>
          {/if}
        </label>

        <label class="form-field">
          <span class="label-text">Java arguments</span>

          <textarea class="java-args-input" bind:value={javaArgsString} placeholder="-Xmx4G -XX:+UseG1GC" rows="4" spellcheck="false"></textarea>
        </label>

        <label class="pack-merging-option">
          <input type="checkbox" bind:checked={profile.settings.enable_pack_merging} />

          <span class="pack-merging-text">
            <span class="pack-merging-title">Enable pack merging</span>
            <span class="pack-merging-description"> Merge compatible resource packs when launching this profile. </span>
          </span>
        </label>

        <label class="form-field advanced-field">
          <span class="label-text">Parameters (JSON)</span>
          <span class="field-description"> Custom key-value parameters passed to the profile. </span>

          <textarea class="parameters-input" bind:value={parametersJson} rows="8" spellcheck="false"></textarea>
        </label>
      </div>
    </section>

    <section class="ram-allocation-section">
      <RamAllocation
        {profile}
        embedded
        saveToService={false}
        onJavaArgsChange={(args) => {
          profile.settings.java_args = args;
          javaArgsString = args.join(" ");
        }} />
    </section>

    <section class="preview-column panel">
      <div class="preview-toolbar">
        <span class="label-text">Profile JSON</span>

        <button
          type="button"
          class="copy-json-button"
          use:clickSound
          onclick={() => void copyJson()}
          title="Copy the current JSON preview"
          aria-label="Copy profile JSON">
          <Icon name={copiedJson ? "check" : "copy"} forceType="svg" size="sm" />
          <span>{copiedJson ? "Copied" : "Copy JSON"}</span>
        </button>
      </div>

      <!-- TODO: Add proper escaping -->
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <pre class="json-preview"><code>{@html highlightedJson}</code></pre>
    </section>

    <div class="actions">
      <button use:successSound type="submit" class="btn btn-primary" disabled={isSaving}>
        {isSaving ? "Saving..." : "Confirm"}
      </button>

      <button use:clickSound type="button" class="btn btn-secondary" onclick={cancelEdit} disabled={isSaving}> Cancel </button>
    </div>
  </form>
</div>

<style lang="scss">
.edit-profile-modal {
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  width: min(94vw, 1500px);
  height: min(92vh, 1000px);
  max-height: 92vh;
  min-height: 0;
  padding: 1.25rem;
  overflow: hidden;

  color: $color-text;
  background: $color-surface-1;
  border-radius: $radius-md;
  box-shadow: 0 0.5rem 2rem rgba(0, 0, 0, 0.3);

  .modal-header {
    flex: 0 0 auto;
    padding-bottom: 0.9rem;
    margin-bottom: 1rem;
    border-bottom: 1px solid color-mix(in srgb, $color-text-muted 22%, transparent);

    h2 {
      margin: 0;
      font-size: 1.2rem;
      font-weight: 650;
    }
  }

  .error-message {
    flex: 0 0 auto;
    padding: 0.7rem 0.9rem;
    margin-bottom: 0.85rem;
    border: 1px solid color-mix(in srgb, $color-error 35%, transparent);
    border-radius: $radius-md;
    background: color-mix(in srgb, $color-error 10%, transparent);
    color: $color-error;
    font-size: 0.85rem;
  }

  .profile-grid {
    display: grid;
    flex: 1 1 0;
    grid-template-columns:
      minmax(0, 0.9fr)
      minmax(0, 1fr)
      minmax(0, 1.15fr);
    grid-template-areas:
      "basic optional preview"
      "ram ram preview"
      "actions actions actions";
    grid-template-rows: minmax(0, 1fr) auto auto;
    align-items: stretch;
    gap: 1rem;

    min-width: 0;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
  }

  .panel {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    min-width: 0;
    min-height: 0;
    padding: 1rem;
    border: 1px solid color-mix(in srgb, $color-text-muted 18%, transparent);
    border-radius: $radius-md;
    background: color-mix(in srgb, $color-surface-2 35%, transparent);
  }

  .basic-settings {
    grid-area: basic;
    gap: 1rem;
    min-height: 0;
    overflow: hidden;
  }

  .profile-identity {
    display: grid;
    flex: 0 0 auto;
    grid-template-columns: 2.25rem minmax(0, 1fr) 2.25rem;
    align-items: center;
    gap: 0.55rem;
    min-width: 0;
  }

  .icon-picker {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 2.25rem;
    box-sizing: border-box;
    overflow: hidden;
    border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;
    cursor: pointer;

    img {
      display: block;
      width: 100%;
      height: 100%;
      object-fit: contain;
    }

    .empty-icon {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 100%;
      color: $color-text-muted;
    }

    input[type="file"] {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      opacity: 0;
      cursor: pointer;
    }

    &:hover {
      border-color: color-mix(in srgb, $color-text-muted 55%, transparent);
    }

    &:focus-within {
      outline: 2px solid color-mix(in srgb, $color-text 45%, transparent);
      outline-offset: 2px;
    }
  }

  .profile-name-input {
    width: 100%;
    min-width: 0;
    height: 2.25rem;
    box-sizing: border-box;
    padding: 0.4rem 0.6rem;
    border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;
    color: $color-text;
    font: inherit;
    font-size: 0.85rem;
    font-weight: 600;

    &:focus {
      outline: none;
      border-color: color-mix(in srgb, $color-text 55%, transparent);
    }
  }

  .label-text {
    color: $color-text;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .description-field,
  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
    max-width: 100%;
    font-size: 0.85rem;
  }

  .description-field {
    flex: 1 1 auto;
    min-height: 0;

    textarea {
      flex: 1 1 auto;
      min-height: 0;
      height: auto;
    }
  }

  input[type="text"],
  textarea,
  select {
    display: block;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 0.6rem 0.7rem;
    border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;
    color: $color-text;
    font: inherit;
    outline: none;

    &:hover:not(:disabled) {
      border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
    }

    &:focus {
      border-color: color-mix(in srgb, $color-text 55%, transparent);
      box-shadow: 0 0 0 2px color-mix(in srgb, $color-text 10%, transparent);
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    &::placeholder {
      color: $color-text-muted;
      opacity: 0.7;
    }
  }

  textarea {
    min-height: 0;
    resize: none;
    overflow: auto;
    line-height: 1.45;
  }

  .favorite-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 2.25rem;
    box-sizing: border-box;
    padding: 0;
    border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;
    color: $color-text-muted;
    cursor: pointer;

    &:hover {
      border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
      background: $color-surface-2;
    }

    &.favorite-active {
      color: $color-warning;
      border-color: color-mix(in srgb, $color-warning 40%, transparent);
      background: color-mix(in srgb, $color-warning 10%, transparent);
    }
  }

  .optional-settings {
    grid-area: optional;
    align-self: stretch;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .optional-heading,
  .preview-toolbar {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    min-width: 0;
  }

  .optional-heading {
    padding-bottom: 0.75rem;
    border-bottom: 1px solid color-mix(in srgb, $color-text-muted 15%, transparent);
  }

  .optional-content {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
    min-height: 0;
    padding-top: 0.85rem;
    overflow-x: hidden;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
  }

  .field-description {
    color: $color-text-muted;
    font-size: 0.75rem;
    line-height: 1.4;
  }

  .java-args-input {
    height: 6.5rem;
    min-height: 6.5rem;
    max-height: 6.5rem;
    flex: 0 0 auto;
    font-family: "Cascadia Code", "Fira Code", monospace;
    font-size: 0.8rem;
  }

  .parameters-input {
    height: 12rem;
    min-height: 12rem;
    max-height: 12rem;
    flex: 0 0 auto;
    font-family: "Cascadia Code", "Fira Code", monospace;
    font-size: 0.8rem;
  }

  .advanced-field {
    flex: 0 0 auto;
    min-height: 0;
  }

  .pack-merging-option {
    display: flex;
    flex: 0 0 auto;
    flex-direction: row;
    align-items: flex-start;
    gap: $space-sm;
    min-width: 0;
    cursor: pointer;

    input[type="checkbox"] {
      flex: 0 0 auto;
      width: $space-lg;
      height: $space-2xl;
      margin: 3px 0 0;
      accent-color: $color-accent;
    }
  }

  .pack-merging-text {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    min-width: 0;
  }

  .pack-merging-title {
    color: $color-text;
  }

  .pack-merging-description {
    color: $color-text-muted;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .ram-allocation-section {
    grid-area: ram;
    align-self: start;
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
  }

  .preview-column {
    grid-area: preview;
    align-self: stretch;
    min-width: 0;
    min-height: 0;
    height: 100%;
    max-height: 100%;
    overflow: hidden;
  }

  .copy-json-button {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    min-height: 2rem;
    padding: 0.35rem 0.6rem;
    border: 1px solid color-mix(in srgb, $color-text-muted 25%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;
    color: $color-text;
    font-size: 0.75rem;
    cursor: pointer;

    &:hover {
      border-color: color-mix(in srgb, $color-text-muted 45%, transparent);
      background: $color-surface-2;
    }
  }

  .json-preview {
    display: block;
    flex: 1 1 0;
    width: 100%;
    min-width: 0;
    min-height: 0;
    height: 0;
    margin: 0.75rem 0 0;
    padding: 0.8rem;
    box-sizing: border-box;

    overflow: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
    border: 1px solid color-mix(in srgb, $color-text-muted 18%, transparent);
    border-radius: $radius-md;
    background: $color-surface-1;

    font-family: "Cascadia Code", "Fira Code", monospace;
    font-size: 0.74rem;
    line-height: 1.55;
    white-space: pre;
    tab-size: 2;

    code {
      display: block;
      width: max-content;
      min-width: 100%;
    }

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
    grid-area: actions;
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding-top: 0.85rem;
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

@media (max-width: 1100px) {
  .edit-profile-modal {
    height: min(92vh, 1000px);
    width: 94vw;

    .profile-grid {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      grid-template-areas:
        "basic optional"
        "preview preview"
        "ram ram"
        "actions actions";
      grid-template-rows: auto auto auto auto;
    }

    .basic-settings {
      min-height: 16rem;
    }

    .optional-settings {
      min-height: 0;
      max-height: 28rem;
    }

    .preview-column {
      height: auto;
      max-height: none;
      min-height: 24rem;
    }

    .json-preview {
      flex: 1 1 0;
      height: 0;
      min-height: 14rem;
    }
  }
}

@media (max-width: 700px) {
  .edit-profile-modal {
    width: 95vw;
    height: 94vh;
    max-height: 94vh;
    padding: 0.85rem;

    .profile-grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas:
        "basic"
        "optional"
        "ram"
        "preview"
        "actions";
      grid-template-rows: auto minmax(18rem, auto) auto minmax(22rem, auto) auto;
      gap: 0.75rem;
    }

    .basic-settings {
      min-height: 14rem;
    }

    .description-field textarea {
      min-height: 7rem;
    }

    .optional-settings {
      min-height: 18rem;
      max-height: 30rem;
    }

    .java-args-input {
      height: 6rem;
      min-height: 6rem;
      max-height: 6rem;
    }

    .parameters-input {
      height: 10rem;
      min-height: 10rem;
      max-height: 10rem;
    }

    .preview-column {
      height: auto;
      max-height: none;
      min-height: 22rem;
    }

    .json-preview {
      flex: 1 1 0;
      height: 0;
      min-height: 12rem;
    }

    .actions {
      flex-direction: column-reverse;

      button {
        width: 100%;
      }
    }
  }
}
</style>
