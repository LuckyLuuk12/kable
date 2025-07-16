<script lang="ts">
  import { onMount } from 'svelte';
  import { IconManager, selectedTemplate } from '../managers/IconManager';

  export let name: string;
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let className: string = '';

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
    await IconManager.initialize();
    updateIcon();
  });

  // Update icon when name changes or when selected template changes
  $: if (name || $selectedTemplate) {
    updateIcon();
  }

  function updateIcon() {
    iconData = IconManager.getIcon(name);
  }

  // Reactive statements for rendering type
  $: isEmoji = iconData.type === 'emoji';
  $: isFontAwesome = iconData.type === 'fontawesome';
  $: isSystem = iconData.type === 'system' || iconData.type === 'css';
  $: isCustom = !isEmoji && !isFontAwesome && !isSystem;
</script>

{#if isEmoji}
  <span class="icon icon-emoji {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {iconData.icon}
  </span>
{:else if isFontAwesome}
  <i class="icon icon-fontawesome {iconData.icon} {sizeClasses[size]} {className}" aria-label={name}></i>
{:else if isSystem}
  <span class="icon icon-system {sizeClasses[size]} {className}" data-icon={iconData.icon} aria-label={name}>
    <!-- System/CSS icon placeholder - fallback to emoji -->
    {iconData.fallback}
  </span>
{:else}
  <!-- Custom template type (svg, image, etc.) - render as span -->
  <span class="icon icon-custom {sizeClasses[size]} {className}" role="img" aria-label={name}>
    {iconData.icon}
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
