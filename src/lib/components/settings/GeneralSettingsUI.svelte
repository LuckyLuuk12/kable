<!-- @component
GeneralSettingsUI - General launcher settings panel

Core launcher configuration including all fields of:
```
export type GeneralSettings = {
  java_path?: string | null;
  game_directory?: string | null;
  on_game_close?: OnGameAction;
  on_game_crash?: OnGameAction;
  on_game_launch?: OnGameAction;
  update_mode?: UpdateMode;
  update_detection?: UpdateDetection;
  update_notification_style?: UpdateNotificationStyle;
};
```

@example
```svelte
◄GeneralSettingsUI /►
```
-->
<script lang="ts">
import { AutoUpdater, app, type GeneralSettings, type OnGameAction, type UpdateDetection, type UpdateMode, type UpdateNotificationStyle } from "$lib";
import { onMount } from "svelte";

let isWideScreen = $state(true);
let detectedJavaPath = $state("");

function checkScreen() {
  isWideScreen = window.innerWidth >= 700;
}

function updateSetting<K extends keyof GeneralSettings>(field: K, value: GeneralSettings[K]) {
  const general = app.customizationService.settings?.general;
  if (!general) return;

  app.customizationService.settings!.general = {
    ...general,
    [field]: value,
  };

  app.customizationService.scheduleSave();
}

function getActionValue(event: Event): OnGameAction {
  const value = (event.currentTarget as HTMLSelectElement).value;

  switch (value) {
    case "ask":
    case "exit":
    case "minimize":
    case "minimize_to_tray":
    case "nothing":
    case "restart":
      return value;
    default:
      return { open: value };
  }
}

function getUpdateDetectionValue(event: Event): UpdateDetection {
  const value = (event.currentTarget as HTMLSelectElement).value;

  switch (value) {
    case "on_startup":
    case "on_close":
    case "manual":
      return value;
    default:
      return "manual";
  }
}

function getUpdateModeValue(event: Event): UpdateMode {
  return (event.currentTarget as HTMLSelectElement).value as UpdateMode;
}

function getUpdateNotificationStyleValue(event: Event): UpdateNotificationStyle {
  return (event.currentTarget as HTMLSelectElement).value as UpdateNotificationStyle;
}

onMount(() => {
  checkScreen();
  window.addEventListener("resize", checkScreen);

  app.launcherService
    .autoDetectJava()
    .then((path) => {
      detectedJavaPath = path;
    })
    .catch((error) => {
      console.warn("Failed to auto-detect Java:", error);
      detectedJavaPath = "Java not found";
    });

  return () => window.removeEventListener("resize", checkScreen);
});
</script>

