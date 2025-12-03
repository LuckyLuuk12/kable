<!-- @component
GeneralSettingsUI - General launcher settings panel

Core launcher configuration including Java paths, memory allocation,
game directory, and window behavior settings.

@example
```svelte
◄GeneralSettingsUI /►
```
-->
<script>
import { settings, AutoUpdater } from "$lib";
import { onMount } from "svelte";
import { autoDetectJava } from "$lib/api/launcher";

let isWideScreen = true;
let detectedJavaPath = "";

function checkScreen() {
  isWideScreen = window.innerWidth >= 700;
}

onMount(() => {
  checkScreen();
  window.addEventListener("resize", checkScreen);

  // Auto-detect Java path for placeholder
  autoDetectJava()
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
        <p class="setting-description">Path to Java executable</p>
      </div>
      <div class="setting-control">
        <input
          type="text"
          id="java-path"
          bind:value={$settings.general.java_path}
          placeholder={detectedJavaPath || "Path to Java executable"}
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
          bind:value={$settings.general.game_directory}
          placeholder="C:/Users/user/AppData/Roaming/.minecraft"
        />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>On Game Close</label>
        <p class="setting-description">
          What should happen when the game closes?
        </p>
      </div>
      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label
              ><input
                type="radio"
                name="on-game-close"
                value="open_logs"
                bind:group={$settings.general.on_game_close}
              /> Open Logs</label
            >
            <label
              ><input
                type="radio"
                name="on-game-close"
                value="open_home"
                bind:group={$settings.general.on_game_close}
              /> Open Home</label
            >
            <label
              ><input
                type="radio"
                name="on-game-close"
                value="exit"
                bind:group={$settings.general.on_game_close}
              /> Exit Application</label
            >
            <label
              ><input
                type="radio"
                name="on-game-close"
                value="minimize"
                bind:group={$settings.general.on_game_close}
              /> Minimize to Tray</label
            >
            <label
              ><input
                type="radio"
                name="on-game-close"
                value="ask"
                bind:group={$settings.general.on_game_close}
              /> Ask</label
            >
          </div>
        {:else}
          <select
            id="on-game-close"
            bind:value={$settings.general.on_game_close}
          >
            <option value="open_logs">Open Logs</option>
            <option value="open_home">Open Home</option>
            <option value="exit">Exit Application</option>
            <option value="minimize">Minimize to Tray</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>On Game Crash</label>
        <p class="setting-description">
          What should happen when the game crashes?
        </p>
      </div>
      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="restart"
                bind:group={$settings.general.on_game_crash}
              /> Restart Game</label
            >
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="open_logs"
                bind:group={$settings.general.on_game_crash}
              /> Open Logs</label
            >
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="open_home"
                bind:group={$settings.general.on_game_crash}
              /> Open Home</label
            >
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="exit"
                bind:group={$settings.general.on_game_crash}
              /> Exit Application</label
            >
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="minimize"
                bind:group={$settings.general.on_game_crash}
              /> Minimize to Tray</label
            >
            <label
              ><input
                type="radio"
                name="on-game-crash"
                value="ask"
                bind:group={$settings.general.on_game_crash}
              /> Ask</label
            >
          </div>
        {:else}
          <select
            id="on-game-crash"
            bind:value={$settings.general.on_game_crash}
          >
            <option value="restart">Restart Game</option>
            <option value="open_logs">Open Logs</option>
            <option value="open_home">Open Home</option>
            <option value="exit">Exit Application</option>
            <option value="minimize">Minimize to Tray</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label>On Game Launch</label>
        <p class="setting-description">
          What should happen when the game launches?
        </p>
      </div>
      <div class="setting-control">
        {#if isWideScreen}
          <div class="radio-group">
            <label
              ><input
                type="radio"
                name="on-game-launch"
                value="keep_open"
                bind:group={$settings.general.on_game_launch}
              /> Keep Application Open</label
            >
            <label
              ><input
                type="radio"
                name="on-game-launch"
                value="exit"
                bind:group={$settings.general.on_game_launch}
              /> Exit Application</label
            >
            <label
              ><input
                type="radio"
                name="on-game-launch"
                value="open_logs"
                bind:group={$settings.general.on_game_launch}
              /> Open Logs</label
            >
            <label
              ><input
                type="radio"
                name="on-game-launch"
                value="minimize"
                bind:group={$settings.general.on_game_launch}
              /> Minimize to Tray</label
            >
            <label
              ><input
                type="radio"
                name="on-game-launch"
                value="ask"
                bind:group={$settings.general.on_game_launch}
              /> Ask</label
            >
          </div>
        {:else}
          <select
            id="on-game-launch"
            bind:value={$settings.general.on_game_launch}
          >
            <option value="keep_open">Keep Application Open</option>
            <option value="exit">Exit Application</option>
            <option value="open_logs">Open Logs</option>
            <option value="minimize">Minimize to Tray</option>
            <option value="ask">Ask</option>
          </select>
        {/if}
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="auto-update-launcher">Auto Update Launcher</label>
        <p class="setting-description">
          Automatically check for launcher updates
        </p>
      </div>
      <div class="setting-control">
        <input
          type="checkbox"
          id="auto-update-launcher"
          bind:checked={$settings.general.auto_update_launcher}
        />
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <label for="show-ads">Show Ads</label>
        <p class="setting-description">
          Show ads in the launcher (no paid subscription required)
        </p>
      </div>
      <div class="setting-control">
        <input
          type="checkbox"
          id="show-ads"
          bind:checked={$settings.general.show_ads}
        />
      </div>
    </div>
  </form>
  <!-- Auto-updater section -->
  <AutoUpdater />
</div>

<style lang="scss">
@use "@kablan/clean-ui/scss/_variables.scss" as *;

.settings-tab {
  background: var(--container);
  border-radius: var(--border-radius-large);
  box-shadow: 0 0.125rem 0.5rem rgba(0, 0, 0, 0.08);
  padding: 2rem 2.5rem;
  margin-bottom: 2rem;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.settings-tab h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, $primary, $secondary);
  color: var(--text-transparent);
  background-clip: text;
  -webkit-background-clip: text;
  -moz-background-clip: text;
  letter-spacing: 0.02em;
}
form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}
.setting-item {
  display: flex;
  align-items: flex-start;
  gap: 2rem;
  padding: 1rem 0;
  border-bottom: 1px solid var(--dark-200);
}
.setting-item:last-child {
  border-bottom: none;
}
.setting-info {
  flex: 1 1 16.25rem;
  min-width: 13.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.setting-info label {
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--text);
  margin-bottom: 0.1rem;
}
.setting-description {
  font-size: 0.95rem;
  color: var(--placeholder);
  margin-bottom: 0.2rem;
  line-height: 1.4;
}
.setting-control {
  flex: 1 1 11.25rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  min-width: 10rem;
}
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
input[type="text"] {
  width: 100%;
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);
  color: var(--text);
}
select {
  font-size: 1rem;
  padding: 0.4em 0.8em;
  border-radius: var(--border-radius);
  border: 1px solid var(--dark-200);
  color: var(--text);
}
</style>
