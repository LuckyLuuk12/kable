<!-- @component
UpdateModal - Modal for application updates

Displays update information and provides options to install, download, or dismiss.
Respects user's update mode preference.

@prop {any | null} [updateInfo=null] - Update information
@prop {boolean} [open=false] - Whether modal is open
@prop {(() => void) | undefined} onclose - Callback when modal is closed
@prop {(() => void) | undefined} oninstallnow - Callback to install update immediately
@prop {(() => void) | undefined} ondownloadandrestart - Callback to download and install on restart
@prop {(() => void) | undefined} ondownload - Callback to download update for later

@example
```svelte
<UpdateModal 
  {updateInfo} 
  bind:open 
  onclose={handleClose} 
  oninstallnow={handleInstallNow}
  ondownloadandrestart={handleDownloadRestart}
  ondownload={handleDownload}
/>
```
-->
<script lang="ts">
import { Icon } from "$lib";
import { clickSound, successSound } from "$lib/actions";
import { marked } from "marked";

export let updateInfo: any = null;
export let open = false;
export let onclose: (() => void) | undefined = undefined;
export let oninstallnow: (() => void) | undefined = undefined;
export let ondownload: (() => void) | undefined = undefined;

let releaseNotesHtml = "";
let isProcessing = false;

$: if (updateInfo?.body && open) {
  const parseMarkdown = async () => {
    releaseNotesHtml = await marked.parse(updateInfo.body, {
      breaks: true,
      gfm: true,
    });
  };
  parseMarkdown();
}

function handleClose() {
  if (isProcessing) return;
  open = false;
  onclose?.();
}

async function handleInstallNow() {
  if (isProcessing) return;
  isProcessing = true;
  try {
    await oninstallnow?.();
  } finally {
    isProcessing = false;
  }
}

async function handleDownload() {
  if (isProcessing) return;
  isProcessing = true;
  try {
    await ondownload?.();
    handleClose();
  } finally {
    isProcessing = false;
  }
}

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget && !isProcessing) {
    handleClose();
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && !isProcessing) {
    handleClose();
  }
}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open && updateInfo}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop" on:click={handleBackdropClick}>
    <div
      class="modal-content"
      on:click|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-modal-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="modal-header">
        <div class="header-content">
          <div class="update-icon">
            <Icon name="arrow-up" size="lg" forceType="svg" />
          </div>
          <div class="header-text">
            <h2 id="update-modal-title">Update Available</h2>
            <p class="version-info">
              Version <strong>v{updateInfo.version}</strong> is now available
              {#if updateInfo.current_version}
                <span class="current-version"
                  >(Current: v{updateInfo.current_version})</span
                >
              {/if}
            </p>
          </div>
        </div>
        <button
          class="close-btn"
          on:click={handleClose}
          use:clickSound
          disabled={isProcessing}
          aria-label="Close modal"
        >
          <Icon name="x" size="md" forceType="svg" />
        </button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        {#if releaseNotesHtml}
          <div class="release-notes">
            <h3>What's New</h3>
            <div class="notes-content markdown-content">
              {@html releaseNotesHtml}
            </div>
          </div>
        {:else}
          <p class="no-notes">
            A new version is available. Click below to update.
          </p>
        {/if}
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button
          class="btn-secondary"
          on:click={handleClose}
          use:clickSound
          disabled={isProcessing}
        >
          Skip
        </button>

        <div class="action-buttons">
          {#if ondownload}
            <button
              class="btn-download"
              on:click={handleDownload}
              use:clickSound
              disabled={isProcessing}
              title="Download update for later installation"
            >
              <Icon name="download" size="sm" forceType="svg" />
              Download & Install on Restart
            </button>
          {/if}

          {#if oninstallnow}
            <button
              class="btn-primary"
              on:click={handleInstallNow}
              use:successSound
              disabled={isProcessing}
              title="Install update and restart the application now"
            >
              <Icon name="check" size="sm" forceType="svg" />
              {isProcessing ? "Installing..." : "Install Now"}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 1rem;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-content {
  background: var(--card);
  border-radius: 0.75rem;
  border: 2px solid var(--primary);
  box-shadow:
    0 20px 60px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.05);
  max-width: 600px;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid color-mix(in srgb, var(--primary), 15%, transparent);
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--primary), 8%, transparent) 0%,
    color-mix(in srgb, var(--secondary), 5%, transparent) 100%
  );

  .header-content {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    flex: 1;

    .update-icon {
      flex-shrink: 0;
      width: 48px;
      height: 48px;
      background: var(--primary);
      border-radius: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
      box-shadow: 0 4px 12px
        color-mix(in srgb, var(--primary), 30%, transparent);
    }

    .header-text {
      flex: 1;
      min-width: 0;

      h2 {
        margin: 0 0 0.25rem 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: var(--text);
      }

      .version-info {
        margin: 0;
        font-size: 0.875rem;
        color: var(--text-muted);

        strong {
          color: var(--primary);
          font-weight: 600;
        }

        .current-version {
          opacity: 0.7;
          font-size: 0.8125rem;
        }
      }
    }
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--placeholder);
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.375rem;
    transition: all 0.2s ease;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--text), 10%, transparent);
      color: var(--text);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;

  .no-notes {
    margin: 0;
    color: var(--text-muted);
    text-align: center;
    padding: 2rem 0;
  }

  .release-notes {
    h3 {
      margin: 0 0 1rem 0;
      font-size: 1rem;
      font-weight: 600;
      color: var(--text);
    }

    .notes-content {
      background: var(--container);
      border-radius: 0.5rem;
      padding: 1rem;
      color: var(--text-muted);
      font-size: 0.875rem;
      line-height: 1.6;
      max-height: 300px;
      overflow-y: auto;
    }
  }
}

