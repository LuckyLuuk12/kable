<script lang="ts">
import { app, type KableProfile, type KableProject, type Project, type ResolvedDependency } from "$lib";

interface Props {
  profile: KableProfile;
  project: Project;
  versionId: string;
  dependencies: ResolvedDependency[];
  onConfirm: (dependencies: KableProject[]) => Promise<void> | void;
  onCancel?: () => void;
}

let { profile, project, versionId, dependencies, onConfirm, onCancel }: Props = $props();

let selectedProjectIds = $state<Set<string>>(new Set());
let installing = $state(false);
let errorMessage = $state<string | null>(null);

function projectKey(dependency: ResolvedDependency): string {
  return `${dependency.project.project.project_id}@${dependency.project.version_id}`;
}

function dependencyKind(dependency: ResolvedDependency): string {
  return String(dependency.dependency_type)
    .replace(/([a-z])([A-Z])/g, "$1_$2")
    .toLowerCase();
}

function isRequired(dependency: ResolvedDependency): boolean {
  return dependencyKind(dependency) === "required";
}

function isOptional(dependency: ResolvedDependency): boolean {
  return dependencyKind(dependency) === "optional";
}

function displayDependencyType(dependency: ResolvedDependency): string {
  return dependencyKind(dependency)
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

interface SelectionState {
  active: Set<string>;
  required: Set<string>;
}

/**
 * Resolve effective selection across the entire tree.
 *
 * A dependency can occur under multiple parents. Its installation state
 * must therefore be shared across every occurrence of the same project
 * and version.
 *
 * Requiredness propagates only through active parent paths. The iteration
 * reaches a fixed point so shared dependencies work regardless of traversal
 * order.
 */
function calculateSelectionState(roots: ResolvedDependency[], manuallySelected: Set<string>): SelectionState {
  const active = new Set<string>();
  const required = new Set<string>();

  let changed = true;

  while (changed) {
    changed = false;

    function visit(nodes: ResolvedDependency[], parentActive: boolean): void {
      for (const dependency of nodes) {
        const key = projectKey(dependency);

        if (parentActive && isRequired(dependency) && !required.has(key)) {
          required.add(key);
          changed = true;
        }

        const shouldBeActive = active.has(key) || (parentActive && (isRequired(dependency) || manuallySelected.has(key) || required.has(key)));

        if (shouldBeActive && !active.has(key)) {
          active.add(key);
          changed = true;
        }

        // A shared dependency can be active through another branch, even
        // when this particular occurrence's parent is inactive.
        visit(dependency.dependencies ?? [], shouldBeActive);
      }
    }

    visit(roots, true);
  }

  return { active, required };
}

const selectionState = $derived.by(() => calculateSelectionState(dependencies, selectedProjectIds));

const selectedDependencies = $derived.by(() => {
  const unique = new Map<string, KableProject>();

  function visit(nodes: ResolvedDependency[]): void {
    for (const dependency of nodes) {
      const key = projectKey(dependency);

      if (selectionState.active.has(key)) {
        unique.set(key, dependency.project);
      }

      visit(dependency.dependencies ?? []);
    }
  }

  visit(dependencies);

  return [...unique.values()];
});

const rootVersion = $derived(project.versions.find((version) => version.id === versionId));

function getVersionNumber(dependency: ResolvedDependency): string {
  const version = dependency.project.project.versions.find((item) => item.id === dependency.project.version_id);

  return version?.version_number ?? dependency.project.version_id;
}

function isSelected(dependency: ResolvedDependency): boolean {
  return selectionState.active.has(projectKey(dependency));
}

function isEffectivelyRequired(dependency: ResolvedDependency): boolean {
  return selectionState.required.has(projectKey(dependency));
}

function toggleDependency(dependency: ResolvedDependency, checked: boolean, parentActive: boolean): void {
  if (installing || isEffectivelyRequired(dependency)) {
    return;
  }

  // Don't allow selecting an optional dependency beneath an inactive parent.
  // A shared dependency already active through another path remains usable.
  if (!parentActive && !selectionState.active.has(projectKey(dependency))) {
    return;
  }

  const key = projectKey(dependency);
  const next = new Set(selectedProjectIds);

  if (checked) {
    next.add(key);
  } else {
    next.delete(key);
  }

  selectedProjectIds = next;
}

async function confirmInstallation(): Promise<void> {
  if (installing) return;

  installing = true;
  errorMessage = null;

  try {
    await onConfirm(selectedDependencies);
  } catch (error) {
    errorMessage = error instanceof Error ? error.message : String(error);
  } finally {
    installing = false;
    app.dismissTop();
  }
}

function cancel(): void {
  if (installing) return;

  onCancel?.();
  app.dismissTop();
}
</script>

{#snippet renderDependency(dependency: ResolvedDependency, depth: number, parentActive: boolean)}
  {@const key = projectKey(dependency)}
  {@const selected = isSelected(dependency)}
  {@const required = isEffectivelyRequired(dependency)}
  {@const kind = dependencyKind(dependency)}
  {@const canToggle = !installing && !required && (parentActive || selected)}
  {@const children = dependency.dependencies ?? []}

  <div class="dependency-node" style={`--tree-depth: ${depth}`} data-selected={selected} data-required={required}>
    <label class="dependency-item" class:selectable={canToggle} class:checked={selected} class:inactive={!parentActive && !selected}>
      <div class="dependency-checkbox">
        <input
          type="checkbox"
          checked={selected}
          disabled={!canToggle}
          aria-label={`${dependency.project.project.title}, ${required ? "required" : displayDependencyType(dependency)}`}
          onchange={(event) => toggleDependency(dependency, event.currentTarget.checked, parentActive)} />
      </div>

      <div class="dependency-details">
        <div class="dependency-name">
          {dependency.project.project.title}
        </div>

        <div class="dependency-meta">
          <span>{getVersionNumber(dependency)}</span>
          <span class="meta-separator">·</span>
          <span>{dependency.project.project.project_type}</span>
        </div>

        <div class="dependency-filename">
          {dependency.project.filename}
        </div>
      </div>

      {#if required}
        <span class="type-label required-label">Required</span>
      {:else if selected}
        <span class="type-label included-label">Included</span>
      {:else if kind === "optional"}
        <span class="type-label optional-label">Optional</span>
      {:else}
        <span class="type-label other-label">{displayDependencyType(dependency)}</span>
      {/if}
    </label>

    {#if children.length > 0}
      <div class="dependency-children" aria-label={`Dependencies of ${dependency.project.project.title}`}>
        {#each children as child, index (`${projectKey(child)}-${index}`)}
          {@render renderDependency(child, depth + 1, selected)}
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<div class="dependency-dialog" role="dialog" aria-modal="true" aria-labelledby="dependency-title">
  <header class="dialog-header">
    <div class="heading-content">
      <div class="eyebrow">PROJECT INSTALLATION</div>
      <h2 id="dependency-title">Review dependencies</h2>
      <p class="description">
        {project.title} may require additional projects. Review the dependency tree before continuing.
      </p>
    </div>

    <button class="icon-button" type="button" aria-label="Close dialog" disabled={installing} onclick={cancel}>
      <span aria-hidden="true">×</span>
    </button>
  </header>

  <div class="install-summary">
    <div class="summary-project">
      <span class="summary-label">Installing</span>
      <strong>{project.title}</strong>
      <span class="summary-version">{rootVersion?.version_number ?? versionId}</span>
    </div>

    <div class="summary-project">
      <span class="summary-label">Profile</span>
      <strong>{profile.metadata.name}</strong>
      <span class="summary-version">{profile.version.minecraft_version}</span>
    </div>
  </div>

  <div class="dependency-list">
    {#if dependencies.length === 0}
      <div class="empty-state">
        <h3>No dependencies found</h3>
        <p>This project does not require any additional projects.</p>
      </div>
    {:else}
      <div class="tree-legend">
        <span class="legend-item">
          <span class="legend-dot required-dot"></span>
          Required
        </span>
        <span class="legend-item">
          <span class="legend-dot optional-dot"></span>
          Optional
        </span>
        <span class="legend-note">Shared dependencies use the same selection everywhere.</span>
      </div>

      <div class="dependency-tree">
        {#each dependencies as dependency, index (`${projectKey(dependency)}-${index}`)}
          {@render renderDependency(dependency, 0, true)}
        {/each}
      </div>
    {/if}
  </div>

  {#if errorMessage}
    <div class="error-message" role="alert">
      <strong>Installation failed</strong>
      <p>{errorMessage}</p>
      <p>You can retry without closing this dialog.</p>
    </div>
  {/if}

  <footer class="dialog-footer">
    <div class="selection-summary">
      <strong>{selectedDependencies.length}</strong>
      <span>
        {selectedDependencies.length === 1 ? "dependency" : "dependencies"} selected
      </span>
    </div>

    <div class="footer-actions">
      <button class="button secondary" type="button" disabled={installing} onclick={cancel}> Cancel </button>

      <button class="button primary" type="button" disabled={installing} onclick={confirmInstallation}>
        {#if installing}
          <span class="spinner"></span>
          Installing…
        {:else}
          Install {selectedDependencies.length + 1} projects
        {/if}
      </button>
    </div>
  </footer>
</div>

<style lang="scss">
.dependency-dialog {
  display: flex;
  flex-direction: column;
  width: min(calc(var(--layout-container-4) + 2 * var(--space-5)), calc(100vw - var(--space-6)));
  max-height: min(calc(var(--layout-container-5) + 5 * var(--space-1)), calc(100vh - 1.5 * var(--space-6)));
  overflow: hidden;
  color: $color-text;
  background: $color-surface-1;
  border: 1px solid $color-border;
  border-radius: $space-3;
  box-shadow: 0 $space-5 $space-8 $color-overlay;
}

.dialog-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: $space-5;
  padding: $space-5;
  border-bottom: 1px solid $color-border;
}

.heading-content {
  min-width: 0;
}

.eyebrow {
  margin-bottom: $space-2;
  color: $color-text-muted;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

h2,
h3,
p {
  margin: 0;
}

h2 {
  font-size: 22px;
  font-weight: 650;
  letter-spacing: -0.025em;
}

.description {
  max-width: calc(var(--layout-container-3) + 2 * var(--space-5));
  margin-top: $space-2;
  color: $color-text-muted;
  font-size: 13px;
  line-height: 1.6;
}

.icon-button {
  display: grid;
  flex: 0 0 auto;
  width: $space-6;
  height: $space-6;
  place-items: center;
  color: $color-text-muted;
  background: transparent;
  border: 1px solid transparent;
  border-radius: $space-2;
  cursor: pointer;
  font-size: 24px;
  line-height: 1;

  &:hover:not(:disabled) {
    color: $color-text;
    background: $color-hover;
    border-color: $color-border;
  }
}

.install-summary {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: $space-5;
  padding: $space-4 $space-5;
  background: $color-surface-2;
  border-bottom: 1px solid $color-border;
}

.summary-project {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: $space-1;
}

.summary-label {
  color: $color-text-muted;
  font-size: 11px;
}

.summary-project strong {
  overflow: hidden;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.summary-version {
  color: $color-text-muted;
  font-size: 11px;
}

.dependency-list {
  min-height: calc(var(--layout-container-1) * 0.4);
  overflow-y: auto;
  padding: $space-3 $space-5 $space-4;
}

.tree-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: $space-3;
  padding: $space-2 0 $space-3;
  border-bottom: 1px solid $color-border-muted;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: $space-1 + 1px;
  color: $color-text-muted;
  font-size: 11px;
}

.legend-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.required-dot {
  background: $color-info;
}

.optional-dot {
  background: $color-accent;
}

.legend-note {
  flex-basis: 100%;
  color: $color-placeholder;
  font-size: 11px;
}

.dependency-tree {
  padding-top: $space-2;
}

.dependency-node {
  min-width: 0;
}

.dependency-children {
  margin-left: $space-4;
  padding-left: $space-2;
  border-left: 1px solid $color-border;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: $space-3;
  min-width: 0;
  margin-top: $space-1;
  padding: $space-3 $space-2;
  border: 1px solid transparent;
  border-radius: $space-2;
  transition:
    background 120ms ease,
    border-color 120ms ease,
    opacity 120ms ease;

  &.selectable {
    cursor: pointer;

    &:hover {
      background: $color-hover;
      border-color: $color-border;
    }
  }

  &.checked {
    background: $color-selected;
    border-color: $color-border-muted;
  }

  &.inactive {
    opacity: 0.55;
  }
}

.dependency-checkbox {
  display: grid;
  flex: 0 0 calc(var(--space-4) + 2px);
  place-items: center;
}

.dependency-checkbox input {
  width: $space-4;
  height: $space-4;
  margin: 0;
  accent-color: $color-accent;
  cursor: inherit;

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }
}

.dependency-details {
  flex: 1;
  min-width: 0;
}

.dependency-name {
  overflow: hidden;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dependency-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-1) + 2px);
  margin-top: $space-1;
  color: $color-text-muted;
  font-size: 11px;
}

.meta-separator {
  opacity: 0.6;
}

.dependency-filename {
  overflow: hidden;
  margin-top: $space-1;
  color: $color-placeholder;
  font-family: monospace;
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.type-label {
  flex: 0 0 auto;
  padding: $space-1 $space-2;
  border: 1px solid $color-border;
  border-radius: calc(var(--space-1) + 1px);
  font-size: 10px;
  font-weight: 600;
}

.required-label {
  color: $color-info;
  background: $color-info-muted;
  border-color: $color-info;
}

.optional-label {
  color: $color-accent-muted;
  background: $color-selected;
  border-color: $color-accent-active;
}

.included-label {
  color: $color-success;
  background: $color-success-muted;
  border-color: $color-success;
}

.other-label {
  color: $color-text-muted;
  background: $color-surface-2;
}

.empty-state {
  padding: $space-7 $space-4;
  text-align: center;

  h3 {
    font-size: 15px;
  }

  p {
    margin-top: $space-2;
    color: $color-text-muted;
    font-size: 12px;
  }
}

.error-message {
  margin: 0 $space-5 $space-4;
  padding: $space-3 $space-4;
  color: $color-error;
  background: $color-error-muted;
  border: 1px solid $color-error;
  border-radius: $space-2;
  font-size: 12px;

  p {
    margin-top: $space-1;
    line-height: 1.5;
  }
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-4;
  padding: $space-4 $space-5;
  border-top: 1px solid $color-border;
  background: $color-surface-1;
}

.selection-summary {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-1) + 1px);
  color: $color-text-muted;
  font-size: 12px;

  strong {
    color: $color-text;
    font-variant-numeric: tabular-nums;
  }
}

.footer-actions {
  display: flex;
  flex: 0 0 auto;
  gap: $space-2;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-2;
  min-height: calc(var(--space-6) + var(--space-1));
  padding: 0 $space-3;
  border: 1px solid transparent;
  border-radius: calc(var(--space-2) - 1px);
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 120ms ease,
    border-color 120ms ease,
    opacity 120ms ease;

  &:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  &.secondary {
    color: $color-text;
    background: transparent;
    border-color: $color-border;

    &:hover:not(:disabled) {
      background: $color-hover;
    }
  }

  &.primary {
    color: $color-text;
    background: $color-accent;
    border-color: $color-accent;

    &:hover:not(:disabled) {
      background: $color-accent-hover;
      border-color: $color-accent-hover;
    }

    &:active:not(:disabled) {
      background: $color-accent-active;
      border-color: $color-accent-active;
    }

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: 2px;
    }
  }
}

.spinner {
  width: $space-3;
  height: $space-3;
  border: 2px solid $color-disabled;
  border-top-color: $color-text;
  border-radius: 50%;
  animation: spin 700ms linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 560px) {
  .dependency-dialog {
    width: calc(100vw - var(--space-5));
    max-height: calc(100vh - var(--space-6));
  }

  .dialog-header {
    padding: $space-4;
  }

  .install-summary {
    gap: $space-3;
    padding: $space-3 $space-4;
  }

  .dependency-list {
    padding-right: $space-3;
    padding-left: $space-3;
  }

  .dependency-children {
    margin-left: $space-3;
    padding-left: $space-1;
  }

  .dialog-footer {
    align-items: stretch;
    flex-direction: column;
    padding: $space-3 $space-4;
  }

  .footer-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .type-label {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .spinner {
    animation-duration: 1.5s;
  }

  .dependency-item,
  .button {
    transition: none;
  }
}
</style>
