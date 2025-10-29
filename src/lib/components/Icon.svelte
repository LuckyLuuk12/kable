<!--
@component
Icon - Flexible icon component supporting multiple icon types

Renders icons from various sources including emoji, FontAwesome, SVG, system icons, and CSS.
Automatically selects the appropriate icon type based on the IconService configuration.

@prop {string} name - The name/identifier of the icon to display
@prop {'sm' | 'md' | 'lg' | 'xl'} [size='md'] - Size of the icon
@prop {string} [className=''] - Additional CSS classes to apply
@prop {'emoji' | 'fontawesome' | 'svg' | 'system' | 'css' | null} [forceType=null] - Force a specific icon type

@example
```svelte
<Icon name="home" size="lg" />
<Icon name="settings" forceType="svg" className="custom-class" />
```
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { IconService, selectedTemplate } from '../services/IconService';

  export let name: string;
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let className: string = '';
  export let forceType: 'emoji' | 'fontawesome' | 'svg' | 'system' | 'css' | null = null;

  let iconData: { icon: string; type: string; fallback: string } = { icon: '❓', type: 'emoji', fallback: '❓' };

  // Size mappings
  const sizeClasses = {
    sm: 'icon-sm',
    md: 'icon-md', 
    lg: 'icon-lg',
    xl: 'icon-xl'
  };

  // Initialize icon system on mount
  onMount(async () => {
    await IconService.initialize();
    updateIcon();
  });

  // Update icon when name changes or when selected template changes
  $: if (name || $selectedTemplate) {
    updateIcon();
  }

  function updateIcon() {
    iconData = IconService.getIconWithFallback(name, forceType);
  }

  // Validate SVG content for security
  function isValidSvg(content: string): boolean {
    if (!content || typeof content !== 'string') return false;
    
    // Must start with <svg and end with </svg>
    if (!content.trim().startsWith('<svg') || !content.trim().endsWith('</svg>')) {
      return false;
    }
    
    // Check for potentially dangerous content
    const dangerousPatterns = [
      /<script/i,
      /javascript:/i,
      /on\w+\s*=/i, // onclick, onload, etc.
      /<iframe/i,
      /<object/i,
      /<embed/i,
      /<link/i,
      /<style/i,
      /<meta/i,
      /data:text\/html/i,
      /data:application\/javascript/i
    ];
    
    return !dangerousPatterns.some(pattern => pattern.test(content));
  }

  // Reactive statements for rendering
  $: type = iconData.type;
  $: icon = iconData.icon;
  
  // Log warning if SVG type but invalid content
  $: if (type === 'svg' && !isValidSvg(icon)) {
    console.warn(`Icon "${name}": Invalid or potentially unsafe SVG content detected, falling back to custom renderer`, icon.substring(0, 100));
  }
</script>

{#if type === 'emoji'}
  <span class="icon icon-emoji {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {icon}
  </span>
{:else if type === 'fontawesome'}
  <i class="icon icon-fontawesome {icon} {sizeClasses[size]} {className}" aria-label={name}></i>
{:else if type === 'svg'}
  <span class="icon icon-svg {sizeClasses[size]} {className}" aria-label={name}>
    {@html icon}
  </span>
{:else if type === 'system' || type === 'css'}
  <span class="icon icon-system {sizeClasses[size]} {className}" data-icon={icon} aria-label={name}>
    <!-- System/CSS icon placeholder - fallback to emoji -->
    {iconData.fallback}
  </span>
{:else}
  <!-- Custom template type (svg, image, etc.) - render as span -->
  <span class="icon icon-custom {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {icon}
  </span>
{/if}

<style lang="scss">
  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    
    &.icon-sm {
      font-size: 0.875rem;
      width: 1rem;
      height: 1rem;
    }
    
    &.icon-md {
      font-size: 1.25rem;
      width: 1.5rem;
      height: 1.5rem;
    }
    
    &.icon-lg {
      font-size: 1.75rem;
      width: 2rem;
      height: 2rem;
    }
    
    &.icon-xl {
      font-size: 2.5rem;
      width: 3rem;
      height: 3rem;
    }
    
    &.icon-emoji {
      font-family: 'Apple Color Emoji', 'Segoe UI Emoji', 'Noto Color Emoji', sans-serif;
    }
    
    &.icon-fontawesome {
      font-family: 'Font Awesome 6 Free', 'Font Awesome 6 Brands';
    }
    
    &.icon-svg {
      // SVG icon specific styles
      color: currentColor;
      
      :global(svg) {
        width: 100%;
        height: 100%;
        stroke: currentColor;
      }
    }
    
    &.icon-system {
      // System icon specific styles
      color: currentColor;
    }
    
    &.icon-custom {
      // Custom template icon styles
      color: currentColor;
      font-family: inherit;
    }
  }
</style>