:global(.markdown-content p) {
  margin: 0.5rem 0;
}

:global(.markdown-content p:first-child) {
  margin-top: 0;
}

:global(.markdown-content p:last-child) {
  margin-bottom: 0;
}

:global(.markdown-content a) {
  color: var(--primary);
  text-decoration: underline;
  transition: opacity 0.15s;
}

:global(.markdown-content a:hover) {
  opacity: 0.8;
}

:global(.markdown-content strong) {
  color: var(--text);
  font-weight: 600;
}

:global(.markdown-content em) {
  font-style: italic;
}

:global(.markdown-content code) {
  background: var(--card);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-family: "Courier New", monospace;
  font-size: 0.85em;
  color: var(--primary);
}

:global(.markdown-content pre) {
  background: var(--card);
  border-radius: 0.375rem;
  padding: 0.75rem;
  overflow-x: auto;
  margin: 0.5rem 0;
}

:global(.markdown-content pre code) {
  background: none;
  padding: 0;
}

:global(.markdown-content h1),
:global(.markdown-content h2),
:global(.markdown-content h3),
:global(.markdown-content h4),
:global(.markdown-content h5),
:global(.markdown-content h6) {
  color: var(--text);
  margin: 1rem 0 0.5rem;
  font-weight: 600;
}

:global(.markdown-content h1:first-child),
:global(.markdown-content h2:first-child),
:global(.markdown-content h3:first-child),
:global(.markdown-content h4:first-child),
:global(.markdown-content h5:first-child),
:global(.markdown-content h6:first-child) {
  margin-top: 0;
}

:global(.markdown-content h1) {
  font-size: 1.5em;
}

:global(.markdown-content h2) {
  font-size: 1.3em;
}

:global(.markdown-content h3) {
  font-size: 1.1em;
}

:global(.markdown-content h4),
:global(.markdown-content h5),
:global(.markdown-content h6) {
  font-size: 1em;
}

:global(.markdown-content ul),
:global(.markdown-content ol) {
  margin: 0.5rem 0;
  padding-left: 1.5rem;
}

:global(.markdown-content ul li),
:global(.markdown-content ol li) {
  margin: 0.25rem 0;
}

:global(.markdown-content blockquote) {
  border-left: 3px solid var(--primary);
  padding-left: 1rem;
  margin: 0.5rem 0;
  color: var(--text-muted);
}

:global(.markdown-content hr) {
  border: none;
  border-top: 1px solid var(--dark-200);
  margin: 1rem 0;
}

:global(.markdown-content img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.375rem;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--dark-200);
  gap: 1rem;

  .action-buttons {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  button {
    padding: 0.625rem 1.25rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }

  .btn-secondary {
    background: transparent;
    border: 1px solid var(--dark-300);
    color: var(--text-muted);

    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--text), 5%, transparent);
      border-color: var(--dark-400);
      color: var(--text);
    }
  }

  .btn-download {
    background: color-mix(in srgb, var(--tertiary), 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--tertiary), 30%, transparent);
    color: var(--tertiary);

    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--tertiary), 25%, transparent);
      border-color: var(--tertiary);
    }
  }

  .btn-primary {
    background: var(--primary);
    color: white;

    &:hover:not(:disabled) {
      background: color-mix(in srgb, var(--primary), 90%, black);
      transform: translateY(-1px);
      box-shadow: 0 4px 12px
        color-mix(in srgb, var(--primary), 30%, transparent);
    }

    &:active:not(:disabled) {
      transform: translateY(0);
    }
  }
}

@media (max-width: 640px) {
  .modal-content {
    max-width: 100%;
    margin: 0.5rem;
  }

  .modal-header .header-content {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .modal-footer {
    flex-direction: column;
    align-items: stretch;

    .action-buttons {
      width: 100%;
      flex-direction: column;

      button {
        width: 100%;
        justify-content: center;
      }
    }

    .btn-secondary {
      order: 3;
    }
  }
}
</style>
