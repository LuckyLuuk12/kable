<script lang="ts">
import type {
  KableInstallation,
  ModpackContext,
  ModpackSelection,
  MrPackDetailed,
  PackFileInfo,
} from "$lib";
import * as modsApi from "$lib/api/mods";

export let open: boolean;
export let modpack: MrPackDetailed | null;
export let context: ModpackContext | null = null;
export let installation: KableInstallation | null = null;
export let onCancel: (() => void) | undefined = undefined;

type PackKind = "mods" | "resourcepacks" | "shaderpacks";
type GroupState = {
  toBeInstalled: PackFileInfo[];
  optional: PackFileInfo[];
  disabled: PackFileInfo[];
};

const emptyGroup = (): GroupState => ({
  toBeInstalled: [],
  optional: [],
  disabled: [],
});

let activeKind: PackKind = "mods";
let groups: Record<PackKind, GroupState> = {
  mods: emptyGroup(),
  resourcepacks: emptyGroup(),
  shaderpacks: emptyGroup(),
};

let installing = false;
let errorMsg: string | null = null;
let lastModpack: MrPackDetailed | null = null;
let installStatusText = "";

$: if (modpack) {
  // Reinitialize state when a new modpack payload arrives.
  if (modpack !== lastModpack) {
    groups = {
      mods: {
        toBeInstalled: modpack.mods.to_be_installed.map((f) => ({ ...f })),
        optional: modpack.mods.optional.map((f) => ({ ...f })),
        disabled: modpack.mods.disabled.map((f) => ({ ...f })),
      },
      resourcepacks: {
        toBeInstalled: modpack.resourcepacks.to_be_installed.map((f) => ({
          ...f,
        })),
        optional: modpack.resourcepacks.optional.map((f) => ({ ...f })),
        disabled: modpack.resourcepacks.disabled.map((f) => ({ ...f })),
      },
      shaderpacks: {
        toBeInstalled: modpack.shaderpacks.to_be_installed.map((f) => ({
          ...f,
        })),
        optional: modpack.shaderpacks.optional.map((f) => ({ ...f })),
        disabled: modpack.shaderpacks.disabled.map((f) => ({ ...f })),
      },
    };
    lastModpack = modpack;
    errorMsg = null;
  }
}

$: activeGroup = groups[activeKind];

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function getSelectedCount(kind: PackKind): number {
  const group = groups[kind];
  return group.toBeInstalled.length + group.optional.length;
}

function getTotalSelectedCount(): number {
  return (
    getSelectedCount("mods") +
    getSelectedCount("resourcepacks") +
    getSelectedCount("shaderpacks")
  );
}

function overwritePathsFor(group: GroupState): string[] {
  return [...group.toBeInstalled, ...group.optional]
    .filter((file) => file.overwrite)
    .map((file) => file.path);
}

function buildSelectionGroup(group: GroupState) {
  return {
    enabled: group.toBeInstalled.map((file) => file.path),
    optional: group.optional.map((file) => file.path),
    disabled: group.disabled.map((file) => file.path),
    overwrite_paths: overwritePathsFor(group),
  };
}

function moveFile(
  file: PackFileInfo,
  from: "toBeInstalled" | "optional" | "disabled",
  to: "toBeInstalled" | "optional" | "disabled",
  kind: PackKind,
) {
  const source = groups[kind];
  // Remove from current list
  const withoutToBeInstalled =
    from === "toBeInstalled"
      ? source.toBeInstalled.filter((f) => f.path !== file.path)
      : source.toBeInstalled;
  const withoutOptional =
    from === "optional"
      ? source.optional.filter((f) => f.path !== file.path)
      : source.optional;
  const withoutDisabled =
    from === "disabled"
      ? source.disabled.filter((f) => f.path !== file.path)
      : source.disabled;

  // Add to new list
  groups = {
    ...groups,
    [kind]: {
      toBeInstalled:
        to === "toBeInstalled"
          ? [...withoutToBeInstalled, file]
          : withoutToBeInstalled,
      optional:
        to === "optional" ? [...withoutOptional, file] : withoutOptional,
      disabled:
        to === "disabled" ? [...withoutDisabled, file] : withoutDisabled,
    },
  };
}

