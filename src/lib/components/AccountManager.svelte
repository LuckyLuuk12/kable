<script lang="ts">
  import { onMount } from 'svelte';
  import { AuthManager, currentAccount, type LauncherAccount } from '$lib';
  import Icon from './Icon.svelte';
  import AuthenticationFlow from './AuthenticationFlow.svelte';

  let isLoading = false;
  let showAddAccount = false;

  // Check if current account is offline (no access token)
  $: isOffline = !$currentAccount || !$currentAccount.access_token;

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
      <div class="section-header">
        <h3>Current Account</h3>
        <div class="account-actions">
          {#if getAccountStatus($currentAccount) === 'expired'}
            <button on:click={refreshToken} class="btn btn-primary btn-sm" disabled={isLoading}>
              <Icon name="refresh" size="sm" />
              {isLoading ? 'Refreshing...' : 'Refresh Token'}
            </button>
          {/if}
          {#if !isOffline}
            <button on:click={signOut} class="btn btn-secondary btn-sm">
              <Icon name="logout" size="sm" />
              Sign Out
            </button>
            <button on:click={removeCurrentAccount} class="btn btn-outline btn-sm">
              <Icon name="trash" size="sm" />
              Remove
            </button>
          {/if}
        </div>
      </div>
      <div class="current-account-card">
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
        <div class="account-details">
          <h4>{$currentAccount.minecraft_profile?.name || $currentAccount.username || 'Unknown User'}</h4>
          <p class="account-type">
            {#if getAccountStatus($currentAccount) === 'offline'}
              Offline Account
            {:else if getAccountStatus($currentAccount) === 'expired'}
              Microsoft Account (Token Expired)
            {:else}
              Microsoft Account
            {/if}
          </p>
          <p class="account-id">UUID: {$currentAccount.minecraft_profile?.id}</p>

          {#if getAccountStatus($currentAccount) !== 'offline'}
            <p class="token-status" class:expired={getAccountStatus($currentAccount) === 'expired'}>
              {formatTokenExpiry($currentAccount)}
            </p>
          {/if}
        </div>
      </div>
    </div>
    {#if isOffline}
      <div class="auth-flow-container">
        <AuthenticationFlow />
      </div>
    {/if}
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
    max-width: 800px;
    margin: 0 auto;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid $dark-600;
    
    h3 {
      margin: 0;
      font-size: 18px;
      font-weight: 600;
      color: $text;
    }
    
    .account-actions {
      display: flex;
      gap: 8px;
    }
  }
  
  .current-account-card {
    background: $container;
    border: 1px solid $dark-600;
    border-radius: 16px;
    padding: 24px;
    display: flex;
    align-items: center;
    gap: 20px;
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
  
  .account-details {
    flex: 1;
    
    h4 {
      margin: 0 0 4px 0;
      font-size: 20px;
      font-weight: 600;
      color: $text;
    }
    
    .account-type {
      margin: 0 0 8px 0;
      font-size: 14px;
      color: $placeholder;
    }
    
    .account-id {
      margin: 0 0 8px 0;
      font-size: 12px;
      color: $placeholder;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    }
    
    .token-status {
      margin: 0;
      font-size: 13px;
      color: $green;
      font-weight: 500;
      
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
    border-radius: 16px;
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
      gap: 16px;
    }
  }
</style>
