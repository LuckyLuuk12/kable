<script lang="ts">
import { onMount } from "svelte";
import {
  Icon,
  Image,
  installations,
  isLoadingInstallations,
  installationsError,
  type KableInstallation,
  Launcher,
  InstallationService,
  settings,
} from "$lib";
import {
  isLaunching,
  currentLaunchingInstallation,
} from "$lib/stores/launcher";
import InstallationsList from "$lib/components/installations/InstallationsList.svelte";
import { openUrl } from "$lib/api/system";

// State variables
let lastPlayedInstallations: KableInstallation[] = [];
let error: string | null = null;
let viewMode: "grid" | "list" = "grid";
let launchStatus = "";
let openDropdownId: string | null = null;

// RAM allocation state
let ramAllocation = 2048; // Default 2GB in MB
let ramInputValue = "2048"; // String value for text input
let isEditingRam = false;
let _commitTimer: ReturnType<typeof setTimeout> | null = null;
let skipReactiveSyncUntil = 0;

// Subscribe to the installations store and update RAM allocation
$: {
  // Only run reactive sync when not editing and not in the skip window
  if (Date.now() >= skipReactiveSyncUntil && !isEditingRam) {
    console.log("Total installations:", $installations.length);

    lastPlayedInstallations = $installations
      .sort((a: KableInstallation, b: KableInstallation) => {
        const aTime = new Date(a.last_used || 0).getTime();
        const bTime = new Date(b.last_used || 0).getTime();
        return bTime - aTime;
      })
      .slice(0, 8); // Show up to 8 installations

    console.log("Last played installations:", lastPlayedInstallations.length);

    // Update RAM allocation when installation changes
    if (lastPlayedInstallations.length > 0) {
      const latestInstallation = lastPlayedInstallations[0];
      console.log("Latest installation java_args:", latestInstallation.java_args);

      // Extract RAM from java_args (look for -Xmx)
      const xmxArg = latestInstallation.java_args?.find((arg) =>
        arg.startsWith("-Xmx"),
      );
      if (xmxArg) {
        console.log("Found Xmx arg:", xmxArg);
        const memValue = xmxArg.replace("-Xmx", "").toLowerCase();
        if (memValue.endsWith("g")) {
          ramAllocation = parseInt(memValue) * 1024;
        } else if (memValue.endsWith("m")) {
          ramAllocation = parseInt(memValue);
        }
        ramInputValue = ramAllocation.toString();
        console.log("Set RAM allocation to:", ramAllocation, "MB");
      }
    }
  }
}
// Check if ads should be shown from settings
$: showAds = $settings?.general?.show_ads ?? false;

// Subscribe to loading and error states
$: isLoading = $isLoadingInstallations;
$: if ($installationsError) {
  error = $installationsError;
}

// Initialize on component mount
onMount(() => {
  console.log("Home page mounted");
  // GameManager is already initialized by the layout with installations loaded

  // Add click outside handler for dropdown
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as Element;
    if (!target.closest(".dropdown-container")) {
      openDropdownId = null;
    }
  }
  document.addEventListener("click", handleClickOutside);

  // Return cleanup function
  return () => {
    document.removeEventListener("click", handleClickOutside);
  };
});

function toggleViewMode() {
  viewMode = viewMode === "grid" ? "list" : "grid";
}

function getModLoaderIcon(modLoader: string) {
  switch (modLoader) {
    case "fabric":
      return "fabric";
    case "forge":
      return "hammer";
    default:
      return "cube";
  }
}

function toggleDropdown(installationId: string) {
  openDropdownId = openDropdownId === installationId ? null : installationId;
}

function closeDropdown() {
  openDropdownId = null;
}

