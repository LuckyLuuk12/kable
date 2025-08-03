<script lang="ts">
  import { onDestroy } from 'svelte';
  import { AuthService } from '$lib';
  import { isAuthenticating } from '../stores/auth';
  import Icon from './Icon.svelte';

  // Authentication state
  let error: string | null = null;
  let selectedAuthMethod: 'authcode' | 'devicecode' = 'authcode';

  // Device Code Flow state
  let deviceCodeData: any = null;
  let isPollingDeviceCode = false;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });

  /**
   * Primary authentication method - Authorization Code Flow
   */

  async function signInWithAuthCode() {
    try {
      error = null;
      await AuthService.signIn();
    } catch (err) {
      console.error('Authorization Code Flow sign in failed:', err);
      error = `Sign in failed: ${err}`;
    }
  }

  /**
   * Fallback authentication method - Device Code Flow
   */
  async function signInWithDeviceCode() {
    try {
      error = null;
      isPollingDeviceCode = true;
      // Step 1: Start device code flow and get device code data for display
      deviceCodeData = await AuthService.startDeviceCodeFlow();
      console.log('📱 Device code started:', deviceCodeData);
      // Step 2: Start polling for completion in the background
      try {
        const account = await AuthService.pollDeviceCodeCompletion(deviceCodeData.device_code);
        await AuthService.refreshAvailableAccounts();
        // Clear device code data and stop polling
        deviceCodeData = null;
        isPollingDeviceCode = false;
        console.log('✅ Device code authentication successful:', account.username);
      } catch (pollError) {
        console.error('❌ Device code polling failed:', pollError);
        error = `Device code authentication failed: ${pollError}`;
        deviceCodeData = null;
        isPollingDeviceCode = false;
      }
    } catch (err) {
      console.error('Device Code Flow sign in failed:', err);
      error = `Device Code sign in failed: ${err}`;
      deviceCodeData = null;
      isPollingDeviceCode = false;
    }
  }

  /**
   * Cancel device code authentication
   */
  function cancelDeviceCode() {
    deviceCodeData = null;
    isPollingDeviceCode = false;
    console.log('📱 Device code authentication cancelled');
  }
</script>

