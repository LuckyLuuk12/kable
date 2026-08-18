<!-- @component
AuthenticationFlow - Microsoft authentication flow UI component

Provides interface for authenticating with Microsoft using Device Code Flow

@example
```svelte
◄AuthenticationFlow /►
```
-->
<script lang="ts">
import { app, Icon, Image } from "$lib";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { onDestroy } from "svelte";
// import * as systemApi from "$lib/api/system";

// Authentication state
let error: string | null = $state(null);

// Device Code Flow state
let deviceCodeData: any = $state(null);
let isPollingDeviceCode = $state(false);
let pollInterval: ReturnType<typeof setInterval> | null = $state(null);

let isAuthenticating = $derived(app.authService.authenticating);

onDestroy(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});

/**
 * Device Code Flow authentication
 */
async function signInWithDeviceCode() {
  try {
    error = null;
    isPollingDeviceCode = true;

    // Step 1: Start device code flow and get device code data for display
    deviceCodeData = await app.authService.startAuth();
    console.log("📱 Device code started:", deviceCodeData);

    // Auto-copy code to clipboard
    await writeText(deviceCodeData.user_code);

    // Step 2: Start polling for completion in the background
    try {
      const account = await app.authService.pollAuth(deviceCodeData.device_code);
      // await app.authService.refreshAvailableAccounts();
      // Clear device code data and stop polling
      deviceCodeData = null;
      isPollingDeviceCode = false;
      console.log("✅ Device code authentication successful:", account.expires_at);
    } catch (pollError) {
      console.error("❌ Device code polling failed:", pollError);
      error = `Device code authentication failed: ${pollError}`;
      deviceCodeData = null;
      isPollingDeviceCode = false;
    }
  } catch (err) {
    console.error("Device Code Flow sign in failed:", err);
    error = `Device Code sign in failed: ${err}`;
    deviceCodeData = null;
    isPollingDeviceCode = false;
  }
}

/**
 * Open verification URL in browser
 */
async function openVerificationUrl() {
  if (deviceCodeData) {
    await app.openUrl(deviceCodeData.verification_uri);
  }
}

/**
 * Cancel device code authentication
 */
function cancelDeviceCode() {
  deviceCodeData = null;
  isPollingDeviceCode = false;
  console.log("📱 Device code authentication cancelled");
}
</script>

<div class="auth-flow">
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      {error}
    </div>
  {/if}

  {#if deviceCodeData}
    <div class="device-code-container">
      <div class="device-code-header">
        <div class="microsoft-logo">
          <Image key="microsoft-logo" alt="Microsoft" width="20px" height="20px" />
        </div>
        <h3>Sign in to Microsoft</h3>
      </div>

      <div class="device-code-card">
        <div class="step-indicator">
          <span class="step-number">1</span>
          <p>Click to open Microsoft activation page</p>
        </div>

        <button onclick={openVerificationUrl} class="verification-link-btn">
          <Icon name="link" size="sm" />
          {deviceCodeData.verification_uri}
        </button>

        <div class="step-indicator">
          <span class="step-number">2</span>
          <p>Enter this code (automatically copied)</p>
        </div>

        <div class="code-display">
          <code class="user-code">{deviceCodeData.user_code}</code>
          <button onclick={() => writeText(deviceCodeData.user_code)} class="copy-btn" title="Copy code">
            <Icon name="duplicate" size="sm" />
          </button>
        </div>

        {#if isPollingDeviceCode}
          <div class="polling-status">
            <div class="spinner"></div>
            <span>Waiting for you to complete authentication...</span>
          </div>
        {/if}

        <div class="device-code-actions">
          <button onclick={cancelDeviceCode} class="btn btn-secondary btn-sm"> Cancel </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="sign-in-container">
      <button onclick={signInWithDeviceCode} class="btn-microsoft" disabled={isAuthenticating}>
        <div class="microsoft-logo-large">
          <Image key="microsoft-logo" alt="Microsoft" width="21px" height="21px" />
        </div>
        <span>{isAuthenticating ? "Signing in..." : "Sign in with Microsoft"}</span>
      </button>

      <p class="auth-disclaimer">
        <Icon name="lock" size="sm" />
        Secure authentication via Microsoft
      </p>
    </div>
  {/if}
</div>

<style lang="scss">
.auth-flow {
  width: 100%;
  padding: 1.5rem;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: color-mix(in srgb, var(--red), 10%, transparent);
  border: 1px solid var(--red);
  border-radius: 8px;
  color: var(--red);
  margin-bottom: 1rem;
  font-size: 0.875rem;
}

.sign-in-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.25rem;
  padding: 0.5rem 0;
}

.btn-microsoft {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 0.75rem 1.75rem;
  background: white;
  color: #5e5e5e;
  border: 1px solid #8c8c8c;
  border-radius: 2px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  min-width: 240px;
  font-family:
    "Segoe UI",
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;

  &:hover:not(:disabled) {
    background: #f3f3f3;
    border-color: #5e5e5e;
  }

  &:active:not(:disabled) {
    background: #e5e5e5;
    transform: scale(0.98);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .microsoft-logo-large {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  span {
    letter-spacing: -0.01em;
  }
}

.auth-disclaimer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--placeholder);
  margin: 0;
}

.device-code-container {
  max-width: 450px;
  margin: 0 auto;
}

.device-code-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.625rem;
  margin-bottom: 1.25rem;

  .microsoft-logo {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  h3 {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 600;
    color: var(--text);
  }
}

.device-code-card {
  background: var(--container);
  border-radius: 10px;
  padding: 0.25rem;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1rem;
}

.step-indicator {
  display: flex;
  align-items: center;
  gap: 0.625rem;

  .step-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: $color-accent;
    color: white;
    border-radius: 50%;
    font-size: 0.8125rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  p {
    margin: 0;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text);
  }
}

.verification-link-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.625rem 0.875rem;
  background: color-mix(in srgb, $color-accent, 8%, transparent);
  border: 1px solid $color-accent;
  border-radius: 6px;
  color: $color-accent;
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: "SF Mono", "Monaco", "Consolas", monospace;
  word-break: break-all;

  &:hover {
    background: color-mix(in srgb, $color-accent, 15%, transparent);
  }

  &:active {
    background: color-mix(in srgb, $color-accent, 20%, transparent);
    transform: scale(0.98);
  }
}

.code-display {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.625rem;
  padding: 1rem;
  background: var(--dark-800);
  border: 2px solid $color-accent;
  border-radius: 8px;

  .user-code {
    font-size: 1.375rem;
    font-weight: 700;
    color: $color-accent;
    letter-spacing: 0.15em;
    font-family: "Courier New", "Courier", monospace;
  }

  .copy-btn {
    background: none;
    border: 1px solid $color-accent;
    border-radius: 5px;
    padding: 0.4375rem;
    color: $color-accent;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      background: color-mix(in srgb, $color-accent, 10%, transparent);
    }

    &:active {
      transform: scale(0.95);
    }
  }
}

.polling-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.625rem;
  padding: 0.875rem;
  background: color-mix(in srgb, $color-accent, 5%, transparent);
  border-radius: 6px;
  color: var(--text);
  font-size: 0.8125rem;

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid $color-border;
    border-top-color: $color-accent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
}

.device-code-actions {
  display: flex;
  justify-content: center;
  margin-top: 0.25rem;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
