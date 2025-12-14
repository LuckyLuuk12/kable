<script lang="ts">
import "$lib/styles/global.scss";
import { NavBar, TitleBar } from "$lib";
import { onMount, onDestroy } from "svelte";
import { get } from "svelte/store";
import {
  getSelectedCssTheme,
  loadCustomCss,
  settings,
  DiscordService,
} from "$lib";
import { soundService } from "$lib/services/SoundService";
import { page } from "$app/stores";

let customCSSLoaded = false;
let currentThemeName = "";
let unsubscribeSettings: (() => void) | null = null;
let unsubscribePage: (() => void) | null = null;

onMount(async () => {
  await loadCustomCSS();

  // Subscribe to settings changes instead of polling
  unsubscribeSettings = settings.subscribe(async ($settings) => {
    // Check if theme has changed in the settings
    const newTheme = $settings?.appearance?.selected_css_theme || "";
    if (newTheme !== currentThemeName) {
      console.log("Theme changed from", currentThemeName, "to", newTheme);
      await reloadCustomCSS();
    }
  });

  // Subscribe to route changes for Discord RPC
  unsubscribePage = page.subscribe(($page) => {
    if ($page?.url?.pathname) {
      DiscordService.updateBrowsing($page.url.pathname).catch((err) =>
        console.error("Failed to update Discord status:", err),
      );
    }
  });
});

// Clean up subscription on component destroy
onDestroy(() => {
  if (unsubscribeSettings) {
    unsubscribeSettings();
    unsubscribeSettings = null;
  }
  if (unsubscribePage) {
    unsubscribePage();
    unsubscribePage = null;
  }
});

async function loadCustomCSS() {
  try {
    // Get the selected CSS theme from settings store first, fallback to API
    const currentSettings = get(settings);
    let themeName = currentSettings?.appearance?.selected_css_theme || "";

    // If not in store, fallback to API
    if (!themeName) {
      themeName = (await getSelectedCssTheme()) || "";
    }

    currentThemeName = themeName;

    if (themeName && themeName !== "default") {
      // Load the CSS content for the theme
      const customCSS = await loadCustomCss(themeName);

      if (customCSS && typeof customCSS === "string") {
        injectCustomCSS(customCSS);
        customCSSLoaded = true;
        console.log("Custom CSS theme loaded successfully:", themeName);
      }
    } else {
      // Default theme selected - remove any existing custom CSS
      removeCustomCSS();
      customCSSLoaded = false;
      console.log("Default theme selected, custom CSS removed");
    }
  } catch (error) {
    // No custom CSS theme or error loading - that's fine
    console.log("No custom CSS theme found or error loading:", error);
    removeCustomCSS();
    customCSSLoaded = false;
  }
}

function injectCustomCSS(cssContent: string) {
  // First, remove any existing custom CSS
  removeCustomCSS();

  // Create a new style element
  const styleElement = document.createElement("style");
  styleElement.type = "text/css";
  styleElement.id = "user-custom-css";
  styleElement.innerHTML = cssContent;

  // Append to head (this ensures it comes after our compiled SCSS)
  document.head.appendChild(styleElement);

  // Force font loading and DOM reflow
  setTimeout(() => {
    // Trigger a reflow to ensure fonts are applied
    document.body.style.fontFamily =
      document.body.style.fontFamily ??
      '"Open Sans", Tahoma, Geneva, sans-serif';
    console.log("Font loading triggered: ", document.body.style.fontFamily);
  }, 100);
}

function removeCustomCSS() {
  // Remove existing custom CSS
  const existingStyle = document.getElementById("user-custom-css");
  if (existingStyle) {
    existingStyle.remove();
    // Trigger a reflow to ensure fonts are applied
    document.body.style.fontFamily =
      document.body.style.fontFamily ??
      '"Open Sans", Tahoma, Geneva, sans-serif';
    console.log(
      "Previous custom CSS removed: ",
      document.body.style.fontFamily,
    );
  }
}

// Function to reload custom CSS (useful for settings)
async function reloadCustomCSS() {
  // Remove existing custom CSS first
  removeCustomCSS();

  // Reload with new theme
  await loadCustomCSS();
}

// Make reload function available globally for settings page
if (typeof window !== "undefined") {
  (window as any).reloadCustomCSS = reloadCustomCSS;
}
</script>

<TitleBar>
  <NavBar>
    <slot />
  </NavBar>
</TitleBar>
