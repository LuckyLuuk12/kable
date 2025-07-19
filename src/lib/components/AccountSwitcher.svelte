<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthManager, currentAccount, availableAccounts } from '$lib';
  import type { LauncherAccount } from '$lib';
  import Icon from './Icon.svelte';

  let showDropdown = false;
  let isLoading = false;

  // Show all available accounts, including offline
  $: validAccounts = $availableAccounts.filter(acc => acc?.username !== $currentAccount?.username);

  // Also log for debugging
  $: {
    console.log('🔍 AccountSwitcher - Available accounts:', $availableAccounts.length, $availableAccounts.map(acc => ({ ...acc })));
    console.log('🔍 AccountSwitcher - Valid accounts after filtering:', validAccounts.length, validAccounts);
    if (validAccounts.length > 0) {
      validAccounts.forEach(acc => console.log('  ✅', acc.local_id, acc.minecraft_profile?.name || acc.username));
    }
  }

  // Determine account status
  function getAccountStatus(account: LauncherAccount | null): 'online' | 'offline' | 'expired' {
    if (!account) return 'offline';
    if (!account.access_token) return 'offline';
    if (account.access_token_expires_at) {
      const expiryDate = new Date(account.access_token_expires_at);
      if (expiryDate <= new Date()) return 'expired';
    }
    return 'online';
  }

  onMount(async () => {
    if ($availableAccounts.length === 0) {
      await AuthManager.refreshAvailableAccounts();
    }
  });

  async function switchAccount(account: LauncherAccount) {
    if (account.local_id === $currentAccount?.local_id) return;
    isLoading = true;
    try {
      await AuthManager.switchAccount(account.local_id);
      showDropdown = false;
    } catch (error) {
      console.error('Failed to switch account:', error);
    } finally {
      showDropdown = false;
      isLoading = false;
    }
  }

  $: {
    console.log('🔍 AccountSwitcher - Current account:', $currentAccount);
  }
</script>

