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
import { onMount } from "svelte";
import { AdvancedSettingsUI, AppearanceSettingsUI, ContentSettingsUI, GeneralSettingsUI, LoggingSettingsUI, MiscSettingsUI, NetworkSettingsUI } from ".";

const sections = ["general", "appearance", "logging", "content", "network", "advanced", "misc"] as const;

let currentSection = $state<(typeof sections)[number]>("general");
let miniNavElement = $state<HTMLElement | null>(null);
let settingsElement = $state<HTMLDivElement | null>(null);
let miniNavWidth = $state(0);

let resizeObserver: ResizeObserver | null = null;
let sectionObserver: IntersectionObserver | null = null;

function scrollToSection(section: (typeof sections)[number]) {
  const element = document.getElementById(section);
  if (!element || !settingsElement) return;

  currentSection = section;

  const top = element.offsetTop - settingsElement.offsetTop;

  settingsElement.scrollTo({
    top,
    behavior: "smooth",
  });
}

onMount(() => {
  if (miniNavElement) {
    miniNavWidth = miniNavElement.offsetWidth;

    if ("ResizeObserver" in window) {
      resizeObserver = new ResizeObserver(() => {
        if (miniNavElement) {
          miniNavWidth = miniNavElement.offsetWidth;
        }
      });

      resizeObserver.observe(miniNavElement);
    }
  }

  if (!settingsElement) return;

  const sectionElements = sections.map((section) => document.getElementById(section)).filter((element): element is HTMLElement => element !== null);

  sectionObserver = new IntersectionObserver(
    (entries) => {
      const visibleSections = entries.filter((entry) => entry.isIntersecting).sort((a, b) => b.intersectionRatio - a.intersectionRatio);

      if (visibleSections.length > 0) {
        const section = visibleSections[0].target.id;

        if (sections.includes(section as (typeof sections)[number])) {
          currentSection = section as (typeof sections)[number];
        }
      }
    },
    {
      root: settingsElement,
      threshold: [0.1, 0.25, 0.5, 0.75],
      rootMargin: "-5% 0px -60% 0px",
    },
  );

  for (const element of sectionElements) {
    sectionObserver.observe(element);
  }

  return () => {
    sectionObserver?.disconnect();
    resizeObserver?.disconnect();

    sectionObserver = null;
    resizeObserver = null;
  };
});
</script>

<div class="settings-content" style={`--mini-nav-width: ${miniNavWidth}px`}>
  <nav class="mini-nav" bind:this={miniNavElement} aria-label="Settings sections">
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
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 2rem;
  width: 100%;
  max-height: 80vh;
  min-height: 0;
}

.mini-nav {
  position: fixed;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  flex: 0 0 auto;
  min-width: fit-content;
  align-self: center;
  transform: translateY(-40%);
}

.mini-nav a {
  color: var(--tertiary);
  text-decoration: none;
  position: relative;
  padding-bottom: 2px;
  transition: all 0.4s ease;

  &.active {
    color: var(--primary);
  }

  &:hover {
    transform: scale(1.15) translateY(-0.15rem) translateX(0.15rem);
  }

  &::before {
    content: "";
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 2px;
    background: var(--tertiary);
    border-radius: 2px;
    transform: scaleX(0);
    transform-origin: left;
    transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 1;
  }

  &:hover::before {
    transform: scaleX(1);
  }

  &.active::before {
    background: var(--primary);
  }
}

.settings {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  flex: 1 1 0;
  min-width: 0;
  max-height: 100%;
  overflow-y: auto;
  margin-left: calc(var(--mini-nav-width, 120px) + 2rem);
}

.settings > section {
  scroll-margin-top: 1rem;
}
</style>