async function handlePlay() {
  isLaunching.set(true);
  launchStatus = "Preparing to launch...";
  let result;

  try {
    // Try to launch the most recent installation
    if (lastPlayedInstallations.length > 0) {
      console.log("Launching installation:", lastPlayedInstallations[0]);
      launchStatus = `Launching ${lastPlayedInstallations[0].name}...`;
      // Launch the installation directly using Launcher
      result = await Launcher.launchInstallation(lastPlayedInstallations[0]);
    } else {
      launchStatus = "Launching default Minecraft...";
      // Use Launcher for quick launch fallback
      result = await Launcher.launchLatest();
    }

    if (result.success) {
      launchStatus = "Launched Minecraft!";
    } else {
      launchStatus = `Launch failed: ${result.error || "Unknown error"}`;
    }
  } catch (err) {
    console.error("Launch error:", err);
    launchStatus = `Launch failed: ${err}`;
  } finally {
    // Reset the button state quickly since Minecraft is now running independently
    setTimeout(
      () => {
        launchStatus = "";
        isLaunching.set(false);
      },
      result?.success ? 2000 : 5000,
    );
  }
}

async function handleInstallationLaunch(installation: KableInstallation) {
  const launchButton = event?.target as HTMLButtonElement;
  const originalText = launchButton?.textContent || "";

  if (launchButton) {
    launchButton.disabled = true;
    launchButton.textContent = "Launching...";
  }

  try {
    // Launch the installation directly using Launcher
    const result = await Launcher.launchInstallation(installation);

    if (result.success) {
      if (launchButton) {
        launchButton.textContent = "Launched!";
      }
      // Installations will be kept in sync by the centralized bootstrap and store updates
    } else {
      alert(`Launch failed: ${result.error || "Unknown error"}`);
    }
  } catch (err) {
    console.error("Installation launch error:", err);
    alert(`Launch failed: ${err}`);
  } finally {
    // Reset button state after a short delay
    setTimeout(() => {
      if (launchButton) {
        launchButton.disabled = false;
        launchButton.textContent = originalText;
      }
    }, 2000);
  }
}

// RAM allocation functions
function updateRamFromSlider() {
  // Update only the display string while sliding; persist on change/commit
  ramInputValue = ramAllocation.toString();
}

function updateRamFromInput() {
  const value = parseInt(ramInputValue);
  if (!isNaN(value) && value >= 512 && value <= 32768) {
    // Round to nearest 256MB increment
    const roundedValue = Math.round(value / 256) * 256;
    // Ensure it stays within bounds after rounding
    const clampedValue = Math.max(512, Math.min(32768, roundedValue));
    ramAllocation = clampedValue;
    ramInputValue = clampedValue.toString();
  } else {
    // Reset to current valid value if invalid input
    ramInputValue = ramAllocation.toString();
  }
}

// Helper to set or replace -Xmx arg in java_args array
function setXmxArg(args: string[] = [], mb: number): string[] {
  const newArgs = [...args];
  const idx = newArgs.findIndex((a) => a.startsWith("-Xmx"));
  const memStr = mb % 1024 === 0 ? `${mb / 1024}G` : `${mb}M`;
  const xmx = `-Xmx${memStr}`;
  if (idx !== -1) {
    newArgs[idx] = xmx;
  } else {
    newArgs.push(xmx);
  }
  return newArgs;
}

// Commit the current ramAllocation to the most-recent installation (debounced)
async function commitRamChange(immediate = false) {
  // If immediate, run commit now; otherwise debounce to avoid excessive writes
  if (!immediate) {
    if (_commitTimer) clearTimeout(_commitTimer);
    _commitTimer = setTimeout(() => {
      _commitTimer = null;
      // call the actual commit
      commitRamChange(true).catch((e) => console.error(e));
    }, 300);
    return;
  }

  // Immediate commit path
  if (lastPlayedInstallations.length === 0) return;
  const inst = lastPlayedInstallations[0];
  isEditingRam = true; // keep UI stable while we persist
  try {
    const newArgs = setXmxArg(inst.java_args || [], ramAllocation);
    const updated = { ...inst, java_args: newArgs } as typeof inst;
    console.log("Committing RAM change for installation:", inst.id, ramAllocation, "MB");
    await InstallationService.updateInstallation(inst.id, updated);
    console.log("RAM allocation committed");
  } catch (err) {
    console.error("Failed to commit RAM allocation:", err);
  } finally {
    // allow reactive updates after commit completes, but delay briefly to avoid races
    // re-apply UI values optimistically so the UI doesn't flicker back
    ramInputValue = ramAllocation.toString();
    skipReactiveSyncUntil = Date.now() + 1000; // 1s grace period
    isEditingRam = false;
  }
}

