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
import "$lib/styles/global.scss";
import { get } from "svelte/store";
import { onMount, onDestroy } from "svelte";
import { page } from "$app/stores";
import { goto } from "$app/navigation";
import {
  currentAccount,
  AuthService,
  SettingsService,
  InstallationService,
  Icon,
  PlayerHead,
  logsService,
  LogsService,
  IconService,
  UpdaterService,
  settings,
  isLaunching,
  currentLaunchingInstallation,
  launchTimeoutHandle,
} from "$lib";
import type {
  NavigationEventPayload,
  BehaviorChoiceEventPayload,
  GameRestartEventPayload,
} from "$lib";

// Here we initialize all the required managers and services
onMount(async () => {
  console.log("Starting layout initialization...");
  // Wait a bit for Tauri to fully initialize
  await new Promise((resolve) => setTimeout(resolve, 50));
  try {
    // Initialize progressive loading listeners FIRST
    await InstallationService.initializeProgressiveLoading();

    // Initialize logs service early so we can emit events
    const logsPromise = logsService.initialize();

    try {
      await logsPromise;
      LogsService.emitLauncherEvent("Kable launcher starting up...", "info");
      LogsService.emitLauncherEvent(
        "Initializing launcher components...",
        "info",
      );
    } catch (e) {
      console.error("Failed to initialize logs service:", e);
    }

    // CRITICAL: Initialize auth service FIRST and WAIT for it
    // This ensures accounts are loaded before user can launch games
    try {
      LogsService.emitLauncherEvent("Initializing authentication...", "info");
      await AuthService.initialize();
      await AuthService.refreshCurrentAccount();
      LogsService.emitLauncherEvent(
        "Authentication initialized successfully",
        "info",
      );

      // Refresh tokens for all accounts in background to ensure they're ready to use
      AuthService.refreshAllAccountTokens().catch((error) => {
        console.error("Failed to refresh all account tokens:", error);
      });
    } catch (e) {
      console.error("Failed to initialize auth service:", e);
      LogsService.emitLauncherEvent(
        `Authentication initialization failed: ${e}`,
        "error",
      );
    }

    // Now start other initialization tasks concurrently (non-blocking)
    const installPromise = InstallationService.loadInstallations();
    const settingsPromise = SettingsService.initialize();
    const iconPromise = IconService.initialize();

    // Load versions in the background AFTER installations start loading
    InstallationService.loadVersions().catch((e) => {
      console.error("Failed to load versions:", e);
    });

    // Await remaining initializations but do not fail fast — collect results
    const results = await Promise.allSettled([
      installPromise,
      settingsPromise,
      iconPromise,
    ]);

    // Check for updates on launch (after settings are loaded)
    if ($settings?.general?.auto_update_launcher !== false) {
      UpdaterService.checkForUpdatesOnLaunch().catch((error: unknown) => {
        console.error("Failed to check for updates on launch:", error);
      });
    }

    // Emit final status and setup listeners (even if some inits failed) — errors are logged above
    try {
      LogsService.emitLauncherEvent(
        "All components initialized successfully",
        "info",
      );
    } catch (e) {
      console.error("Failed to emit final init log:", e);
    }
    console.log("Layout initialization complete");

    // Set up settings behavior event listeners
    await setupSettingsEventListeners();
  } catch (error) {
    console.error("Tauri initialization error:", error);
    LogsService.emitLauncherEvent(`Initialization error: ${error}`, "error");
  }
});