{#if ($currentAccount || $availableAccounts.length > 0)}
<div class="account-switcher">
  <div class="current-account">
    <div class="account-avatar-container" on:mouseenter={() => showDropdown = true} on:mouseleave={() => showDropdown = false} role="button" tabindex="0">
      <div class="account-avatar minecraft-head" title="{$currentAccount?.minecraft_profile?.name || $currentAccount?.username}'s avatar">
        <span class="avatar-letter">{($currentAccount?.minecraft_profile?.name || $currentAccount?.username || 'U').charAt(0).toUpperCase()}</span>
      </div>
      {#if getAccountStatus($currentAccount) === 'online'}
        <div class="status-indicator online" title="Online"></div>
      {:else if getAccountStatus($currentAccount) === 'offline'}
        <div class="status-indicator offline" title="Offline"></div>
      {:else}
        <div class="status-indicator expired" title="Token Expired"></div>
      {/if}
    </div>

    <div class="account-info">
      <span class="username">{$currentAccount?.minecraft_profile?.name || $currentAccount?.username || 'Unknown User'}</span>
      <span class="account-type">
        {#if getAccountStatus($currentAccount) === 'offline'}
          Offline Account
        {:else if getAccountStatus($currentAccount) === 'expired'}
          Microsoft Account (Token Expired)
        {:else}
          Microsoft Account
        {/if}
      </span>
    </div>

    <div class="dropdown-chevron" class:rotated={showDropdown}>
      <Icon name={showDropdown ? 'chevron-up' : 'chevron-down'} forceType="svg" />
    </div>

    <div class="dropdown-menu">
      {#each validAccounts as account (account.local_id)}
        <div class="account-item" class:active={account.local_id === $currentAccount?.local_id}>
          <button 
            class="account-button"
            on:click={() => switchAccount(account)}
          >
            <div class="account-avatar-container">
              <div class="account-avatar minecraft-head" title="{account.minecraft_profile?.name || account.username}'s avatar">
                <span class="avatar-letter">{(account.minecraft_profile?.name || account.username || 'U').charAt(0).toUpperCase()}</span>
              </div>
              {#if getAccountStatus(account) === 'online'}
                <div class="status-indicator online" title="Online"></div>
              {:else if getAccountStatus(account) === 'offline'}
                <div class="status-indicator offline" title="Offline"></div>
              {:else}
                <div class="status-indicator expired" title="Token Expired"></div>
              {/if}
            </div>

            <div class="account-info">
              <span class="username">{account.minecraft_profile?.name || account.username || 'Unknown User'}</span>
              <span class="account-type">
                {#if getAccountStatus(account) === 'offline'}
                  Offline Account
                {:else if getAccountStatus(account) === 'expired'}
                  Microsoft Account (Token Expired)
                {:else}
                  Microsoft Account
                {/if}
              </span>
            </div>
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>
{:else}
  <div class="no-account-container">
    <button class="sign-in-btn" on:click={() => AuthManager.signIn()}>
      <div class="sign-in-avatar">
        <Icon name="user-plus" size="lg" />
      </div>
      <div class="sign-in-info">
        <span class="sign-in-text">Sign in to Microsoft</span>
        <span class="sign-in-help">Access online features and view your Minecraft profile.</span>
      </div>
      <Icon name="arrow-right" />
    </button>
  </div>
{/if}

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  
  .account-switcher {
    position: relative;
    display: inline-block;
    min-width: 15rem;
    width: 100%;
  }
  
  // Removed unused .current-account-container
  
  .current-account {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    width: 100%;
    background: $container;
    border: 0.0625rem solid $dark-600;
    border-radius: 0.75rem;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    font-family: inherit;
    color: inherit;
    
    &:hover:not(:disabled) {
      background: $button-hover;
      border-color: $primary;
      box-shadow: 0 0.125rem 0.5rem rgba($primary, 0.15);
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  
    // show the dropdown-menu when hovered / focused
    &:hover .dropdown-menu {
      display: block;
    }
  }
  
  .account-avatar-container {
    position: relative;
    flex-shrink: 0;
  }
  
  .account-avatar {
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 50%;
    background: $container;
    border: 0.125rem solid $dark-600;
    transition: border-color 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 1rem;
    color: $text;
    
    &.minecraft-head {
      background: linear-gradient(135deg, $primary, $primary-600);
      color: white;
      border-color: $primary;
    }
  
    .avatar-letter {
      user-select: none;
    }
  }
  
  .status-indicator {
    position: absolute;
    bottom: -0.125rem;
    right: -0.125rem;
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 50%;
    border: 0.125rem solid $container;
    
    &.online {
      background: $green;
    }
    
    &.offline {
      background: $yellow;
    }
    
    &.expired {
      background: $red;
    }
  }
  
  .account-info {
    flex: 1;
    text-align: left;
    min-width: 0;
  }
  
  .username {
    display: block;
    font-weight: 600;
    color: $text;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .account-type {
    display: block;
    font-size: 0.75rem;
    color: $placeholder;
    margin-top: 0.125rem;
  }
  
  .dropdown-chevron {
    flex-shrink: 0;
    fill: $placeholder;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    
    &.rotated {
      transform: rotate(180deg);
    }
  }
  
  .dropdown-menu {
    display: none;
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: $card;
    border: 0.0625rem solid $dark-600;
    border-radius: 0.75rem;
    box-shadow: 
      0 0.625rem 1.5625rem rgba(0, 0, 0, 0.3),
      0 0 0 0.0625rem rgba(255, 255, 255, 0.05);
    z-index: 1000;
    overflow: hidden;
    backdrop-filter: blur(1.25rem);
  }
  
  // Removed unused .accounts-list and custom scrollbar styles
  
  .account-item {
    position: relative;
    border-radius: 0.5rem;
    overflow: hidden;
    
    &.active {
      background: rgba($primary, 0.1);
    }
  }
  
  .account-button {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    width: 100%;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background-color 0.2s ease;
    text-align: left;
    font-family: inherit;
    color: inherit;
    border-radius: 0.5rem;
    
    &:hover:not(:disabled) {
      background: $container;
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: default;
    }
  }
  
  // Removed unused .account-details (not present in markup)
  
  // Removed unused .account-meta, .account-id, .offline-badge, .expired-badge
  
  // Removed unused .current-badge
  
  // Removed unused .remove-btn.trash-btn and hover styles
  
  /* Add Account Button */
  // Removed unused .add-account-item, .add-account-btn, .add-icon
  
  /* Sign-in button when no accounts */
  // Removed unused .no-account-container, .sign-in-btn, .sign-in-avatar, .sign-in-info, .sign-in-text, .sign-in-help, .sign-in-arrow, .sign-in-btn:hover .sign-in-arrow
  
  /* Account selection when no current account */
  // Removed unused .account-selection, .selection-header, .account-count
  
  /* Responsive design */
  @media (max-width: 48rem) {
    .account-switcher {
      min-width: 12.5rem;
    }
    
    .dropdown-menu {
      left: -0.5rem;
      right: -0.5rem;
    }
  }
</style>