<div class="settings-tab">
  <h2>General Settings</h2>
  <p>Configure general settings for the application.</p>

  <form>
    <div class="setting-item">
      <div class="setting-info">
        <label for="java-path">Java Path</label>
        <p class="setting-description">Path to the Java executable</p>
      </div>
      <div class="setting-control">
        <input
          type="text"
          id="java-path"
          value={app.customizationService.settings?.general?.java_path ?? ""}
          placeholder={detectedJavaPath || "Path to Java executable"}
          onchange={(event) => updateSetting("java_path", (event.currentTarget as HTMLInputElement).value || null)}
        />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="game-directory">Game Directory</label>
        <p class="setting-description">Path to your .minecraft folder</p>
      </div>
      <div class="setting-control">
        <input
          type="text"
          id="game-directory"
          value={app.customizationService.settings?.general?.game_directory ?? ""}
          placeholder="C:/Users/user/AppData/Roaming/.minecraft"
          onchange={(event) => updateSetting("game_directory", (event.currentTarget as HTMLInputElement).value || null)}
        />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="on-game-close">On Game Close</label>
        <p class="setting-description">What should happen when the game closes?</p>
      </div>

      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={JSON.stringify(app.customizationService.settings?.general?.on_game_close) === JSON.stringify({ open: "logs" })}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="logs"
              />
              Open Logs
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={JSON.stringify(app.customizationService.settings?.general?.on_game_close) === JSON.stringify({ open: "home" })}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="home"
              />
              Open Home
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={app.customizationService.settings?.general?.on_game_close === "exit"}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="exit"
              />
              Exit Application
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={app.customizationService.settings?.general?.on_game_close === "minimize"}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="minimize"
              />
              Minimize
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={app.customizationService.settings?.general?.on_game_close === "minimize_to_tray"}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="minimize_to_tray"
              />
              Minimize to Tray
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={app.customizationService.settings?.general?.on_game_close === "nothing"}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="nothing"
              />
              Do Nothing
            </label>

            <label>
              <input
                type="radio"
                name="on-game-close"
                checked={app.customizationService.settings?.general?.on_game_close === "ask"}
                onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
                value="ask"
              />
              Ask
            </label>
          </div>
        {:else}
          <select
            id="on-game-close"
            value={JSON.stringify(app.customizationService.settings?.general?.on_game_close)}
            onchange={(event) => updateSetting("on_game_close", getActionValue(event))}
          >
            <option value={JSON.stringify({ open: "logs" })}>Open Logs</option>
            <option value={JSON.stringify({ open: "home" })}>Open Home</option>
            <option value="exit">Exit Application</option>
            <option value="minimize">Minimize</option>
            <option value="minimize_to_tray">Minimize to Tray</option>
            <option value="nothing">Do Nothing</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="">On Game Crash</label>
        <p class="setting-description">What should happen when the game crashes?</p>
      </div>

      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "restart"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="restart"
              />
              Restart Game
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={JSON.stringify(app.customizationService.settings?.general?.on_game_crash) === JSON.stringify({ open: "logs" })}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="logs"
              />
              Open Logs
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={JSON.stringify(app.customizationService.settings?.general?.on_game_crash) === JSON.stringify({ open: "home" })}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="home"
              />
              Open Home
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "exit"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="exit"
              />
              Exit Application
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "minimize"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="minimize"
              />
              Minimize
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "minimize_to_tray"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="minimize_to_tray"
              />
              Minimize to Tray
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "nothing"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="nothing"
              />
              Do Nothing
            </label>

            <label>
              <input
                type="radio"
                name="on-game-crash"
                checked={app.customizationService.settings?.general?.on_game_crash === "ask"}
                onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
                value="ask"
              />
              Ask
            </label>
          </div>
        {:else}
          <select
            id="on-game-crash"
            value={JSON.stringify(app.customizationService.settings?.general?.on_game_crash)}
            onchange={(event) => updateSetting("on_game_crash", getActionValue(event))}
          >
            <option value="restart">Restart Game</option>
            <option value={JSON.stringify({ open: "logs" })}>Open Logs</option>
            <option value={JSON.stringify({ open: "home" })}>Open Home</option>
            <option value="exit">Exit Application</option>
            <option value="minimize">Minimize</option>
            <option value="minimize_to_tray">Minimize to Tray</option>
            <option value="nothing">Do Nothing</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label>On Game Launch</label>
        <p class="setting-description">What should happen when the game launches?</p>
      </div>

      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={app.customizationService.settings?.general?.on_game_launch === "nothing"}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="nothing"
              />
              Keep Application Open
            </label>

            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={app.customizationService.settings?.general?.on_game_launch === "exit"}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="exit"
              />
              Exit Application
            </label>

            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={JSON.stringify(app.customizationService.settings?.general?.on_game_launch) === JSON.stringify({ open: "logs" })}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="logs"
              />
              Open Logs
            </label>

            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={app.customizationService.settings?.general?.on_game_launch === "minimize"}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="minimize"
              />
              Minimize
            </label>

            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={app.customizationService.settings?.general?.on_game_launch === "minimize_to_tray"}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="minimize_to_tray"
              />
              Minimize to Tray
            </label>

            <label>
              <input
                type="radio"
                name="on-game-launch"
                checked={app.customizationService.settings?.general?.on_game_launch === "ask"}
                onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
                value="ask"
              />
              Ask
            </label>
          </div>
        {:else}
          <select
            id="on-game-launch"
            value={JSON.stringify(app.customizationService.settings?.general?.on_game_launch)}
            onchange={(event) => updateSetting("on_game_launch", getActionValue(event))}
          >
            <option value="nothing">Keep Application Open</option>
            <option value="exit">Exit Application</option>
            <option value={JSON.stringify({ open: "logs" })}>Open Logs</option>
            <option value="minimize">Minimize</option>
            <option value="minimize_to_tray">Minimize to Tray</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="update-detection">Update Detection</label>
        <p class="setting-description">When should the launcher check for updates?</p>
      </div>

      <div class="setting-control">
        <select
          id="update-detection"
          value={typeof app.customizationService.settings?.general?.update_detection === "string"
            ? app.customizationService.settings?.general.update_detection
            : "manual"}
          onchange={(event) => updateSetting("update_detection", getUpdateDetectionValue(event))}
        >
          <option value="manual">Manual</option>
          <option value="on_startup">On Startup</option>
          <option value="on_close">On Close</option>
        </select>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="update-mode">Update Mode</label>
        <p class="setting-description">Choose how updates should be applied: install automatically, manually handle updates, or ask for confirmation.</p>
      </div>

      <div class="setting-control">
        <select
          id="update-mode"
          value={app.customizationService.settings?.general?.update_mode ?? "manual"}
          onchange={(event) => updateSetting("update_mode", getUpdateModeValue(event))}
        >
          <option value="automatic">Automatic</option>
          <option value="manual">Manually</option>
          <option value="on_confirm">Update on Confirm</option>
        </select>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="update-notification-style">Update Notification Style</label>
        <p class="setting-description">Choose how update notifications are displayed.</p>
      </div>

      <div class="setting-control">
        <select
          id="update-notification-style"
          value={app.customizationService.settings?.general?.update_notification_style ?? "notification"}
          onchange={(event) => updateSetting("update_notification_style", getUpdateNotificationStyleValue(event))}
        >
          <option value="notification">Notification Toast</option>
          <option value="modal">Modal Dialog</option>
        </select>
      </div>
    </div>
  </form>

  <AutoUpdater />
