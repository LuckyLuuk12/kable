<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthManager, currentAccount, type LauncherAccount } from '$lib';
  import Icon from './Icon.svelte';
  import AuthenticationFlow from './AuthenticationFlow.svelte';

  let isLoading = false;
  let showAddAccount = false;

  // Check if current account is offline (no access token)
  $: isOffline = !$currentAccount || !$currentAccount.access_token;

  let showActionsDropdown = false;

  async function refreshTokenDCF() {
    if (!$currentAccount || isOffline) return;
    isLoading = true;
    try {
      await AuthManager.signInWithDeviceCode();
    } catch (error) {
      console.error('Device Code Flow refresh failed:', error);
    } finally {
      isLoading = false;
      showActionsDropdown = false;
    }
  }

  async function reloginCurrentAccount() {
    if (!$currentAccount) return;
    isLoading = true;
    try {
      await AuthManager.signIn();
    } catch (error) {
      console.error('Manual re-login failed:', error);
    } finally {
      isLoading = false;
      showActionsDropdown = false;
    }
  }
  // Determine account status
  function getAccountStatus(account: LauncherAccount): 'online' | 'offline' | 'expired' {
    if (!account.access_token) return 'offline';
    if (account.access_token_expires_at) {
      const expiryDate = new Date(account.access_token_expires_at);
      if (expiryDate <= new Date()) return 'expired';
    }
    return 'online';
  }

  // Format token expiry for display
  function formatTokenExpiry(account: LauncherAccount): string {
    if (!account.access_token_expires_at) return 'Never expires';
    const expiryDate = new Date(account.access_token_expires_at);
    const now = new Date();
    const diff = expiryDate.getTime() - now.getTime();
    if (diff <= 0) return 'Expired';
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    if (days > 0) return `Expires in ${days} day${days > 1 ? 's' : ''}`;
    if (hours > 0) return `Expires in ${hours} hour${hours > 1 ? 's' : ''}`;
    return 'Expires soon';
  }

  onMount(async () => {
    // Initialize authentication and load accounts
    await AuthManager.initialize();
    await AuthManager.refreshAvailableAccounts();
  });

  /**
   * Remove current account
   */
  async function removeCurrentAccount() {
    if (!$currentAccount) return;
    if (!confirm(`Remove "${$currentAccount.minecraft_profile?.name || $currentAccount.username}" from your accounts?`)) {
      return;
    }
    isLoading = true;
    try {
      await AuthManager.removeAccount($currentAccount.local_id);
    } catch (error) {
      console.error('Failed to remove account:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Refresh current account token
   */
  async function refreshToken() {
    if (!$currentAccount || isOffline) return;
    
    isLoading = true;
    try {
      await AuthManager.refreshCurrentAccount();
    } catch (error) {
      console.error('Token refresh failed:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Sign out current account
   */
  async function signOut() {
    try {
      await AuthManager.signOut();
    } catch (error) {
      console.error('Sign out failed:', error);
    }
  }
</script>

<div class="account-manager">
  {#if $currentAccount}
    <div class="current-account-section">
      <div class="current-account-card">
        <div class="account-avatar-details-row">
          <div class="account-avatar-container">
            <div class="account-avatar minecraft-head large" title="{$currentAccount.minecraft_profile?.name || $currentAccount.username}'s avatar">
              <span class="avatar-letter">{($currentAccount.minecraft_profile?.name || $currentAccount.username || 'U').charAt(0).toUpperCase()}</span>
            </div>
            {#if getAccountStatus($currentAccount) === 'online'}
              <div class="status-indicator online" title="Online"></div>
            {:else if getAccountStatus($currentAccount) === 'offline'}
              <div class="status-indicator offline" title="Offline"></div>
            {:else}
              <div class="status-indicator expired" title="Token Expired"></div>
            {/if}
          </div>
          <div class="account-details-horizontal">
            <div class="account-details-main">
              <h4>{$currentAccount.minecraft_profile?.name || $currentAccount.username || 'Unknown User'}</h4>
              <p class="account-type">
                {#if getAccountStatus($currentAccount) === 'offline'}
                  Offline Account
                {:else}
                  Microsoft Account
                {/if}
              </p>
            </div>
            <div class="account-details-side">
              <span class="account-id">UUID: {$currentAccount.minecraft_profile?.id}</span>
              {#if getAccountStatus($currentAccount) !== 'offline'}
                <span class="token-status" class:expired={getAccountStatus($currentAccount) === 'expired'}>
                  {formatTokenExpiry($currentAccount)}
                </span>
              {/if}
            </div>
          </div>
        </div>
        <div class="dropdown account-actions-dropdown"
          on:mouseenter={() => showActionsDropdown = true}
          on:mouseleave={() => showActionsDropdown = false}
          on:focusin={() => showActionsDropdown = true}
          on:focusout={() => showActionsDropdown = false}
          role="menu" tabindex="0"
        >
          <button class="btn btn-secondary dropdown-toggle actions-dropdown-btn" aria-label="Account Actions">
            <Icon name="more-horizontal" size="sm" />
          </button>
          <div class="dropdown-menu actions-dropdown-menu" style="right: 0.25rem; top: 2.35rem; min-width: 180px;" tabindex="-1">
            <button class="dropdown-action" on:click={refreshToken} disabled={isLoading}>
              <Icon name="refresh" size="sm" /> Refresh
            </button>
            <button class="dropdown-action" on:click={reloginCurrentAccount} disabled={isLoading}>
              <Icon name="login" size="sm" /> Re-login
            </button>
            <div class="dropdown-separator"></div>
            <button class="dropdown-action" on:click={refreshTokenDCF} disabled={isLoading}>
              <Icon name="login" size="sm" /> Re-login (DCF)
            </button>
            <div class="dropdown-separator"></div>
            <button class="dropdown-action" on:click={signOut}>
              <Icon name="logout" size="sm" /> Sign Out
            </button>
            <button class="dropdown-action danger" on:click={removeCurrentAccount}>
              <Icon name="trash" size="sm" /> Remove
            </button>
          </div>
        </div>
      </div>
      {#if isOffline}
        <div class="auth-flow-container">
          <AuthenticationFlow />
        </div>
      {/if}
    </div>
  {:else}
    <div class="no-accounts-container">
      <div class="welcome-message">
        <div class="welcome-icon">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <circle cx="24" cy="16" r="8" stroke="currentColor" stroke-width="2"/>
            <path d="M8 40c0-8.837 7.163-16 16-16s16 7.163 16 16" stroke="currentColor" stroke-width="2"/>
          </svg>
        </div>
        <h3>Welcome to Kable</h3>
           <!-- <button on:click={reloginCurrentAccount} class="btn btn-warning btn-sm" disabled={isLoading}>
             <Icon name="login" size="sm" />
             {isLoading ? 'Logging in...' : 'Re-login'}
           </button> -->
        <p>Sign in with your Microsoft account to get started with Minecraft.</p>
      </div>
      <div class="auth-flow-container">
        <AuthenticationFlow />
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;
  
  .account-manager {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .dropdown.account-actions-dropdown {
    position: absolute;
    top: 1rem;
    right: 1rem;
    z-index: 20;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    &:hover .dropdown-menu,
    &:focus-within .dropdown-menu,
    .dropdown-menu:hover,
    .dropdown-menu:focus-within {
      opacity: 1;
      pointer-events: auto;
      z-index: 1001;
      display: flex;
    }
  }

  .dropdown-toggle.actions-dropdown-btn {
    background: none;
    border: none;
    padding: 0.5rem 0.75rem;
    border-radius: $border-radius;
    cursor: pointer;
    color: $text;
    transition: background 0.12s;
    height: 2.25rem;
    min-height: 2.25rem;
    max-height: 2.25rem;
    display: flex;
    align-items: center;
    z-index: 10;
    &:hover, &:focus {
      background: $button-hover;
    }
  }

  .dropdown-menu.actions-dropdown-menu {
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s cubic-bezier(0.4,0,0.2,1), z-index 0s linear 0.15s;
    position: absolute;
    min-width: 180px;
    background: rgba($card, 0.94);
    border: 1px solid $dark-200;
    border-radius: $border-radius;
    box-shadow: 0 2px 16px 4px rgba(0,0,0,0.18), 0 2px 8px rgba(0,0,0,0.08);
    z-index: 1;
    flex-direction: column;
    padding: 0.5rem 0;
    backdrop-filter: blur(0.7rem) saturate(1.2);
    -webkit-backdrop-filter: blur(0.7rem) saturate(1.2);
    display: flex;
    .dropdown-separator {
      height: 1px;
      background: $dark-200;
      margin: 0.3rem 0;
    }
    button {
      width: 100%;
      background: none;
      border: none;
      padding: 0.5rem 1rem;
      text-align: left;
      color: $text;
      font-size: 1rem;
      border-radius: 0;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 0.5rem;
      transition: background 0.12s;
    }
    .danger {
      color: $red-700;
    }
  }
  
  .account-avatar-container {
    position: relative;
    flex-shrink: 0;
  }
  
  .account-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-weight: 600;
    color: white;
    border: 3px solid $dark-600;
    transition: border-color 0.2s ease;
    
    &.minecraft-head {
      background: linear-gradient(135deg, $primary, $primary-600);
      border-color: $primary;
    }
    
    &.large {
      width: 64px;
      height: 64px;
      font-size: 24px;
    }
    
    width: 40px;
    height: 40px;
    font-size: 16px;
    
    .avatar-letter {
      user-select: none;
    }
  }
  
  .status-indicator {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 3px solid $container;
    
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
  .account-avatar-details-row {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    width: 100%;
    position: relative;
  }
  .account-details-horizontal {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 2.5rem;
    flex: 1;
    min-width: 0;
    justify-content: space-between;
  }
  .account-details-main {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    h4 {
      margin: 0 0 4px 0;
      font-size: 20px;
      font-weight: 600;
      color: $text;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 220px;
    }
    .account-type {
      margin: 0 0 8px 0;
      font-size: 14px;
      color: $placeholder;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 220px;
    }
  }
  .account-details-side {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: flex-end;
    min-width: 0;
    .account-id {
      font-size: 12px;
      color: $placeholder;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .token-status {
      font-size: 13px;
      color: $green;
      font-weight: 500;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 180px;
      &.expired {
        color: $red;
      }
    }
  }
  
  .account-id {
    font-size: 11px;
    color: $placeholder;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    opacity: 0.8;
  }
  
  .no-accounts-container {
    text-align: center;
    padding: 48px 24px;
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 1rem;
  }
  
  .welcome-message {
    margin-bottom: 32px;
    
    .welcome-icon {
      width: 64px;
      height: 64px;
      margin: 0 auto 16px;
      background: linear-gradient(135deg, $primary, $primary-600);
      border-radius: 16px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
    }
    
    h3 {
      margin: 0 0 8px 0;
      font-size: 24px;
      font-weight: 600;
      color: $text;
    }
    
    p {
      margin: 0;
      font-size: 16px;
      color: $placeholder;
      line-height: 1.5;
    }
  }
  
  .auth-flow-container {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 16px;
    overflow: hidden;
  }
  
  /* Responsive design */
  @media (max-width: 768px) {
    .current-account-card {
      flex-direction: column;
      text-align: center;
      gap: 1rem;
    }
  }
</style>
