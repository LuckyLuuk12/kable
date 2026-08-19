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

let error = $state<string | null>(null);
let deviceCodeData = $state<Awaited<ReturnType<typeof app.authService.startAuth>> | null>(null);
let isPollingDeviceCode = $state(false);

let isAuthenticating = $derived(app.authService.authenticating);

function clearAuthenticationState() {
  deviceCodeData = null;
  isPollingDeviceCode = false;
}

onDestroy(clearAuthenticationState);

async function signInWithDeviceCode() {
  if (isAuthenticating || isPollingDeviceCode) return;

  error = null;
  isPollingDeviceCode = true;

  try {
    deviceCodeData = await app.authService.startAuth();

    await writeText(deviceCodeData.user_code);

    const token = await app.authService.pollAuth(deviceCodeData.device_code);

    await app.authService.authenticateAccount(token);

    clearAuthenticationState();
  } catch (err) {
    console.error("Device Code authentication failed:", err);

    error = err instanceof Error ? err.message : String(err);

    clearAuthenticationState();
  }
}

async function openVerificationUrl() {
  if (!deviceCodeData) return;

  try {
    await app.openUrl(deviceCodeData.verification_uri);
  } catch (err) {
    console.error("Failed to open verification URL:", err);
    error = "Failed to open the Microsoft authentication page.";
  }
}

async function copyUserCode() {
  if (!deviceCodeData) return;

  try {
    await writeText(deviceCodeData.user_code);
  } catch (err) {
    console.error("Failed to copy device code:", err);
    error = "Failed to copy the authentication code.";
  }
}

function cancelDeviceCode() {
  clearAuthenticationState();
}
</script>

<div class="auth-flow">
  {#if error}
    <div class="error-message">
      <Icon name="alert" size="sm" />
      <span>{error}</span>
    </div>
  {/if}

  {#if deviceCodeData}
    <div class="device-code-container">
      <div class="device-code-header">
        <div class="microsoft-logo">
          <Image key="microsoft-logo" alt="Microsoft" width="22px" height="22px" />
        </div>

        <div class="device-code-title">
          <h3>Sign in with Microsoft</h3>
          <p>Complete the authentication in your browser.</p>
        </div>
      </div>

      <div class="device-code-card">
        <div class="authentication-step">
          <div class="step-number">1</div>

          <div class="step-content">
            <span class="step-title">Open the Microsoft activation page</span>

            <button class="verification-link-btn" onclick={openVerificationUrl}>
              <Icon name="link" size="sm" />
              <span>{deviceCodeData.verification_uri}</span>
              <Icon name="external-link" size="sm" />
            </button>
          </div>
        </div>

        <div class="step-divider"></div>

        <div class="authentication-step">
          <div class="step-number">2</div>

          <div class="step-content">
            <span class="step-title">Enter the authentication code</span>

            <button class="code-display" onclick={copyUserCode} title="Copy authentication code">
              <code>{deviceCodeData.user_code}</code>

              <span class="copy-button">
                <Icon name="duplicate" size="sm" />
              </span>
            </button>

            <span class="code-hint"> The code has been copied to your clipboard. </span>
          </div>
        </div>

        {#if isPollingDeviceCode}
          <div class="polling-status">
            <div class="spinner"></div>

            <div>
              <span class="polling-title">Waiting for authentication</span>
              <span class="polling-description"> This window will continue automatically once you finish signing in. </span>
            </div>
          </div>
        {/if}

        <button class="cancel-button" onclick={cancelDeviceCode} disabled={isPollingDeviceCode}> Cancel </button>
      </div>
    </div>
  {:else}
    <div class="sign-in-container">
      <div class="sign-in-icon">
        <Image key="microsoft-logo" alt="Microsoft" width="28px" height="28px" />
      </div>

      <div class="sign-in-content">
        <h3>Add Microsoft account</h3>
        <p>Sign in with your Microsoft account to add it to Kable.</p>
      </div>

      <button class="btn-microsoft" onclick={signInWithDeviceCode} disabled={isAuthenticating}>
        <div class="microsoft-logo-large">
          <Image key="microsoft-logo" alt="Microsoft" width="21px" height="21px" />
        </div>

        <span>
          {isAuthenticating ? "Starting sign-in..." : "Sign in with Microsoft"}
        </span>
      </button>

      <div class="auth-disclaimer">
        <Icon name="lock" size="sm" />
        <span>Secure authentication via Microsoft</span>
      </div>
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
  align-items: flex-start;
  gap: 0.6rem;
  margin-bottom: 1rem;
  padding: 0.75rem 0.875rem;
  color: var(--red);
  background: color-mix(in srgb, var(--red) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--red) 35%, transparent);
  border-radius: var(--border-radius);
  font-size: 0.8rem;
  line-height: 1.4;
}

.sign-in-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 0;
  text-align: center;
}

