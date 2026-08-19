<script lang="ts">
import { app, type KableProfile } from "$lib";
import { clickSound, successSound } from "$lib/actions";
import { MODAL_CONTEXT, type ModalContext } from "$lib/utils/modal";
import { getContext } from "svelte";

const modal = getContext<ModalContext>(MODAL_CONTEXT);

let {
  profile,
}: {
  profile: KableProfile;
} = $props();

let installation = $state(app.profilesService.memoryClone(profile));
let javaArgsString = $state(installation.settings.java_args?.join(" ") ?? "");
let parametersJson = $state(JSON.stringify(installation.settings.parameters_map ?? {}, null, 2));
let showOptional = $state(false);
let isSaving = $state(false);
let error = $state<string | null>(null);

async function confirmEdit() {
  error = null;

  installation.settings.java_args = javaArgsString.split(" ").filter((arg) => arg.length > 0);

  try {
    const parsed = JSON.parse(parametersJson || "{}");

    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      installation.settings.parameters_map = parsed;
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
  <h2>
    Edit Installation{#if installation.metadata.name}
      - {installation.metadata.name}
    {/if}
  </h2>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <form
    class="two-column-form"
    onsubmit={(e) => {
      e.preventDefault();
      confirmEdit();
    }}>
    <div class="left-column">
      <label>
        Name:
        <input type="text" bind:value={installation.metadata.name} />
      </label>

      <label>
        Icon:
        <input type="text" bind:value={installation.metadata.icon} />
      </label>

      <label>
        Description (optional):
        <textarea bind:value={installation.metadata.description}></textarea>
      </label>

      <label class="favorite-row">
        <span>Favorite:</span>
        <input type="checkbox" bind:checked={installation.metadata.favorite} />
      </label>
    </div>

    <div class="right-column">
      <details bind:open={showOptional} class="optional-section">
        <summary>Optional settings</summary>

        <div class="optional-content">
          <label>
            Java Args:
            <input type="text" bind:value={javaArgsString} />
          </label>

          <label>
            Parameters (JSON object):
            <textarea bind:value={parametersJson} rows="6"></textarea>
          </label>
        </div>
      </details>
    </div>

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
  padding: 2rem;
  background: $color-surface-1;
  border-radius: $radius-md;
  width: 80vw;
  max-width: 900px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 0.5rem 2rem rgba(0, 0, 0, 0.3);

  h2 {
    margin-bottom: 1rem;
    color: var(--text);
  }

  .error-message {
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border-radius: $radius-md;
    background: color-mix(in srgb, $color-error, transparent 85%);
    color: $color-error;
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

    .left-column,
    .right-column {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .favorite-row {
      flex-direction: row;
      align-items: center;
      gap: 0.5rem;
    }

    .optional-section {
      summary {
        cursor: pointer;
        font-weight: 600;
        margin-bottom: 0.5rem;
        color: $color-text;
      }
    }

    .optional-content {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      margin-top: 1rem;
    }

    .actions {
      grid-column: 1 / -1;
      display: flex;
      gap: 1rem;
      justify-content: flex-end;

      button {
        padding: 0.5rem 1.5rem;
        border-radius: $radius-md;
        border: none;
        font-size: 1rem;
        cursor: pointer;

        &:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      }
    }
  }
}
</style>