<div class="auth-flow">
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  <div class="auth-header">
    <div class="auth-icon">
      <Icon name="user-plus" size="xl" />
    </div>
    <h3>Sign in to Microsoft</h3>
    <p>Sign in with your Microsoft account to access online features and view your Minecraft profile.</p>
  </div>
  
  {#if deviceCodeData}
    <div class="device-code-info">
      <div class="device-code-header">
        <Icon name="info" size="sm" />
        <span>Device Code Authentication</span>
      </div>
      
      <div class="device-code-instructions">
        <p>Go to: <strong>{deviceCodeData.verification_uri}</strong></p>
        <p>Enter this code:</p>
        <div class="code-display">
          <code class="user-code">{deviceCodeData.user_code}</code>
          <button on:click={() => AuthService.copyToClipboard(deviceCodeData.user_code)} class="copy-btn">
            <Icon name="duplicate" size="sm" />
          </button>
        </div>
        
        {#if isPollingDeviceCode}
          <div class="polling-status">
            <Icon name="refresh" size="sm" />
            <span>Waiting for authentication...</span>
          </div>
        {/if}
        
        <div class="device-code-actions">
          <button on:click={() => AuthService.copyToClipboard(deviceCodeData.user_code)} class="btn btn-secondary">
            <Icon name="duplicate" size="sm" />
            Copy Code
          </button>
          <button on:click={cancelDeviceCode} class="btn btn-outline">
            <Icon name="x" size="sm" />
            Cancel
          </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="auth-options">
      <div class="auth-method-selector">
        <label class="radio-option">
          <input type="radio" bind:group={selectedAuthMethod} value="authcode" />
          <span>Authorization Code Flow (Recommended)</span>
        </label>
        <label class="radio-option">
          <input type="radio" bind:group={selectedAuthMethod} value="devicecode" />
          <span>Device Code Flow (Alternative)</span>
        </label>
      </div>
      
      <div class="auth-actions">
        {#if selectedAuthMethod === 'authcode'}
          <button on:click={signInWithAuthCode} class="btn btn-primary" disabled={$isAuthenticating}>
            <Icon name="microsoft" size="sm" />
            {$isAuthenticating ? 'Signing in...' : 'Sign in with Microsoft'}
          </button>
          <p class="auth-help">
            <Icon name="info" size="sm" />
            Opens a browser window for secure authentication.
          </p>
        {:else}
          <button on:click={signInWithDeviceCode} class="btn btn-secondary" disabled={$isAuthenticating}>
            <Icon name="qr-code" size="sm" />
            {$isAuthenticating ? 'Signing in...' : 'Sign in with Device Code'}
          </button>
          <p class="auth-help">
            <Icon name="info" size="sm" />
            Use this if the browser method doesn't work.
          </p>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
  @use '@kablan/clean-ui/scss/variables' as *;

  .auth-flow {
    width: 100%;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: rgba($red, 0.1);
    border: 1px solid $red;
    border-radius: $border-radius;
    color: $red;
    margin-bottom: 1rem;
  }

  .auth-header {
    text-align: center;
    margin-bottom: 2rem;

    .auth-icon {
      margin-bottom: 1.5rem;
      color: $primary;
      opacity: 0.8;
    }

    h3 {
      margin: 0 0 1rem;
      font-size: 1.25rem;
      font-weight: 600;
      color: $text;
    }

    p {
      margin: 0;
      color: $placeholder;
      line-height: 1.6;
    }
  }

  .auth-options {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    align-items: center;
  }

  .auth-method-selector {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 100%;
    max-width: 400px;

    .radio-option {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 1rem;
      background: $container;
      border: 1px solid $dark-600;
      border-radius: $border-radius;
      cursor: pointer;
      transition: all 0.2s ease;

      &:hover {
        border-color: $primary;
        background: rgba($primary, 0.05);
      }

      input[type="radio"] {
        accent-color: $primary;
      }

      span {
        color: $text;
        font-weight: 500;
      }
    }
  }

  .auth-actions {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
    width: 100%;
    max-width: 300px;

    .btn {
      width: 100%;
    }
  }

  .auth-help {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: $placeholder;
    margin: 0;
    text-align: center;
  }

  .device-code-info {
    padding: 1.5rem;
    background: rgba($primary, 0.05);
    border: 1px solid rgba($primary, 0.1);
    border-radius: $border-radius;
    text-align: center;
    
    .device-code-header {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      margin-bottom: 1rem;
      color: $primary;
      font-weight: 600;
    }
    
    .device-code-instructions {
      margin-bottom: 1rem;
      
      p {
        margin: 0 0 0.5rem;
        color: $text;
        font-size: 0.9rem;
      }
      
      .code-display {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        margin-bottom: 1rem;
        padding: 1rem;
        background: $container;
        border: 2px solid $primary;
        border-radius: $border-radius;
        
        .user-code {
          font-size: 1.25rem;
          font-weight: 700;
          color: $primary;
          letter-spacing: 0.1em;
        }
        
        .copy-btn {
          background: none;
          border: 1px solid rgba($primary, 0.3);
          border-radius: 4px;
          padding: 0.25rem;
          color: $primary;
          cursor: pointer;
          transition: all 0.2s ease;
          
          &:hover {
            background: rgba($primary, 0.1);
            border-color: $primary;
          }
        }
      }
    }
    
    .polling-status {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.5rem;
      margin-bottom: 1rem;
      color: $placeholder;
      
      :global(.icon) {
        animation: spin 1s linear infinite;
      }
    }
    
    .device-code-actions {
      display: flex;
      gap: 1rem;
      justify-content: center;
      margin-top: 1rem;
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
