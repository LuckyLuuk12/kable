<!-- @component
NavBar - Main navigation bar for the Kable launcher

Provides navigation between different sections (installations, mods, shaders, etc.),
displays user account information, and shows active installation status.
Initializes all required services on mount.

@example
```svelte
◄NavBar /►
```
-->
<script lang="ts">
import { resolve } from "$app/paths";
import { page } from "$app/state";
import { app, buttonSound, Icon, PlayerHead } from "$lib";
import "$lib/styles/global.scss";
import { onDestroy, onMount, type Snippet } from "svelte";

let { children }: { children: Snippet } = $props();

// Navigation items - conditionally include logs based on settings
let navItems = $derived([
  { path: "/", label: "Home", icon: "home" },
  { path: "/profiles", label: "Profiles", icon: "minecraft" },
  { path: "/mods", label: "Mods", icon: "mods" },
  { path: "/resourcepacks", label: "Resource Packs", icon: "image" },
  { path: "/shaders", label: "Shaders", icon: "shaders" },
  // { path: '/resources', label: 'Resources', icon: 'resources' },
  { path: "/worlds", label: "Worlds", icon: "world" },
  { path: "/skins", label: "Skins", icon: "palette" },
  // Only show logs if enabled in settings (default: true for developers)
  ...(app.customizationService.settings?.advanced?.enable_advanced_features !== false ? [{ path: "/logs", label: "Logs", icon: "terminal" }] : []),
  // Only show advanced page if enabled in settings (default: false)
  ...(app.customizationService.settings?.advanced?.enable_advanced_features === true ? [{ path: "/advanced", label: "Advanced", icon: "wrench" }] : []),
]);

// State for navigation collapse
let activeAccount = $derived(app.authService.activeAccount);
let isNavCollapsed = $state(true);
let currentPath = $derived(page.url.pathname);
$effect(() => console.log(currentPath));

function toggleNavigation() {
  isNavCollapsed = !isNavCollapsed;
}

// Handle keyboard shortcuts
function handleKeydown(event: KeyboardEvent) {
  // Ctrl/Cmd + B to toggle navigation
  if ((event.ctrlKey || event.metaKey) && event.key === "b") {
    event.preventDefault();
    toggleNavigation();
  }
}

// Tooltip element and logic for showing a single tooltip (prevents native title tooltip duplicates)
let tooltipEl: HTMLDivElement | null = null;
let tooltipTimer: number | null = null;

function showTooltipForTarget(target: Element | null) {
  if (!tooltipEl || !target) return;
  const title = (target as HTMLElement).dataset?.title || (target as HTMLElement).getAttribute("aria-label") || "";
  if (!title) return;

  tooltipEl.textContent = title;
  tooltipEl.setAttribute("aria-hidden", "false");
  tooltipEl.classList.add("visible");

  // position next to element
  const rect = target.getBoundingClientRect();
  const left = rect.right + 8; // 8px gap
  const top = rect.top + rect.height / 2;
  tooltipEl.style.left = `${Math.max(8, left)}px`;
  tooltipEl.style.top = `${top}px`;
}

function hideTooltip() {
  if (!tooltipEl) return;
  tooltipEl.setAttribute("aria-hidden", "true");
  tooltipEl.classList.remove("visible");
}

function attachTooltipListeners() {
  // attach to nav-items inside the sidebar
  const items = document.querySelectorAll(".sidebar .nav-item");
  items.forEach((item) => {
    // remove possible existing handlers to avoid duplicates
    item.removeEventListener("mouseenter", itemMouseEnter as EventListener);
    item.removeEventListener("mouseleave", itemMouseLeave as EventListener);
    item.removeEventListener("focus", itemFocus as EventListener, true);
    item.removeEventListener("blur", itemBlur as EventListener, true);

    item.addEventListener("mouseenter", itemMouseEnter as EventListener);
    item.addEventListener("mouseleave", itemMouseLeave as EventListener);
    item.addEventListener("focus", itemFocus as EventListener, true);
    item.addEventListener("blur", itemBlur as EventListener, true);
  });
}