function toggleOverwrite(
  file: PackFileInfo,
  list: "toBeInstalled" | "optional",
  kind: PackKind,
) {
  const source = groups[kind];
  if (list === "toBeInstalled") {
    groups = {
      ...groups,
      [kind]: {
        ...source,
        toBeInstalled: source.toBeInstalled.map((f) =>
          f.path === file.path ? { ...f, overwrite: !f.overwrite } : f,
        ),
      },
    };
  } else {
    groups = {
      ...groups,
      [kind]: {
        ...source,
        optional: source.optional.map((f) =>
          f.path === file.path ? { ...f, overwrite: !f.overwrite } : f,
        ),
      },
    };
  }
}

async function handleConfirm() {
  if (!installation) {
    errorMsg = "No installation selected.";
    return;
  }
  if (!context) {
    errorMsg = "Missing modpack context from backend.";
    return;
  }
  installing = true;
  errorMsg = null;
  installStatusText = `Downloading and installing ${getTotalSelectedCount()} selected files...`;

  const selection: ModpackSelection = {
    mods: buildSelectionGroup(groups.mods),
    resourcepacks: buildSelectionGroup(groups.resourcepacks),
    shaderpacks: buildSelectionGroup(groups.shaderpacks),
  };

  let installSucceeded = false;
  try {
    await modsApi.applyModpackSelection(installation, selection, context);
    installSucceeded = true;
  } catch (e) {
    errorMsg =
      typeof e === "object" && e && "message" in e
        ? (e as any).message
        : String(e);
  } finally {
    installing = false;
    installStatusText = "";
  }

  if (installSucceeded) {
    handleClose();
  }
}

function handleClose() {
  if (installing) {
    return;
  }
  // Reset local state for next open
  groups = {
    mods: emptyGroup(),
    resourcepacks: emptyGroup(),
    shaderpacks: emptyGroup(),
  };
  lastModpack = null;
  activeKind = "mods";
  errorMsg = null;
  installStatusText = "";
  onCancel?.();
}
</script>

