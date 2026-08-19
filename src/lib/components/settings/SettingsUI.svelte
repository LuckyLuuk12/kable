<!-- @component
SettingsUI - Main settings interface with tabbed navigation

Container component that manages all settings panels with smooth scrolling
navigation and responsive mini-nav sidebar.

@example
```svelte
◄SettingsUI /►
```
-->
<script lang="ts">
import { AdvancedSettingsUI, AppearanceSettingsUI, ContentSettingsUI, GeneralSettingsUI, LoggingSettingsUI, MiscSettingsUI, NetworkSettingsUI } from ".";

const sections = ["general", "appearance", "logging", "content", "network", "advanced", "misc"] as const;

type Section = (typeof sections)[number];

let currentSection = $state<Section>("general");
let settingsElement = $state<HTMLDivElement | null>(null);

function scrollToSection(section: Section) {
  const element = document.getElementById(section);
  if (!element || !settingsElement) return;

  currentSection = section;

  const top = element.getBoundingClientRect().top - settingsElement.getBoundingClientRect().top + settingsElement.scrollTop;

  settingsElement.scrollTo({
    top,
    behavior: "smooth",
  });
}

$effect(() => {
  const element = settingsElement;
  if (!element) return;

  const updateCurrentSection = () => {
    const containerTop = element.getBoundingClientRect().top;

    let closestSection: Section = sections[0];
    let closestDistance = Infinity;

    for (const section of sections) {
      const sectionElement = document.getElementById(section);
      if (!sectionElement) continue;

      const distance = Math.abs(sectionElement.getBoundingClientRect().top - containerTop);

      if (distance < closestDistance) {
        closestDistance = distance;
        closestSection = section;
      }
    }

    currentSection = closestSection;
  };

  element.addEventListener("scroll", updateCurrentSection, {
    passive: true,
  });

  updateCurrentSection();

  return () => {
    element.removeEventListener("scroll", updateCurrentSection);
  };
});
</script>

<div class="settings-content">
  <nav class="mini-nav" aria-label="Settings sections">
    {#each sections as section (section)}
      <a
        href={`#${section}`}
        class:active={currentSection === section}
        aria-current={currentSection === section ? "page" : undefined}
        onclick={(event) => {
          event.preventDefault();
          scrollToSection(section);
        }}
      >
        {section.charAt(0).toUpperCase() + section.slice(1)}
      </a>
    {/each}
  </nav>

  <div class="settings" bind:this={settingsElement}>
    <section id="general">
      <GeneralSettingsUI />
    </section>

    <section id="appearance">
      <AppearanceSettingsUI />
    </section>

    <section id="logging">
      <LoggingSettingsUI />
    </section>

    <section id="content">
      <ContentSettingsUI />
    </section>

    <section id="network">
      <NetworkSettingsUI />
    </section>

    <section id="advanced">
      <AdvancedSettingsUI />
    </section>

    <section id="misc">
      <MiscSettingsUI />
    </section>
  </div>
</div>

<style lang="scss">
.settings-content {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 2rem;

  width: 100%;
  height: calc(100vh - 3 * $space-md);
  min-height: 0;
  padding: $space-md;

  overflow: hidden;
}

.mini-nav {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;

  align-self: center;

  min-width: 9rem;
  padding: 0.5rem;

  border: 1px solid $color-border-muted;
  border-radius: $radius-xl;

  background: color-mix(in srgb, $color-surface-1 70%, transparent);
  box-shadow: 0 0.25rem 1rem rgba(0, 0, 0, 0.06);
}

.mini-nav a {
  position: relative;

  display: flex;
  align-items: center;

  min-height: 2.25rem;
  padding: 0.5rem 0.75rem 0.5rem 1rem;

  border-radius: $radius-md;

  color: $color-text-muted;
  text-decoration: none;

  font-size: 0.875rem;
  font-weight: 500;

  transition:
    background 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;

  &::before {
    content: "";

    position: absolute;
    left: 0.35rem;
    top: 50%;

    width: 0.2rem;
    height: 0.4rem;

    border-radius: 999px;

    background: $color-accent;

    opacity: 0;
    transform: translateY(-50%) scaleY(0.5);

    transition:
      opacity 0.2s ease,
      height 0.2s ease,
      transform 0.2s ease;
  }

  &:hover {
    color: $color-text;
    background: $color-surface-2;
    transform: translateX(0.15rem);
  }

  &.active {
    color: $color-accent;

    background: color-mix(in srgb, $color-accent 10%, $color-surface-1);

    &::before {
      height: 1.25rem;
      opacity: 1;
      transform: translateY(-50%) scaleY(1);
    }
  }

  &:focus-visible {
    outline: none;

    box-shadow:
      0 0 0 2px $color-focus,
      0 0 0 4px color-mix(in srgb, $color-focus 20%, transparent);
  }
}

.settings {
  display: flex;
  flex-direction: column;
  gap: 1rem;

  min-width: 0;
  min-height: 0;

  overflow-y: auto;
  overflow-x: hidden;
}

.settings > section {
  scroll-margin-top: 1rem;
}
</style>