</div>

<style lang="scss">
.settings-tab {
  width: 100%;

  display: flex;
  flex-direction: column;
  gap: 1.75rem;

  padding: 2rem 2.5rem;
  margin-bottom: 2rem;

  background: $color-surface-1;
  border: 1px solid $color-border-muted;
  border-radius: $radius-2xl;
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
}

.settings-tab h2 {
  margin: 0;

  color: $color-text;
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0.02em;

  background: linear-gradient(to right, $color-accent, $color-accent-secondary);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.settings-tab > p {
  margin: -1rem 0 0;

  color: $color-text-muted;
  font-size: 0.95rem;
  line-height: 1.5;
}

form {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 2.5rem;

  min-width: 0;
  padding: 1.25rem 0;

  border-bottom: 1px solid $color-border-muted;

  &:last-child {
    border-bottom: none;
  }
}

.setting-info {
  flex: 1 1 16rem;
  min-width: 13rem;

  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.setting-info label {
  margin: 0;

  color: $color-text;
  font-size: 1rem;
  font-weight: 500;
  line-height: 1.4;
}

.setting-description {
  margin: 0;

  color: $color-text-muted;
  font-size: 0.875rem;
  line-height: 1.45;
}

.setting-control {
  flex: 1 1 18rem;
  min-width: 12rem;

  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.75rem;
}

.radio-group {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.6rem;
  width: 100%;
  max-width: 20rem;
}

.radio-group label {
  display: flex;
  align-items: center;
  gap: 0.6rem;

  width: 100%;

  color: $color-text;
  cursor: pointer;
}

.radio-group input[type="radio"] {
  appearance: none;

  width: 1rem;
  height: 1rem;
  flex: 0 0 1rem;

  margin: 0;

  border: 2px solid $color-surface-3;
  border-radius: 50%;

  background: $color-surface-2;

  cursor: pointer;

  transition:
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    border-color: $color-accent-muted;
  }

  &:checked {
    border-color: $color-accent;
    background: $color-accent;
    box-shadow: inset 0 0 0 3px $color-surface-3;
  }

  &:focus-visible {
    outline: none;
    box-shadow:
      inset 0 0 0 3px $color-surface-3,
      0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

/* Inputs */

input[type="text"],
select {
  width: 100%;
  max-width: 20rem;

  padding: 0.55rem 0.8rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.9rem;

  outline: none;

  transition:
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    border-color: $color-accent-muted;
    background: $color-surface-3;
  }

  &:focus {
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

select {
  cursor: pointer;
}

/* Radio buttons */

input[type="radio"] {
  width: 1rem;
  height: 1rem;

  margin: 0;

  accent-color: $color-accent;
  cursor: pointer;
}

/* Checkbox */

input[type="checkbox"] {
  width: 1.1rem;
  height: 1.1rem;

  margin: 0;

  accent-color: $color-accent;
  cursor: pointer;
}

/* Buttons */

button {
  padding: 0.55rem 0.9rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.875rem;
  font-weight: 500;

  cursor: pointer;

  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    transform 0.15s ease;

  &:hover {
    background: $color-surface-3;
    border-color: $color-accent;
    color: $color-accent;
  }

  &:active {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: none;
    border-color: $color-focus;
    box-shadow: 0 0 0 3px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

/* Responsive */

@media (max-width: 700px) {
  .settings-tab {
    padding: 1.5rem;
  }

  .setting-item {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }

  .setting-info {
    min-width: 0;
  }

  .setting-control {
    width: 100%;
    min-width: 0;

    justify-content: flex-start;
  }

  input[type="text"],
  select {
    max-width: none;
  }
}
</style>
