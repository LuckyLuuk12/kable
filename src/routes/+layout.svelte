<script lang="ts">
import { page } from "$app/state";
import { app, debug, error, log, NavBar, TitleBar } from "$lib";
import ModalHost from "$lib/components/ModalHost.svelte";
import "$lib/styles/global.scss";
import "@fortawesome/fontawesome-free/css/all.min.css";
import { onMount, untrack } from "svelte";

let { children } = $props();

onMount(() => {
  const observer = new MutationObserver(() => {
    const style = document.getElementById("user-custom-css");

    debug("[CSS DEBUG] DOM state", {
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
  log("Starting layout initialization...");

  try {
    await app.initAll();
    log("Layout initialization complete");
  } catch (err) {
    error("Tauri initialization error:", err);
  }
});

// Update Discord RPC whenever the current page changes.
$effect(() => {
  const pathname = page?.url?.pathname;
  if (!pathname) return;
  untrack(() => {
    debug("Updating Discord status for path:", pathname);
    app.discordService.setBrowsing(pathname).catch((err) => error("Failed to update Discord status:", err));
  });
});
</script>

<TitleBar>
  <NavBar>
    {@render children()}
  </NavBar>
</TitleBar>

<ModalHost />
