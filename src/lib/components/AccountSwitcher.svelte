<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthManager } from '../managers/AuthManager';
  import { currentAccount, availableAccounts, isAuthenticated } from '../stores/auth';
  import type { MicrosoftAccount } from '../types';
  import { fly, scale } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  
  let showDropdown = false;
  let isLoading = false;
  let dropdownElement: HTMLElement;
  
  onMount(async () => {
    // Initialize authentication and load accounts
    await AuthManager.initialize();
  });
  
  async function switchAccount(account: MicrosoftAccount) {
    if (account.uuid === $currentAccount?.uuid) return;
    
    isLoading = true;
    try {
      await AuthManager.switchToAccount(account.uuid);
      showDropdown = false;
    } catch (error) {
      console.error('Failed to switch account:', error);
    } finally {
      isLoading = false;
    }
  }
  
  async function signOut(account: MicrosoftAccount, event: Event) {
    event.stopPropagation();
    isLoading = true;
    try {
      await AuthManager.signOutAccount(account.uuid);
    } catch (error) {
      console.error('Failed to sign out account:', error);
    } finally {
      isLoading = false;
    }
  }
  
  async function addAccount() {
    isLoading = true;
    try {
      await AuthManager.signIn();
      showDropdown = false;
    } catch (error) {
      console.error('Failed to add account:', error);
    } finally {
      isLoading = false;
    }
  }
  
  function toggleDropdown(event: Event) {
    event.stopPropagation();
    showDropdown = !showDropdown;
  }
  
  function closeDropdown() {
    showDropdown = false;
  }
  
  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as Element;
    if (!target.closest('.account-switcher')) {
      closeDropdown();
    }
  }
  
  // Handle keyboard navigation
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeDropdown();
    }
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<div class="account-switcher">
  {#if $isAuthenticated && $currentAccount}
    <div class="current-account-container">
      <button 
        class="current-account" 
        on:click={toggleDropdown} 
        disabled={isLoading}
        aria-expanded={showDropdown}
        aria-haspopup="true"
      >
        <div class="account-avatar-container">
          <img 
            src={$currentAccount.skin_url || '/default-avatar.png'} 
            alt="{$currentAccount.username}'s avatar"
            class="account-avatar"
            loading="lazy"
          />
          <div class="status-indicator online" title="Online"></div>
        </div>
        
        <div class="account-info">
          <span class="username">{$currentAccount.username}</span>
          <span class="account-type">Microsoft Account</span>
        </div>
        
        <svg 
          class="dropdown-chevron" 
          class:rotated={showDropdown} 
          width="16" 
          height="16" 
          viewBox="0 0 16 16"
          aria-hidden="true"
        >
          <path d="M4.427 6.573l3.396 3.396a.25.25 0 00.354 0l3.396-3.396A.25.25 0 0011.396 6H4.604a.25.25 0 00-.177.427z"/>
        </svg>
      </button>
      
      {#if showDropdown}
        <div 
          class="dropdown-menu"
          bind:this={dropdownElement}
          transition:fly={{ y: -10, duration: 200, easing: quintOut }}
        >
          <div class="dropdown-header">
            <h4>Switch Account</h4>
            <span class="account-count">{$availableAccounts.length} account{$availableAccounts.length !== 1 ? 's' : ''}</span>
          </div>
          
          <div class="accounts-list">
            {#each $availableAccounts as account (account.uuid)}
              <div 
                class="account-item" 
                class:active={account.uuid === $currentAccount?.uuid}
                transition:scale={{ duration: 150, start: 0.95 }}
              >
                <button 
                  class="account-button"
                  on:click={() => switchAccount(account)}
                  disabled={isLoading || account.uuid === $currentAccount?.uuid}
                >
                  <div class="account-avatar-container">
                    <img 
                      src={account.skin_url || '/default-avatar.png'} 
                      alt="{account.username}'s avatar"
                      class="account-avatar small"
                      loading="lazy"
                    />
                    {#if account.uuid === $currentAccount?.uuid}
                      <div class="status-indicator current" title="Current account"></div>
                    {/if}
                  </div>
                  
                  <div class="account-details">
                    <span class="username">{account.username}</span>
                    <span class="account-id">{account.uuid.slice(0, 8)}...{account.uuid.slice(-8)}</span>
                  </div>
                  
                  {#if account.uuid === $currentAccount?.uuid}
                    <div class="current-badge">
                      <svg width="12" height="12" viewBox="0 0 12 12">
                        <path d="M10 3L4.5 8.5 2 6" stroke="currentColor" stroke-width="2" fill="none"/>
                      </svg>
                      Current
                    </div>
                  {/if}
                </button>
                
                {#if account.uuid !== $currentAccount?.uuid}
                  <button 
                    class="remove-btn" 
                    on:click={(e) => signOut(account, e)}
                    disabled={isLoading}
                    title="Remove account"
                    aria-label="Remove {account.username}"
                  >
                    <svg width="14" height="14" viewBox="0 0 14 14">
                      <path d="M11 3L3 11M3 3l8 8" stroke="currentColor" stroke-width="2"/>
                    </svg>
                  </button>
                {/if}
              </div>
            {/each}
          </div>
          
          <div class="dropdown-footer">
            <button 
              class="add-account-btn" 
              on:click={addAccount} 
              disabled={isLoading}
            >
              {#if isLoading}
                <div class="loading-spinner"></div>
                <span>Adding...</span>
              {:else}
                <svg width="16" height="16" viewBox="0 0 16 16">
                  <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="2"/>
                </svg>
                <span>Add Microsoft Account</span>
              {/if}
            </button>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <button 
      class="sign-in-btn" 
      on:click={addAccount} 
      disabled={isLoading}
    >
      {#if isLoading}
        <div class="loading-spinner"></div>
        <span>Signing in...</span>
      {:else}
        <svg width="16" height="16" viewBox="0 0 16 16">
          <path d="M15 8a7 7 0 1 1-14 0 7 7 0 0 1 14 0ZM4.5 7.5a.5.5 0 0 0 0 1h5.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3a.5.5 0 0 0 0-.708l-3-3a.5.5 0 1 0-.708.708L10.293 7.5H4.5Z"/>
        </svg>
        <span>Sign in with Microsoft</span>
      {/if}
    </button>
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  
  .account-switcher {
    position: relative;
    display: inline-block;
    min-width: 240px;
  }
  
  .current-account-container {
    position: relative;
  }
  
  .current-account {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    width: 100%;
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    font-family: inherit;
    color: inherit;
    
    &:hover:not(:disabled) {
      background: $button-hover;
      border-color: $primary;
      box-shadow: 0 2px 8px rgba($primary, 0.15);
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
    
    &[aria-expanded="true"] {
      border-color: $primary;
      box-shadow: 0 0 0 2px rgba($primary, 0.15);
    }
  }
  
  .account-avatar-container {
    position: relative;
    flex-shrink: 0;
  }
  
  .account-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    object-fit: cover;
    background: $container;
    border: 2px solid $dark-600;
    transition: border-color 0.2s ease;
    
    &.small {
      width: 28px;
      height: 28px;
    }
  }
  
  .status-indicator {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid $container;
    
    &.online {
      background: $green;
    }
    
    &.current {
      background: $primary;
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
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .account-type {
    display: block;
    font-size: 12px;
    color: $placeholder;
    margin-top: 2px;
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
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    background: $background;
    border: 1px solid $dark-600;
    border-radius: 12px;
    box-shadow: 
      0 10px 25px rgba(0, 0, 0, 0.3),
      0 0 0 1px rgba(255, 255, 255, 0.05);
    z-index: 1000;
    overflow: hidden;
    backdrop-filter: blur(20px);
  }
  
  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
    border-bottom: 1px solid $dark-600;
    
    h4 {
      margin: 0;
      font-size: 15px;
      font-weight: 600;
      color: $text;
    }
    
    .account-count {
      font-size: 12px;
      color: $placeholder;
      background: $container;
      padding: 2px 8px;
      border-radius: 6px;
    }
  }
  
  .accounts-list {
    max-height: 240px;
    overflow-y: auto;
    padding: 8px 0;
    
    /* Custom scrollbar */
    &::-webkit-scrollbar {
      width: 6px;
    }
    
    &::-webkit-scrollbar-track {
      background: transparent;
    }
    
    &::-webkit-scrollbar-thumb {
      background: $dark-600;
      border-radius: 3px;
    }
    
    &::-webkit-scrollbar-thumb:hover {
      background: $placeholder;
    }
  }
  
  .account-item {
    position: relative;
    margin: 0 8px;
    border-radius: 8px;
    overflow: hidden;
    
    &.active {
      background: rgba($primary, 0.1);
    }
  }
  
  .account-button {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    width: 100%;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background-color 0.2s ease;
    text-align: left;
    font-family: inherit;
    color: inherit;
    border-radius: 8px;
    
    &:hover:not(:disabled) {
      background: $container;
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: default;
    }
  }
  
  .account-details {
    flex: 1;
    min-width: 0;
    
    .username {
      font-size: 13px;
      margin-bottom: 2px;
    }
  }
  
  .account-id {
    display: block;
    font-size: 11px;
    color: $placeholder;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    opacity: 0.8;
  }
  
  .current-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    padding: 4px 8px;
    background: $primary;
    color: white;
    border-radius: 6px;
    font-weight: 500;
    white-space: nowrap;
    
    svg {
      flex-shrink: 0;
    }
  }
  
  .remove-btn {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 24px;
    height: 24px;
    border: none;
    background: $button;
    color: $placeholder;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    opacity: 0;
    
    .account-item:hover & {
      opacity: 1;
    }
    
    &:hover:not(:disabled) {
      background: $red;
      color: white;
      transform: translateY(-50%) scale(1.1);
    }
  }
  
  .dropdown-footer {
    padding: 12px 16px 16px;
    border-top: 1px solid $dark-600;
  }
  
  .add-account-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 12px 16px;
    background: $primary;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    font-family: inherit;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    
    &:hover:not(:disabled) {
      background: $primary-600;
      transform: translateY(-1px);
      box-shadow: 0 4px 12px rgba($primary, 0.3);
    }
    
    &:active:not(:disabled) {
      transform: translateY(0);
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
      transform: none;
    }
    
    svg {
      flex-shrink: 0;
    }
  }
  
  .sign-in-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 20px;
    background: $primary;
    color: white;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    font-family: inherit;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    min-width: 200px;
    
    &:hover:not(:disabled) {
      background: $primary-600;
      transform: translateY(-2px);
      box-shadow: 0 8px 25px rgba($primary, 0.3);
    }
    
    &:active:not(:disabled) {
      transform: translateY(-1px);
    }
    
    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
      transform: none;
    }
    
    svg {
      flex-shrink: 0;
    }
  }
  
  .loading-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid currentColor;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    flex-shrink: 0;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  /* Responsive design */
  @media (max-width: 768px) {
    .account-switcher {
      min-width: 200px;
    }
    
    .dropdown-menu {
      left: -8px;
      right: -8px;
    }
  }
</style>
