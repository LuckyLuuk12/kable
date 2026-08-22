<script lang="ts">
import { page } from "$app/state";
import { app, NavBar, TitleBar } from "$lib";
import ModalHost from "$lib/components/ModalHost.svelte";
import "$lib/styles/global.scss";
import { onMount } from "svelte";

let { children } = $props();

let currentThemeName = app.customizationService.settings?.appearance?.selected_css_theme || "system";

onMount(async () => {
  // First initialize the app and load any necessary data
  console.log("Starting layout initialization...");
  try {
    await app.initAll();
    console.log("Layout initialization complete");
  } catch (error: any) {
    console.error("Tauri initialization error:", error);
  }
  // Load the custom CSS theme if one is selected
  const theme = app.customizationService.settings?.appearance?.selected_css_theme || "system";
  await app.customizationService.setTheme(theme);
});

// Use effect to listen for page changes and update Discord RPC accordingly
$effect(() => {
  if (!page?.url?.pathname) return;
  app.discordService.setBrowsing(page.url.pathname).catch((err) => console.error("Failed to update Discord status:", err));
});

// Reload theme CSS by listening with effect() to settings from app.customizationService
$effect(() => {
  const newTheme = app.customizationService.settings?.appearance?.selected_css_theme || "system";
  if (newTheme !== currentThemeName) {
    console.log("Theme changed from", currentThemeName, "to", newTheme);
    app.customizationService.setTheme(newTheme);
  }
});
</script>

<TitleBar>
  <NavBar>
    {@render children()}
  </NavBar>
</TitleBar>
<ModalHost />