function formatRamDisplay(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)}GB`;
  }
  return `${mb}MB`;
}

// Handle advertisement link clicks
async function handleAdClick(url: string) {
  try {
    await openUrl(url);
  } catch (err) {
    console.error("Failed to open URL:", err);
  }
}
</script>

<div class="page-wrapper">
  <!-- Personal advertisement (which users can disable in settings) -->
  {#if showAds}
    <div class="advertisement-banner">
      <div class="banner-background">
        <Image
          key="advertisement-banner"
          alt="Banner"
          className="banner-image"
          width="100%"
          height="100%"
        />
      </div>
      <div class="banner-overlay"></div>
      <div class="banner-content">
        <div class="banner-actions">
          <button
            class="banner-button primary"
            on:click={() => handleAdClick("https://kablan.nl")}
          >
            <Image
              key="kablan-logo"
              alt="Kablan"
              className="button-image"
              width="auto"
              height="2.5rem"
            />
            <span>Kablan.nl</span>
          </button>

          <button
            class="banner-button secondary"
            on:click={() =>
              handleAdClick("https://modrinth.com/mod/luckybindings")}
          >
            <Image
              key="luckybindings-logo"
              alt="LuckyBindings"
              className="button-image"
              width="auto"
              height="2.5rem"
            />
            <span>LuckyBindings Mod</span>
          </button>

          <button
            class="banner-button kofi"
            on:click={() => handleAdClick("https://ko-fi.com/luckyluuk")}
          >
            <Image
              key="kofi-logo"
              alt="Ko-fi"
              className="button-image"
              width="auto"
              height="2.5rem"
            />
            <span>Support me on Ko-fi</span>
          </button>
        </div>

        <div class="artist-recruitment">
          <span class="recruitment-text"
            >Are you an artist, willing to improve my tools?
          </span>
          <button
            class="recruitment-link"
            on:click={() => handleAdClick("https://discord.gg/qRTevFvHbx")}
          >
            Get in contact with me
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Installations List Section -->
  <div class="installations-section">
    <InstallationsList isGrid isSmall limit={15} />
  </div>

  <!-- Bottom Controls Section -->
  <div class="bottom-controls">
    <!-- Play Button (Centered) -->
    <div class="play-section">
      <button
        class="play-button"
        on:click={handlePlay}
        disabled={$isLaunching || lastPlayedInstallations.length === 0}
      >
        {#if $isLaunching}
          <Icon name="refresh" size="md" forceType="svg" className="spin" />
          <span>{"Launching..."}</span>
        {:else}
          <Icon name="play" size="md" forceType="svg" />
          <span>Play Minecraft</span>
        {/if}
      </button>
      {#if lastPlayedInstallations.length === 0}
        <p class="no-installations">
          No installations found. Please check your Minecraft directory in
          settings.
        </p>
      {/if}
      {#if launchStatus}
        <p
          class="launch-status"
          class:error={launchStatus.includes("fail") ||
            launchStatus.includes("error")}
        >
          {launchStatus}
        </p>
      {/if}
    </div>

    <!-- RAM Allocation Controls (Bottom Right) -->
    <div class="ram-controls">
      <div class="ram-header">
        <span class="installation-name">
          {lastPlayedInstallations.length > 0
            ? lastPlayedInstallations[0].name
            : "No Installation"}
        </span>
        <span class="ram-display">{formatRamDisplay(ramAllocation)}</span>
      </div>

      <div class="ram-inputs">
        <!-- Slider Input -->
        <div class="ram-slider-container">
          <input
            type="range"
            class="ram-slider"
            bind:value={ramAllocation}
            on:input={() => { updateRamFromSlider(); isEditingRam = true; }}
            on:change={() => { commitRamChange(); }}
            min="512"
            max="32768"
            step="256"
          />
          <div class="slider-labels">
            <span>512MB</span>
            <span>8GB</span>
            <span>16GB</span>
            <span>32GB</span>
          </div>
        </div>

        <!-- Text Input -->
        <div class="ram-text-container">
          <input
            type="text"
            class="ram-input"
            bind:value={ramInputValue}
            on:focus={() => { isEditingRam = true; }}
            on:blur={() => {
              updateRamFromInput();
              // commit immediately on blur and let commitRamChange manage isEditingRam
              commitRamChange(true);
            }}
            on:keydown={(e) => {
              if (e.key === "Enter") {
                updateRamFromInput();
                // commit immediately on Enter
                commitRamChange(true);
              }
            }}
            placeholder="2048"
          />
          <span class="ram-unit">MB</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/variables" as *;
.page-wrapper {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--container);
  overflow: hidden;
  border-radius: var(--border-radius);
}

.advertisement-banner {
  background: transparent !important;
  position: relative;
  height: 250px;
  margin: -1.5rem;
  border-radius: 1rem;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  margin-bottom: 0.5rem;

  .banner-background {
    position: absolute;
    inset: 0;

    :global(.banner-image) {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
  }

  .banner-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0.3) 0%,
      rgba(0, 0, 0, 0.5) 100%
    );
  }

  .banner-content {
    position: relative;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1;

    .banner-actions {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 1.5rem;
      width: 100%;
      max-width: 1200px;
      padding: 0 2rem;
      justify-items: center;

      .banner-button {
        font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif !important;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;
        padding: 0.5rem 1rem;
        border-radius: 0.5rem;
        font-weight: 900;
        text-decoration: none;
        transition: all 0.2s ease;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        backdrop-filter: blur(2px);
        border: none;
        cursor: pointer;
        font-size: 1rem;
        max-width: fit-content;

        :global(.button-image) {
          flex-shrink: 0;
          height: 40px;
          width: auto;
          object-fit: contain;
        }

        &:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        }

        &:active {
          transform: translateY(0);
        }

        &.primary {
          background: #230f2c;
          color: var(--text);

          &:hover {
            background: rgba(#230f2c, 0.7);
          }
        }

        &.secondary {
          background: #db8b12;
          color: var(--text);

          &:hover {
            background: rgba(#db8b12, 0.7);
          }
        }

        &.kofi {
          background: #13c3a8;
          color: var(--text);
          border: 1px solid color-mix(in srgb, #13c3a8, 70%, transparent);

          &:hover {
            background: rgba(#13c3a8, 0.7);
          }
        }

        span {
          white-space: nowrap;
        }
      }
    }

    .artist-recruitment {
      position: absolute;
      bottom: 1rem;
      left: 2rem;
      font-size: 0.75rem;
      color: color-mix(in srgb, var(--text), 70%, transparent);
      display: flex;
      align-items: center;
      gap: 0.25rem;

      .recruitment-text {
        font-weight: 400;
      }

      .recruitment-link {
        background: none;
        border: none;
        color: var(--text);
        text-decoration: underline;
        cursor: pointer;
        font-size: 0.75rem;
        font-weight: 500;
        padding: 0;
        transition: color 0.2s ease;

        &:hover {
          color: var(--primary);
        }
      }
    }
  }
}

.installations-section {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 2rem;
  margin-bottom: -2rem;
  z-index: 1;
}

.bottom-controls {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  padding: 1.5rem 2rem;
  background: linear-gradient(
    to top,
    var(--container) 40%,
    color-mix(in srgb, var(--container), 60%, transparent) 80%,
    color-mix(in srgb, var(--container), 15%, transparent) 90%,
    transparent 100%
  );
  backdrop-filter: blur(0.125rem);
  flex-shrink: 0;
  position: relative;
  z-index: 10;
  min-height: 5rem;
  z-index: 2;
}

.play-section {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.ram-controls {
  position: absolute;
  bottom: 0;
  right: 0;
  margin: 0;
  padding: 1rem;
  background: var(--card);
  border: 1px solid var(--dark-600);
  border-radius: 0.5rem;
  max-width: 23.75rem;
  min-width: 20rem;

  .ram-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text);

    .installation-name {
      color: var(--text);
      font-weight: 500;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
      margin-right: 0.5rem;
    }

    .ram-display {
      color: var(--primary);
      font-weight: 600;
      flex-shrink: 0;
    }
  }

  .ram-inputs {
    display: flex;
    gap: 0.75rem;
    align-items: center;

    .ram-slider-container {
      flex: 1;

      .ram-slider {
        width: 100%;
        height: 0.25rem;
        border-radius: 0.125rem;
        outline: none;
        appearance: none;
        cursor: pointer;

        &::-webkit-slider-thumb {
          appearance: none;
          width: 1rem;
          height: 1rem;
          background: var(--primary);
          border-radius: 50%;
          cursor: pointer;
          transition: all 0.2s ease;

          &:hover {
            background: var(--primary-600);
            transform: scale(1.1);
          }
        }

        &::-moz-range-thumb {
          width: 1rem;
          height: 1rem;
          background: var(--primary);
          border-radius: 50%;
          border: none;
          cursor: pointer;
          transition: all 0.2s ease;

          &:hover {
            background: var(--primary-600);
            transform: scale(1.1);
          }
        }
      }

      .slider-labels {
        display: flex;
        justify-content: space-between;
        margin-top: 0.25rem;
        font-size: 0.625rem;
        color: var(--placeholder);
      }
    }

    .ram-text-container {
      display: flex;
      align-items: center;
      gap: 0.375rem;
      flex-shrink: 0;

      .ram-input {
        width: 3.75rem;
        padding: 0.375rem 0.5rem;
        background: var(--dark-600);
        border: 1px solid var(--dark-500);
        border-radius: 0.25rem;
        color: var(--text);
        font-size: 0.75rem;
        text-align: center;
        transition: border-color 0.2s ease;

        &:focus {
          outline: none;
          border-color: var(--primary);
        }

        &::placeholder {
          color: var(--placeholder);
        }
      }

      .ram-unit {
        font-size: 0.75rem;
        color: var(--placeholder);
        font-weight: 500;
      }
    }
  }
}

.play-button {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 2rem;
  background: var(--primary);
  color: var(--text-white);
  border: none;
  border-radius: 0.75rem;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 20rem;
  justify-content: center;

  &:hover:not(:disabled) {
    background: var(--primary-600);
    transform: translateY(-0.125rem);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }
}

.no-installations {
  margin: 1rem 0 0;
  color: var(--placeholder);
  font-size: 0.875rem;
}

.launch-status {
  margin: 1rem 0 0;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  background: color-mix(in srgb, var(--green), 10%, transparent);
  color: var(--green);
  border: 1px solid color-mix(in srgb, var(--green), 30%, transparent);

  &.error {
    background: color-mix(in srgb, var(--red), 10%, transparent);
    color: var(--red);
    border-color: color-mix(in srgb, var(--red), 30%, transparent);
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes dropdownSlide {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

// Responsive design
@media (max-width: 768px) {
  .play-section {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .play-button {
    min-width: auto;
    width: 100%;
    max-width: 300px;
  }
}
</style>