{#if open && modpack}
  <div class="packdiff-modal-backdrop" tabindex="-1">
    <div class="packdiff-modal-content">
      <div class="packdiff-modal-header">
        <button class="packdiff-close-btn" on:click={handleClose} title="Close"
          >×</button
        >
      </div>
      <div class="packdiff-modal-controls">
        <div class="kind-tabs">
          <button
            class="kind-tab"
            class:active={activeKind === "mods"}
            on:click={() => (activeKind = "mods")}
          >
            Mods ({getSelectedCount("mods")})
          </button>
          <button
            class="kind-tab"
            class:active={activeKind === "resourcepacks"}
            on:click={() => (activeKind = "resourcepacks")}
          >
            Resourcepacks ({getSelectedCount("resourcepacks")})
          </button>
          <button
            class="kind-tab"
            class:active={activeKind === "shaderpacks"}
            on:click={() => (activeKind = "shaderpacks")}
          >
            Shaderpacks ({getSelectedCount("shaderpacks")})
          </button>
        </div>
        <span class="file-count">Selected: {getTotalSelectedCount()}</span>
      </div>
      <div class="packdiff-modal-body">
        <div class="packdiff-columns">
          <div class="packdiff-column">
            <div class="packdiff-section-title">To Be Installed</div>
            <div class="packdiff-section-list">
              {#if activeGroup.toBeInstalled.length === 0}
                <div class="empty-state">No files to install.</div>
              {:else}
                {#each activeGroup.toBeInstalled as file (file.path)}
                  <div class="file-row">
                    <span class="file-path">{file.path}</span>
                    <span class="size">({formatFileSize(file.file_size)})</span>
                    {#if file.already_installed}
                      <label class="overwrite-label">
                        <input
                          type="checkbox"
                          checked={file.overwrite}
                          on:change={() =>
                            toggleOverwrite(file, "toBeInstalled", activeKind)}
                        />
                        <span class="conflict">overwrite</span>
                      </label>
                    {/if}
                    <button
                      class="arrow-btn"
                      title="Disable"
                      on:click={() =>
                        moveFile(file, "toBeInstalled", "disabled", activeKind)}
                      >→</button
                    >
                    <button
                      class="arrow-btn"
                      title="Make Optional"
                      on:click={() =>
                        moveFile(file, "toBeInstalled", "optional", activeKind)}
                      >↓</button
                    >
                  </div>
                {/each}
              {/if}
            </div>
          </div>
          <div class="packdiff-column">
            <div class="packdiff-section-title">Optional Files</div>
            <div class="packdiff-section-list">
              {#if activeGroup.optional.length === 0}
                <div class="empty-state">No optional files.</div>
              {:else}
                {#each activeGroup.optional as file (file.path)}
                  <div class="file-row">
                    <span class="file-path">{file.path}</span>
                    <span class="size">({formatFileSize(file.file_size)})</span>
                    {#if file.already_installed}
                      <label class="overwrite-label">
                        <input
                          type="checkbox"
                          checked={file.overwrite}
                          on:change={() =>
                            toggleOverwrite(file, "optional", activeKind)}
                        />
                        <span class="conflict">overwrite</span>
                      </label>
                    {/if}
                    <button
                      class="arrow-btn"
                      title="Disable"
                      on:click={() =>
                        moveFile(file, "optional", "disabled", activeKind)}
                      >→</button
                    >
                    <button
                      class="arrow-btn"
                      title="Make Required"
                      on:click={() =>
                        moveFile(file, "optional", "toBeInstalled", activeKind)}
                      >↑</button
                    >
                  </div>
                {/each}
              {/if}
            </div>
          </div>
          <div class="packdiff-column">
            <div class="packdiff-section-title">Disabled Files</div>
            <div class="packdiff-section-list">
              {#if activeGroup.disabled.length === 0}
                <div class="empty-state">No disabled files.</div>
              {:else}
                {#each activeGroup.disabled as file (file.path)}
                  <div class="file-row">
                    <span class="file-path">{file.path}</span>
                    <span class="size">({formatFileSize(file.file_size)})</span>
                    <span class="conflict">(disabled)</span>
                    <button
                      class="arrow-btn"
                      title="Enable"
                      on:click={() =>
                        moveFile(file, "disabled", "toBeInstalled", activeKind)}
                      >←</button
                    >
                    <button
                      class="arrow-btn"
                      title="Make Optional"
                      on:click={() =>
                        moveFile(file, "disabled", "optional", activeKind)}
                      >↑</button
                    >
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </div>
      <div class="packdiff-modal-footer">
        {#if installing}
          <div class="installing-state" role="status" aria-live="polite">
            <div class="progress-indicator"><span></span></div>
            <span>{installStatusText}</span>
          </div>
        {/if}
        <button class="cancel-btn" on:click={handleClose} disabled={installing}
          >Cancel</button
        >
        <button
          class="confirm-btn"
          on:click={handleConfirm}
          disabled={getTotalSelectedCount() === 0 || installing}
        >
          {installing ? "Installing..." : "Install Selected"}
        </button>
        {#if errorMsg}
          <span class="error-msg">{errorMsg}</span>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
.packdiff-modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.4);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}
.packdiff-modal-content {
  background: var(--container, #232323);
  border-radius: 12px;
  min-width: 700px;
  max-width: 98vw;
  min-height: 420px;
  max-height: 90vh;
  box-shadow: 0 2px 16px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}
.packdiff-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 2rem 0.7rem 2rem;
  background: var(--header, #181818);
  border-bottom: 1px solid #333;
}
.packdiff-close-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 1.7rem;
  cursor: pointer;
  transition: color 0.2s;
  padding: 0 0.5em;
}
.packdiff-close-btn:hover {
  color: #fff;
}
.packdiff-modal-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.7rem 2rem 0.7rem 2rem;
  background: var(--header, #181818);
  border-bottom: 1px solid #333;
  gap: 2rem;
}
.kind-tabs {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.kind-tab {
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #bfbfbf;
  cursor: pointer;
  font-size: 0.9rem;
  padding: 0.3rem 0.65rem;
  transition:
    background 0.15s,
    color 0.15s,
    border-color 0.15s;
}
.kind-tab:hover {
  background: #343434;
  color: #fff;
}
.kind-tab.active {
  background: #4caf50;
  border-color: #4caf50;
  color: #fff;
}
.file-count {
  color: #aaa;
  font-size: 1em;
}
.packdiff-modal-body {
  flex: 1 1 auto;
  padding: 0;
  background: var(--container, #232323);
  display: flex;
  flex-direction: column;
}
.packdiff-columns {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  height: 100%;
  min-height: 300px;
  padding: 1.2rem 2rem 1.2rem 2rem;
  box-sizing: border-box;
}
.packdiff-column {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--card, #222);
  border-radius: 8px;
  margin: 0 0.2rem;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}
.packdiff-section-title {
  font-size: 1.08rem;
  font-weight: 600;
  padding: 0.7rem 1rem 0.5rem 1rem;
  border-bottom: 1px solid #333;
  background: var(--header, #191919);
}
.packdiff-section-list {
  flex: 1 1 auto;
  overflow-y: auto;
  padding: 0.5rem 0.5rem 0.5rem 0.5rem;
  min-height: 120px;
  max-height: 45vh;
}
.file-row {
  display: flex;
  align-items: center;
  gap: 0.7em;
  padding: 0.35em 0.5em;
  border-radius: 5px;
  margin-bottom: 0.15em;
  transition: background 0.15s;
  background: none;
}
.file-row label {
  display: flex;
  align-items: center;
  gap: 0.7em;
  width: 100%;
  cursor: pointer;
}
.file-path {
  flex: 1 1 0;
  word-break: break-all;
}
.size {
  color: #888;
  font-size: 0.95em;
}
.conflict {
  color: #e6b800;
  font-size: 0.95em;
  margin-left: 0.5em;
}
.empty-state {
  color: #888;
  font-style: italic;
  margin: 0.5em 0 0.5em 1em;
}
.packdiff-modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 1.2rem;
  padding: 1rem 2rem 1.1rem 2rem;
  background: var(--header, #181818);
  border-top: 1px solid #333;
  position: sticky;
  bottom: 0;
  z-index: 2;
}
.installing-state {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-right: auto;
  color: #9fd0a1;
  font-size: 0.95rem;
}
.progress-indicator {
  width: 150px;
  height: 7px;
  border-radius: 999px;
  overflow: hidden;
  background: #2c3a2d;
  border: 1px solid #3a4f3b;
}
.progress-indicator span {
  display: block;
  height: 100%;
  width: 45%;
  background: linear-gradient(90deg, #4caf50, #79d27d);
  animation: modal-indeterminate 1.1s ease-in-out infinite;
}
@keyframes modal-indeterminate {
  0% {
    transform: translateX(-120%);
  }
  100% {
    transform: translateX(260%);
  }
}
.cancel-btn,
.confirm-btn {
  padding: 0.5em 1.5em;
  border-radius: 5px;
  border: none;
  font-size: 1.05em;
  cursor: pointer;
  transition:
    background 0.2s,
    color 0.2s;
}
.cancel-btn {
  background: #333;
  color: #aaa;
}
.cancel-btn:hover {
  background: #444;
  color: #fff;
}
.cancel-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.confirm-btn {
  background: #4caf50;
  color: #fff;
  font-weight: 600;
}
.confirm-btn:disabled {
  background: #2a2a2a;
  color: #888;
  cursor: not-allowed;
}
.overwrite-label {
  display: flex;
  align-items: center;
  gap: 0.3em;
  margin-left: 0.5em;
}
.arrow-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 1.2em;
  cursor: pointer;
  margin-left: 0.5em;
  transition: color 0.2s;
  padding: 0 0.3em;
}
.arrow-btn:hover {
  color: #4caf50;
}
</style>