function itemMouseEnter(e: Event) {
  const target = e.currentTarget as Element;
  // delay slightly so quick mouse passes don't flash tooltip
  tooltipTimer = window.setTimeout(() => showTooltipForTarget(target), 60);
}

function itemMouseLeave() {
  if (tooltipTimer) {
    clearTimeout(tooltipTimer);
    tooltipTimer = null;
  }
  hideTooltip();
}

function itemFocus(e: Event) {
  const target = e.currentTarget as Element;
  showTooltipForTarget(target);
}

function itemBlur() {
  hideTooltip();
}

onMount(() => {
  // create tooltip element if not present
  if (!tooltipEl) {
    const el = document.createElement("div");
    el.className = "nav-tooltip";
    el.setAttribute("role", "tooltip");
    el.setAttribute("aria-hidden", "true");
    document.body.appendChild(el);
    tooltipEl = el as HTMLDivElement;
  }
  attachTooltipListeners();
});

onDestroy(() => {
  // cleanup
  const items = document.querySelectorAll(".sidebar .nav-item");
  items.forEach((item) => {
    item.removeEventListener("mouseenter", itemMouseEnter as EventListener);
    item.removeEventListener("mouseleave", itemMouseLeave as EventListener);
    item.removeEventListener("focus", itemFocus as EventListener, true);
    item.removeEventListener("blur", itemBlur as EventListener, true);
  });
  if (tooltipEl && tooltipEl.parentNode) tooltipEl.parentNode.removeChild(tooltipEl);
  tooltipEl = null;

  // Cleanup all services on destroy (e.g., when app is closed)
  app.destroyAll().catch((err) => {
    console.error("Failed to destroy all services:", err);
  });
});
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="app-layout" class:nav-open={!isNavCollapsed} onkeydown={handleKeydown} role="application" tabindex="-1">
  <nav class="sidebar" class:collapsed={isNavCollapsed}>
    <!-- Header Section with Profile -->
    <div class="header-section">
      <a
        use:buttonSound
        href={resolve("/accounts", {})}
        class="user-profile"
        class:active={currentPath === "/accounts"}
        data-title="Account Settings"
        aria-label="Account Settings">
        <div class="user-avatar">
          <PlayerHead account={activeAccount} size={40} />
        </div>
        <div class="header-content" class:collapsed={isNavCollapsed}>
          <h1 class="app-title">{activeAccount?.username}</h1>
          <span class="app-subtitle">{!!activeAccount?.access_token ? "Logged in" : "Not logged in"}</span>
        </div>
      </a>
    </div>

    <!-- Hamburger Toggle -->
    <div class="hamburger-section">
      <button
        use:buttonSound
        class="hamburger-btn"
        onclick={toggleNavigation}
        aria-label={isNavCollapsed ? "Expand navigation" : "Collapse navigation"}
        data-title={isNavCollapsed ? "Expand navigation (Ctrl+B)" : "Collapse navigation (Ctrl+B)"}>
        <Icon name={isNavCollapsed ? "arrow-right" : "arrow-left"} size="lg" forceType="svg" />
      </button>
    </div>

    <!-- Main Navigation -->
    <div class="nav-items">
      {#each navItems as item (item.path)}
        <a use:buttonSound href={resolve(item.path, {})} class="nav-item" class:active={currentPath === item.path} data-title={item.label} aria-label={item.label}>
          <Icon name={item.icon} size="md" className="nav-icon" />
          <span class="label" class:collapsed={isNavCollapsed}>{item.label}</span>
        </a>
      {/each}
    </div>

    <!-- Settings at Bottom -->
    <div class="bottom-section">
      <a
        use:buttonSound
        href={resolve("/settings", {})}
        class="nav-item settings-item"
        class:active={currentPath === "/settings"}
        data-title="Settings"
        aria-label="Settings">
        <Icon name="settings" size="md" className="nav-icon" />
        <span class="label" class:collapsed={isNavCollapsed}>Settings</span>
      </a>
    </div>
  </nav>

  <main class="content">
    {@render children()}
  </main>
</div>

<style lang="scss">
.app-layout {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background: $color-background;
  color: $color-text;
}

.sidebar {
  min-width: calc(fit-content + 2rem);
  background: $color-surface-1;
  border-right: 1px solid $color-border;
  display: flex;
  flex-direction: column;
  padding: 0.25rem;
  transition: width 0.3s ease;
  resize: horizontal;

  &.collapsed {
    width: 3.5rem;

    .hamburger-btn {
      margin: 0 -0.25rem;
    }
    .header-section > .user-profile {
      margin: 0 0.25rem;
    }
  }
}

.header-section {
  margin-bottom: 1rem;

  .user-profile {
    margin-left: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0;
    border-radius: var(--border-radius);
    text-decoration: none;
    color: var(--text);
    transition: all 0.2s ease;
    cursor: pointer;

    &:hover,
    &.active {
      & .user-avatar {
        background: $color-accent;
      }
    }

    .user-avatar {
      margin-top: 0.5rem;
      width: 2.5rem;
      height: 2.5rem;
      border-radius: 40%;
      background: color-mix(in srgb, $color-accent, 10%, transparent);
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--text-white);
      flex-shrink: 0;
      overflow: hidden;
      position: relative;
    }

    .header-content {
      display: flex;
      flex-direction: column;
      min-width: 0;

      &.collapsed {
        display: none;
      }

      .app-title {
        margin: 0;
        font-size: 1rem;
        font-weight: 800;
        color: $color-accent;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .app-subtitle {
        font-size: 0.65rem;
        color: var(--placeholder);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
    }
  }
}

.hamburger-section {
  margin-bottom: 1.5rem;

  .hamburger-btn {
    background: transparent;
    border: none;
    border-radius: var(--border-radius);
    padding: 0 0.75rem;
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      color: $color-accent;
      border-color: $color-accent;
    }
  }
}

.nav-items {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: var(--border-radius);
  text-decoration: none;
  color: var(--text);
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    background: $color-surface-2;
  }

  &.active {
    background: linear-gradient(155deg, #{"color-mix(in srgb, $color-accent 15%, transparent)"}, #{"color-mix(in srgb, $color-accent 1%, transparent)"});
    backdrop-filter: blur(15px);
    color: var(--text-white);
  }

  :global(.nav-icon) {
    flex-shrink: 0;
  }

  .label {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;

    &.collapsed {
      display: none;
    }
  }

  // Collapsed state - center icons and show tooltips
  .sidebar.collapsed & {
    justify-content: center;
    padding: 0.75rem;

    // tooltip handled by JS-controlled .nav-tooltip element
  }
}

.bottom-section {
  margin-top: auto;
  border-top: 1px solid $color-border;
}

.content {
  flex: 1;
  overflow-y: auto;
  transition: margin-left 0.3s ease;
}

// Mobile responsive behavior
@media (max-width: 768px) {
  .sidebar {
    position: fixed;
    left: 0;
    top: 0;
    height: 100vh;
    z-index: 1000;
    transform: translateX(-100%);
    transition: transform 0.3s ease;

    &:not(.collapsed) {
      transform: translateX(0);
    }

    &.collapsed {
      transform: translateX(0);
      width: 60px;
    }
  }

  .content {
    margin-left: 0;
  }

  .app-layout {
    &::before {
      content: "";
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      z-index: 10;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.3s ease;
    }

    &.nav-open::before {
      opacity: 1;
      pointer-events: auto;
    }
  }
}

/* global tooltip element used by the sidebar for collapsed-state tooltips */
:global(.nav-tooltip) {
  position: fixed;
  transform: translateY(-50%);
  left: 0;
  top: 0;
  background: $color-background;
  color: $color-text;
  padding: 0.375rem 0.6rem;
  border-radius: calc(var(--radius-md) * 0.85);
  font-size: 0.875rem;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition:
    opacity 0.12s ease,
    transform 0.12s ease;
  border: 1px solid $color-border;
  z-index: 2147483647;
}

:global(.nav-tooltip.visible) {
  opacity: 1;
  pointer-events: none;
  transform: translateY(-50%);
}
</style>
