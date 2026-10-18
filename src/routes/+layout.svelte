<script lang="ts">
import { page } from "$app/state";
import { app, NavBar, TitleBar } from "$lib";
import ModalHost from "$lib/components/ModalHost.svelte";
import "$lib/styles/global.scss";
import "@fortawesome/fontawesome-free/css/all.min.css";
import { onMount } from "svelte";

let { children } = $props();

onMount(() => {
  const observer = new MutationObserver(() => {
    const style = document.getElementById("user-custom-css");

    console.debug("[CSS DEBUG] DOM state", {
      exists: style !== null,
      length: style?.textContent.length ?? 0,
      selectedTheme: app.customizationService.selectedCssTheme,
      savedTheme: app.customizationService.settings?.appearance?.theme,
    });
  });

  observer.observe(document.head, {
    childList: true,
    subtree: true,
    characterData: true,
  });

  return () => observer.disconnect();
});

onMount(async () => {
  console.log("Starting layout initialization...");

  try {
    await app.initAll();
    console.log("Layout initialization complete");
  } catch (error) {
    console.error("Tauri initialization error:", error);
  }
});

// Update Discord RPC whenever the current page changes.
$effect(() => {
  const pathname = page?.url?.pathname;
  if (!pathname) return;

  app.discordService.setBrowsing(pathname).catch((error) => console.error("Failed to update Discord status:", error));
});
</script>

<TitleBar>
  <NavBar>
    {@render children()}
  </NavBar>
</TitleBar>

<ModalHost />
