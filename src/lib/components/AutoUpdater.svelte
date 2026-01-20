<!-- @component
AutoUpdater - Handles checking for and installing application updates

Automatically checks for updates on mount and provides UI for manual update checks.
Displays update information including version numbers and allows users to install updates.

@example
```svelte
◄AutoUpdater /►
```
-->
<script lang="ts">
import { onMount } from "svelte";
import {
  checkForUpdates,
  installUpdate,
  getCurrentVersion,
  downloadUpdate,
  applyDownloadedUpdate,
} from "$lib";
import { marked } from "marked";
import { settings } from "$lib/stores";
import { clickSound, successSound } from "$lib/actions";

let currentVersion = "";
let updateInfo: any = null;
let isChecking = false;
let isInstalling = false;
let isDownloading = false;
let isApplying = false;
let error = "";
let releaseNotesHtml = "";
let downloadedPath: string | null = null;

onMount(async () => {
  try {
    currentVersion = await getCurrentVersion();
  } catch (e) {
    console.error("Failed to get current version:", e);
  }
});

async function handleCheckForUpdates() {
  isChecking = true;
  error = "";

  try {
    // Respect the user's nightly update preference
    const checkNightly = $settings?.advanced?.check_nightly_updates ?? false;
    console.log(
      "[AutoUpdater] Checking for updates with checkNightly:",
      checkNightly,
    );
    console.log("[AutoUpdater] Full settings.advanced:", $settings?.advanced);
    updateInfo = await checkForUpdates(checkNightly);

    if (updateInfo?.body) {
      releaseNotesHtml = await marked.parse(updateInfo.body, {
        breaks: true,
        gfm: true,
      });
    }
  } catch (e) {
    error = `Failed to check for updates: ${e}`;
  } finally {
    isChecking = false;
  }
}

async function handleInstallUpdate() {
  if (!updateInfo) return;

  isInstalling = true;
  error = "";

  try {
    const checkNightly = $settings?.advanced?.check_nightly_updates ?? false;
    await installUpdate(checkNightly);
    // App will restart automatically after update
  } catch (e) {
    error = `Failed to install update: ${e}`;
    isInstalling = false;
  }
}

async function handleDownloadUpdate() {
  if (!updateInfo) return;
  isDownloading = true;
  error = "";
  try {
    const checkNightly = $settings?.advanced?.check_nightly_updates ?? false;
    downloadedPath = await downloadUpdate(checkNightly);
  } catch (e) {
    error = `Failed to download update: ${e}`;
  } finally {
    isDownloading = false;
  }
}

async function handleInstallDownloaded() {
  isApplying = true;
  error = "";
  try {
    await applyDownloadedUpdate();
  } catch (e) {
    error = `Failed to apply downloaded update: ${e}`;
    isApplying = false;
  }
}
</script>