// Set up event listeners for settings behavior
async function setupSettingsEventListeners() {
  try {
    const { listen } = await import("@tauri-apps/api/event");

    // Navigation events
    await listen<NavigationEventPayload>("navigate-to-logs", (event) => {
      console.log("Navigating to logs due to settings:", event.payload);
      LogsService.emitLauncherEvent(
        "Navigating to logs page due to game settings",
        "info",
      );
      goto("/logs");
    });

    await listen<NavigationEventPayload>("navigate-to-home", (event) => {
      console.log("Navigating to home due to settings:", event.payload);
      LogsService.emitLauncherEvent(
        "Navigating to home page due to game settings",
        "info",
      );
      goto("/");
    });

    // User choice dialogs
    await listen<BehaviorChoiceEventPayload>(
      "ask-launch-behavior",
      async (event) => {
        console.log(
          "User choice requested for launch behavior:",
          event.payload,
        );
        const choice = await showBehaviorDialog(
          "Launch Behavior",
          "What should happen when the game launches?",
          event.payload.options,
        );
        if (choice) {
          await handleUserChoice("on_game_launch", choice);
        }
      },
    );

    await listen<BehaviorChoiceEventPayload>(
      "ask-close-behavior",
      async (event) => {
        console.log("User choice requested for close behavior:", event.payload);
        const choice = await showBehaviorDialog(
          "Close Behavior",
          `What should happen now? (Game exited with code ${event.payload.exit_code})`,
          event.payload.options,
        );
        if (choice) {
          await handleUserChoice("on_game_close", choice);
        }
      },
    );

    await listen<BehaviorChoiceEventPayload>(
      "ask-crash-behavior",
      async (event) => {
        console.log("User choice requested for crash behavior:", event.payload);
        const choice = await showBehaviorDialog(
          "Game Crashed",
          `The game crashed (exit code ${event.payload.exit_code}). What should we do?`,
          event.payload.options,
        );
        if (choice) {
          await handleUserChoice("on_game_crash", choice);
        }
      },
    );

    await listen<GameRestartEventPayload>("game-restart-requested", (event) => {
      console.log("Game restart requested:", event.payload);
      LogsService.emitLauncherEvent(
        `Game restart requested due to crash (exit code: ${event.payload.exit_code})`,
        "warn",
      );
      // TODO: Implement game restart functionality
      alert(
        "Game restart feature is not implemented yet. Please launch manually.",
      );
    });

    // Game started event - clear launching UI indicators
    await listen<{ pid: number; installation_id: string }>(
      "game-started",
      (event) => {
        console.log("Game started event received:", event.payload);
        LogsService.emitLauncherEvent(
          `Game started (PID: ${event.payload.pid})`,
          "info",
        );
        try {
          isLaunching.set(false);
          currentLaunchingInstallation.set(null);
          // Clear fallback timeout if set
          try {
            const prev = get(launchTimeoutHandle);
            if (prev) clearTimeout(prev);
            launchTimeoutHandle.set(null);
          } catch (e) {
            console.warn("Failed to clear launch timeout handle", e);
          }
        } catch (e) {
          console.warn(
            "Failed to clear launching indicators on game-started event:",
            e,
          );
        }
      },
    );

    LogsService.emitLauncherEvent(
      "Settings behavior event listeners initialized",
      "info",
    );
  } catch (error) {
    console.error("Failed to set up settings event listeners:", error);
    LogsService.emitLauncherEvent(
      `Failed to set up settings event listeners: ${error}`,
      "error",
    );
  }
}

// Show a dialog for user behavior choice
async function showBehaviorDialog(
  title: string,
  message: string,
  options: string[],
): Promise<string | null> {
  console.log("Showing behavior dialog:", { title, message, options });
  const optionLabels: Record<string, string> = {
    keep_open: "Keep Launcher Open",
    exit: "Close Launcher",
    minimize: "Minimize Launcher",
    open_logs: "Open Logs Page",
    open_home: "Go to Home Page",
    restart: "Restart Game",
    close: "Close Launcher",
    ask: "Ask Me Each Time",
  };

  const buttons = options.map((opt) => optionLabels[opt] || opt);

  // Use browser's confirm for now - could be replaced with a custom modal
  if (options.length === 2) {
    const result = confirm(
      `${title}\n\n${message}\n\nClick OK for "${buttons[0]}" or Cancel for "${buttons[1]}"`,
    );
    return result ? options[0] : options[1];
  } else {
    // For multiple options, show a simple prompt
    let promptMessage = `${title}\n\n${message}\n\nOptions:\n`;
    buttons.forEach((label, index) => {
      promptMessage += `${index + 1}. ${label}\n`;
    });
    promptMessage += "\nEnter the number of your choice:";

    const choice = prompt(promptMessage);
    const choiceIndex = parseInt(choice || "0") - 1;

    if (choiceIndex >= 0 && choiceIndex < options.length) {
      return options[choiceIndex];
    }
  }

  return null;
}

// Handle user's choice by executing the action
async function handleUserChoice(settingType: string, choice: string) {
  LogsService.emitLauncherEvent(
    `User chose "${choice}" for ${settingType}`,
    "info",
  );

  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const window = getCurrentWindow();

    switch (choice) {
      case "exit":
      case "close":
        await window.close();
        break;
      case "minimize":
        await window.minimize();
        break;
      case "open_logs":
        goto("/logs");
        break;
      case "open_home":
        goto("/");
        break;
      case "restart":
        LogsService.emitLauncherEvent("Game restart requested by user", "info");
        alert(
          "Game restart feature is not implemented yet. Please launch manually.",
        );
        break;
      case "keep_open":
        // Do nothing - keep launcher open
        LogsService.emitLauncherEvent(
          "Keeping launcher open as requested",
          "info",
        );
        break;
      default:
        console.warn(`Unknown choice: ${choice}`);
    }
  } catch (error) {
    console.error("Error handling user choice:", error);
    LogsService.emitLauncherEvent(
      `Error handling user choice: ${error}`,
      "error",
    );
  }
}

// Navigation items - conditionally include logs based on settings
$: navItems = [
  { path: "/", label: "Home", icon: "home" },
  { path: "/installations", label: "Installations", icon: "minecraft" },
  { path: "/mods", label: "Mods", icon: "mods" },
  { path: "/resourcepacks", label: "Resource Packs", icon: "image" },
  { path: "/shaders", label: "Shaders", icon: "shaders" },
  // { path: '/resources', label: 'Resources', icon: 'resources' },
  { path: "/maps", label: "Worlds", icon: "world" },
  { path: "/skins", label: "Skins", icon: "palette" },
  // Only show logs if enabled in settings (default: true for developers)
  ...($settings?.logging.show_logs_page_in_nav !== false
    ? [{ path: "/logs", label: "Logs", icon: "terminal" }]
    : []),
  // Only show advanced page if enabled in settings (default: false)
  ...($settings?.advanced.show_advanced_page === true
    ? [{ path: "/advanced", label: "Advanced", icon: "wrench" }]
    : []),
];