.sign-in-icon {
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: 12px;
}

.sign-in-content {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;

  h3 {
    margin: 0;
    color: var(--text);
    font-size: 1rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--placeholder);
    font-size: 0.8rem;
  }
}

.btn-microsoft {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.7rem;
  min-width: 250px;
  padding: 0.7rem 1.25rem;
  color: #242424;
  background: #fff;
  border: 1px solid #8c8c8c;
  border-radius: 3px;
  font-family:
    "Segoe UI",
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition:
    background 0.12s ease,
    border-color 0.12s ease,
    transform 0.12s ease;

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
}

.microsoft-logo-large {
  display: flex;
  align-items: center;
  justify-content: center;
}

.auth-disclaimer {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin: 0;
  color: var(--placeholder);
  font-size: 0.7rem;
}

.device-code-container {
  width: 100%;
  max-width: 480px;
  margin: 0 auto;
}

.device-code-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.microsoft-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.device-code-title {
  min-width: 0;

  h3 {
    margin: 0 0 0.2rem;
    color: var(--text);
    font-size: 1rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--placeholder);
    font-size: 0.75rem;
  }
}

.device-code-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  background: $color-surface-3;
  border: 1px solid $color-border;
  border-radius: var(--border-radius);
}

.authentication-step {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.step-number {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: white;
  background: $color-accent;
  border-radius: 50%;
  font-size: 0.7rem;
  font-weight: 700;
}

.step-content {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 0;
  flex: 1;
}

.step-title {
  color: var(--text);
  font-size: 0.8rem;
  font-weight: 500;
}

.step-divider {
  height: 1px;
  margin-left: 36px;
  background: $color-border;
}

.verification-link-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.6rem 0.7rem;
  color: $color-accent;
  background: color-mix(in srgb, $color-accent 6%, transparent);
  border: 1px solid color-mix(in srgb, $color-accent 35%, transparent);
  border-radius: calc(var(--border-radius) - 2px);
  font-family: "SF Mono", "Monaco", "Consolas", monospace;
  font-size: 0.7rem;
  text-align: left;
  cursor: pointer;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;

  span {
    min-width: 0;
    flex: 1;
    overflow-wrap: anywhere;
  }

  &:hover {
    background: color-mix(in srgb, $color-accent 10%, transparent);
    border-color: color-mix(in srgb, $color-accent 55%, transparent);
  }
}

.code-display {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.8rem 0.9rem;
  color: $color-accent;
  background: $color-surface-3;
  border: 1px solid color-mix(in srgb, $color-accent 45%, transparent);
  border-radius: calc(var(--border-radius) - 2px);
  cursor: pointer;
  text-align: left;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;

  &:hover {
    background: color-mix(in srgb, $color-accent 6%, transparent);
    border-color: $color-accent;
  }

  &:active {
    transform: scale(0.99);
  }

  code {
    color: $color-accent;
    font-family: "SF Mono", "Monaco", "Consolas", "Courier New", monospace;
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: 0.15em;
  }
}

.copy-button {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: $color-accent;
  border: 1px solid color-mix(in srgb, $color-accent 40%, transparent);
  border-radius: calc(var(--border-radius) - 3px);
}

.code-hint {
  color: var(--placeholder);
  font-size: 0.7rem;
}

.polling-status {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  padding: 0.75rem;
  background: color-mix(in srgb, $color-accent 5%, transparent);
  border: 1px solid color-mix(in srgb, $color-accent 15%, transparent);
  border-radius: calc(var(--border-radius) - 2px);
}

.spinner {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  border: 2px solid $color-border;
  border-top-color: $color-accent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

.polling-status > div:last-child {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.polling-title {
  color: var(--text);
  font-size: 0.75rem;
  font-weight: 600;
}

.polling-description {
  color: var(--placeholder);
  font-size: 0.7rem;
  line-height: 1.35;
}

.cancel-button {
  align-self: center;
  padding: 0.4rem 0.8rem;
  color: var(--placeholder);
  background: transparent;
  border: none;
  border-radius: calc(var(--border-radius) - 2px);
  font-size: 0.75rem;
  cursor: pointer;
  transition:
    color 0.12s ease,
    background 0.12s ease;

  &:hover:not(:disabled) {
    color: var(--text);
    background: $color-surface-3;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
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