<div class="updater-section">
  <div class="section-header">
    <h3>Auto-Update</h3>
    <p>Current version: <span class="version">{currentVersion}</span></p>
  </div>

  <div class="update-controls">
    <button
      class="check-button"
      on:click={handleCheckForUpdates}
      use:clickSound
      disabled={isChecking || isInstalling}
    >
      {#if isChecking}
        Checking...
      {:else}
        Check for Updates
      {/if}
    </button>

    {#if updateInfo}
      <div class="update-available">
        <h4>Update Available: v{updateInfo.version}</h4>
        {#if releaseNotesHtml}
          <div class="update-notes">
            <p><strong>Release Notes:</strong></p>
            <div class="notes-content markdown-content">
              {@html releaseNotesHtml}
            </div>
          </div>
        {/if}
        <div class="update-actions">
          <button
            class="download-button"
            on:click={handleDownloadUpdate}
            use:clickSound
            disabled={isDownloading || isInstalling}
            title="Download installer now; will be applied on restart or when you click 'Install downloaded'"
          >
            {#if isDownloading}
              Downloading...
            {:else}
              Download Update
            {/if}
            <span class="small-version">v{updateInfo.version}</span>
          </button>

          <button
            class="install-button"
            on:click={handleInstallUpdate}
            use:successSound
            disabled={isInstalling}
            title="App will restart to complete installation"
          >
            {#if isInstalling}
              Installing...
            {:else}
              Install Now
            {/if}
            <span class="small-version">v{updateInfo.version}</span>
          </button>

          <button
            class="install-downloaded-button"
            on:click={handleInstallDownloaded}
            use:successSound
            disabled={!downloadedPath || isApplying}
            title="Install the previously downloaded update and restart the app"
          >
            {#if isApplying}
              Installing...
            {:else}
              Install Downloaded
            {/if}
            {#if downloadedPath}
              <span class="small-version">({downloadedPath})</span>
            {/if}
          </button>
        </div>
      </div>
    {:else if !isChecking && currentVersion}
      <p class="up-to-date">You're running the latest version</p>
    {/if}

    {#if error}
      <div class="error">
        {error}
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
.updater-section {
  background: var(--card);
  border-radius: 0.5rem;
  padding: 1.5rem;
  margin-bottom: 1rem;
}

.section-header {
  margin-bottom: 1rem;

  h3 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
    font-size: 1.125rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .version {
    color: var(--primary);
    font-weight: 500;
  }
}

.update-controls {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.check-button {
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 0.375rem;
  padding: 0.75rem 1.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  align-self: flex-start;

  &:hover:not(:disabled) {
    background: var(
      --primary-hover,
      #{"color-mix(in srgb, var(--primary) 90%, black)"}
    );
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

.update-available {
  background: var(--container);
  border: 2px solid var(--primary);
  border-radius: 0.5rem;
  padding: 1rem;

  h4 {
    margin: 0 0 0.75rem 0;
    color: var(--primary);
    font-size: 1rem;
    font-weight: 600;
  }
}

.update-notes {
  margin-bottom: 1rem;

  p {
    margin: 0 0 0.5rem 0;
    color: var(--text);
    font-size: 0.875rem;
    font-weight: 500;
  }

  .notes-content {
    background: var(--card);
    border-radius: 0.375rem;
    padding: 0.75rem;
    color: var(--text-muted);
    font-size: 0.875rem;
    line-height: 1.5;
    overflow-y: auto;

    &.markdown-content {
      :global(p) {
        margin: 0.5rem 0;

        &:first-child {
          margin-top: 0;
        }

        &:last-child {
          margin-bottom: 0;
        }
      }

      :global(a) {
        color: var(--primary);
        text-decoration: underline;
        transition: opacity 0.15s;

        &:hover {
          opacity: 0.8;
        }
      }

      :global(strong) {
        color: var(--text);
        font-weight: 600;
      }

      :global(em) {
        font-style: italic;
      }

      :global(code) {
        background: var(--container);
        padding: 0.125rem 0.375rem;
        border-radius: 0.25rem;
        font-family: "Courier New", monospace;
        font-size: 0.85em;
        color: var(--primary);
      }

      :global(pre) {
        background: var(--container);
        border-radius: 0.375rem;
        padding: 0.75rem;
        overflow-x: auto;
        margin: 0.5rem 0;
      }

      :global(pre code) {
        background: none;
        padding: 0;
        font-size: 0.875rem;
      }

      :global(h1),
      :global(h2),
      :global(h3),
      :global(h4),
      :global(h5),
      :global(h6) {
        color: var(--text);
        margin: 0.75rem 0 0.5rem 0;
        font-weight: 600;

        &:first-child {
          margin-top: 0;
        }
      }

      :global(h1) {
        font-size: 1.25rem;
      }

      :global(h2) {
        font-size: 1.1rem;
      }

      :global(h3) {
        font-size: 1rem;
      }

      :global(h4),
      :global(h5),
      :global(h6) {
        font-size: 0.95rem;
      }

      :global(ul),
      :global(ol) {
        margin: 0.5rem 0;
        padding-left: 1.5rem;

        &:first-child {
          margin-top: 0;
        }

        &:last-child {
          margin-bottom: 0;
        }
      }

      :global(li) {
        margin: 0.25rem 0;
      }

      :global(blockquote) {
        border-left: 3px solid var(--primary);
        padding-left: 0.75rem;
        margin: 0.5rem 0;
        color: var(--text-muted);
        font-style: italic;
      }

      :global(hr) {
        border: none;
        border-top: 1px solid var(--border);
        margin: 0.75rem 0;
      }

      :global(img) {
        max-width: 100%;
        height: auto;
        border-radius: 0.375rem;
        margin: 0.5rem 0;
      }

      :global(table) {
        width: 100%;
        border-collapse: collapse;
        margin: 0.5rem 0;
      }

      :global(table th),
      :global(table td) {
        padding: 0.5rem;
        border: 1px solid var(--border);
        text-align: left;
      }

      :global(table th) {
        background: var(--container);
        font-weight: 600;
      }
    }
  }
}

.update-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  flex-wrap: wrap;
}

.download-button {
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 0.375rem;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
}

.install-downloaded-button {
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 0.375rem;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
}

.small-version {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-left: 0.5rem;
}

.install-button {
  background: var(--status-success);
  color: var(--text-white);
  border: none;
  border-radius: 0.375rem;
  padding: 0.75rem 1.5rem;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    background: var(--status-success-hover);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

.up-to-date {
  color: var(--status-success);
  font-size: 0.875rem;
  font-weight: 500;
  margin: 0;
  padding: 0.5rem 0;
}

.error {
  background: var(--status-error-bg);
  border: 1px solid var(--status-error);
  border-radius: 0.375rem;
  padding: 0.75rem;
  color: var(--status-error);
  font-size: 0.875rem;
  line-height: 1.4;
}
</style>