// State for navigation collapse
let isNavCollapsed = true;

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

$: currentPath = $page.url.pathname;
$: () => console.log(currentPath);

// Tooltip element and logic for showing a single tooltip (prevents native title tooltip duplicates)
let tooltipEl: HTMLDivElement | null = null;
let tooltipTimer: number | null = null;

function showTooltipForTarget(target: Element | null) {
  if (!tooltipEl || !target) return;
  const title =
    (target as HTMLElement).dataset?.title ||
    (target as HTMLElement).getAttribute("aria-label") ||
    "";
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
  if (tooltipEl && tooltipEl.parentNode)
    tooltipEl.parentNode.removeChild(tooltipEl);
  tooltipEl = null;
});
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="app-layout"
  class:nav-open={!isNavCollapsed}
  on:keydown={handleKeydown}
  role="application"
  tabindex="-1"
>
  <nav class="sidebar" class:collapsed={isNavCollapsed}>
    <!-- Header Section with Profile -->
    <div class="header-section">
      <a
        href="/profile"
        class="user-profile"
        class:active={currentPath === "/profile"}
      >
        <div class="user-avatar">
          <PlayerHead account={$currentAccount} size={40} />
        </div>
        {#if !isNavCollapsed}
          <div class="header-content">
            <h1 class="app-title">{$currentAccount?.username}</h1>
            <span class="app-subtitle"
              >{!!$currentAccount?.access_token
                ? "Logged in"
                : "Not logged in"}</span
            >
          </div>
        {/if}
      </a>
    </div>

    <!-- Hamburger Toggle -->
    <div class="hamburger-section">
      <button
        class="hamburger-btn"
        on:click={toggleNavigation}
        aria-label={isNavCollapsed
          ? "Expand navigation"
          : "Collapse navigation"}
        data-title={isNavCollapsed
          ? "Expand navigation (Ctrl+B)"
          : "Collapse navigation (Ctrl+B)"}
      >
        <Icon
          name={isNavCollapsed ? "arrow-right" : "arrow-left"}
          size="lg"
          forceType="svg"
        />
      </button>
    </div>

    <!-- Main Navigation -->
    <div class="nav-items">
      {#each navItems as item}
        <a
          href={item.path}
          class="nav-item"
          class:active={currentPath === item.path}
          data-title={item.label}
          aria-label={item.label}
        >
          <Icon name={item.icon} size="md" className="nav-icon" />
          {#if !isNavCollapsed}
            <span class="label">{item.label}</span>
          {/if}
        </a>
      {/each}
    </div>

    <!-- Settings at Bottom -->
    <div class="bottom-section">
      <a
        href="/settings"
        class="nav-item settings-item"
        class:active={currentPath === "/settings"}
        data-title="Settings"
        aria-label="Settings"
      >
        <Icon name="settings" size="md" className="nav-icon" />
        {#if !isNavCollapsed}
          <span class="label">Settings</span>
        {/if}
      </a>
    </div>
  </nav>

  <main class="content">
    <slot />
  </main>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;

.app-layout {
  display: flex;
  height: 100%;
  overflow: scroll;
  background: var(--background);
  color: var(--text);
}

.sidebar {
  min-width: calc(fit-content + 2rem);
  background: var(--container);
  border-right: 1px solid var(--dark-600);
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
        background: var(--primary);
      }
    }

    .user-avatar {
      margin-top: 0.5rem;
      width: 2.5rem;
      height: 2.5rem;
      border-radius: 40%;
      background: color-mix(in srgb, var(--primary), 10%, transparent);
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

      .app-title {
        margin: 0;
        font-size: 1rem;
        font-weight: 800;
        color: var(--primary);
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
      color: var(--primary);
      border-color: var(--primary);
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
    background: var(--button-hover);
  }

  &.active {
    background: linear-gradient(
      155deg,
      #{"color-mix(in srgb, var(--primary) 15%, transparent)"},
      #{"color-mix(in srgb, var(--primary) 1%, transparent)"}
    );
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
  }

  // Collapsed state - center icons and show tooltips
  .sidebar.collapsed & {
    justify-content: center;
    padding: 0.75rem;

    .label {
      display: none;
    }

    // tooltip handled by JS-controlled .nav-tooltip element
  }
}

.bottom-section {
  margin-top: auto;
  border-top: 1px solid var(--dark-600);
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
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
    padding: 1rem;
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
  background: var(--container);
  color: var(--text);
  padding: 0.375rem 0.6rem;
  border-radius: calc(var(--border-radius) * 0.85);
  font-size: 0.875rem;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition:
    opacity 0.12s ease,
    transform 0.12s ease;
  border: 1px solid var(--dark-600);
  z-index: 2147483647;
}

:global(.nav-tooltip.visible) {
  opacity: 1;
  pointer-events: none;
  transform: translateY(-50%);
}
</style>
