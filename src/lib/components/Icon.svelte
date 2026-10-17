<!-- @component
Icon - Flexible icon component supporting multiple icon types.

Uses CustomizationService.getIcon() to resolve icons from the active
icon template. Supports emoji, Font Awesome CSS classes, SVG, and
custom icon types.

@prop {string} name - The name/identifier of the icon to display
@prop {'sm' | 'md' | 'lg' | 'xl'} [size='md'] - Icon size
@prop {string} [className=''] - Additional CSS classes
@prop {string} [forceType] - Optional icon type override
-->
<script lang="ts">
import { app, type IconData, type IconType } from "$lib";

let {
  name,
  size = "md",
  className = "",
  forceType = undefined,
}: {
  name: string;
  size?: "sm" | "md" | "lg" | "xl";
  className?: string;
  forceType?: string;
} = $props();

type IconValue = IconData | string;

let iconData: IconValue | undefined = $state({
  icon: "❓",
  type: "emoji",
});

let requestVersion = 0;

const sizeClasses = {
  sm: "icon-sm",
  md: "icon-md",
  lg: "icon-lg",
  xl: "icon-xl",
} as const;

/**
 * Recognize Font Awesome class strings when the service returns a
 * string rather than a typed IconData object, or when legacy template
 * data incorrectly labels a Font Awesome string as an emoji.
 *
 * Supports Font Awesome 5 and 6 naming conventions.
 */
function isFontAwesomeClass(value: string): boolean {
  return /^(?:fa|fas|far|fab|fal|fat|fad|fass|fasr|fasl|fa-(?:solid|regular|brands|light|thin|duotone|sharp|semibold|sharp-solid|sharp-regular|sharp-light|sharp-thin|sharp-duotone))\s/.test(
    value.trim(),
  );
}

/**
 * Resolve the effective icon type.
 *
 * Typed data from CustomizationService is authoritative, except when
 * an icon is unmistakably an SVG or Font Awesome class string.
 */
function resolveIconType(data: IconValue | undefined): IconType | null {
  if (data == null) return null;

  const value = typeof data === "string" ? data : data.icon;
  const trimmed = value.trim();

  if (trimmed.startsWith("<svg")) {
    return "svg";
  }

  if (isFontAwesomeClass(trimmed)) {
    return "css_class";
  }

  if (typeof data !== "string" && data.type) {
    return data.type;
  }

  return "emoji";
}

/**
 * Validate SVG content before passing it to {@html}.
 *
 * This is a conservative sanity check, not a complete SVG sanitizer.
 * Icon templates should contain trusted SVG markup.
 */
function isValidSvg(content: string): boolean {
  const trimmed = content.trim();

  if (!/^<svg(?:\s|>)/i.test(trimmed) || !/<\/svg>\s*$/i.test(trimmed)) {
    return false;
  }

  const dangerousPatterns = [
    /<script\b/i,
    /<foreignObject\b/i,
    /<iframe\b/i,
    /<object\b/i,
    /<embed\b/i,
    /<link\b/i,
    /<meta\b/i,
    /<style\b/i,
    /javascript\s*:/i,
    /vbscript\s*:/i,
    /data\s*:\s*text\/html/i,
    /data\s*:\s*application\/javascript/i,
    /\bon\w+\s*=/i,
  ];

  return !dangerousPatterns.some((pattern) => pattern.test(trimmed));
}

/**
 * Re-resolve the icon when its name, forced type, or selected template
 * changes. The request counter prevents stale asynchronous results from
 * overwriting a newer icon.
 */
$effect(() => {
  const currentName = name;
  const currentForceType = forceType;

  // Establish a reactive dependency on the active template.
  const currentTemplate = app.customizationService.selectedIconTemplate;

  // The template value is intentionally read to trigger re-resolution.
  void currentTemplate;

  const version = ++requestVersion;

  async function loadIcon() {
    try {
      const result = await app.customizationService.getIcon(currentName, currentForceType);

      if (version !== requestVersion) return;

      iconData = result ?? {
        icon: "❓",
        type: "emoji",
      };
    } catch (error) {
      if (version !== requestVersion) return;

      console.error(`Failed to resolve icon "${currentName}":`, error);

      iconData = {
        icon: "❓",
        type: "emoji",
      };
    }
  }

  void loadIcon();

  return () => {
    // Invalidate this request if the effect reruns or is destroyed.
    if (version === requestVersion) {
      requestVersion++;
    }
  };
});

let type = $derived(resolveIconType(iconData));

let icon = $derived(typeof iconData === "string" ? iconData : (iconData?.icon ?? "❓"));

let validSvg = $derived(type === "svg" && isValidSvg(icon));

$effect(() => {
  if (type === "svg" && !validSvg) {
    console.warn(`Icon "${name}" contains invalid SVG markup; rendering it as text instead.`);
  }
});
</script>

{#if type === "emoji"}
  <span class="icon icon-emoji {sizeClasses[size]} {className}" role="img" aria-label={name}>{icon}</span>
{:else if type === "css_class"}
  <i class="icon icon-fontawesome {icon} {sizeClasses[size]} {className}" role="img" aria-label={name}></i>
{:else if type === "svg" && validSvg}
  <span class="icon icon-svg {sizeClasses[size]} {className}" role="img" aria-label={name}>
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html icon}
  </span>
{:else}
  <span class="icon icon-custom {sizeClasses[size]} {className}" role="img" aria-label={name}>{icon}</span>
{/if}

<style lang="scss">
.icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  line-height: 1;

  &.icon-sm {
    width: 1rem;
    height: 1rem;
    font-size: 0.875rem;
  }

  &.icon-md {
    width: 1.5rem;
    height: 1.5rem;
    font-size: 1.25rem;
  }

  &.icon-lg {
    width: 2rem;
    height: 2rem;
    font-size: 1.75rem;
  }

  &.icon-xl {
    width: 3rem;
    height: 3rem;
    font-size: 2.5rem;
  }

  &.icon-emoji {
    font-family: "Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji", sans-serif;
  }

  // Font Awesome owns its font family, weight, and glyph rendering.
  // This component only supplies the icon classes.

  &.icon-svg {
    color: currentColor;

    :global(svg) {
      display: block;
      width: 100%;
      height: 100%;
    }
  }

  &.icon-system {
    color: currentColor;
  }

  &.icon-custom {
    color: currentColor;
    font-family: inherit;
  }
}
</style>
