<!--
@component
AutoUpdater - Handles checking for and installing application updates

Automatically checks for updates on mount and provides UI for manual update checks.
Displays update information including version numbers and allows users to install updates.

@example
```svelte
<AutoUpdater />
```
-->
<!--
	Auto-update component
	Handles checking for and installing updates
-->
<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { checkForUpdates, installUpdate, getCurrentVersion } from '$lib/api';

	const dispatch = createEventDispatcher();

	let currentVersion = '';
	let updateInfo: any = null;
	let isChecking = false;
	let isInstalling = false;
	let error = '';

	onMount(async () => {
		try {
			currentVersion = await getCurrentVersion();
		} catch (e) {
			console.error('Failed to get current version:', e);
		}
	});

	async function handleCheckForUpdates() {
		isChecking = true;
		error = '';
		
		try {
			updateInfo = await checkForUpdates();
		} catch (e) {
			error = `Failed to check for updates: ${e}`;
		} finally {
			isChecking = false;
		}
	}

	async function handleInstallUpdate() {
		if (!updateInfo) return;
		
		isInstalling = true;
		error = '';
		
		try {
			await installUpdate();
			// App will restart automatically after update
		} catch (e) {
			error = `Failed to install update: ${e}`;
			isInstalling = false;
		}
	}
</script>

<div class="updater-section">
	<div class="section-header">
		<h3>Auto-Update</h3>
		<p>Current version: <span class="version">{currentVersion}</span></p>
	</div>

	<div class="update-controls">
		<button 
			class="check-button" 
			on:click={handleCheckForUpdates}
			disabled={isChecking || isInstalling}
		>
			{#if isChecking}
				Checking...
			{:else}
				Check for Updates
			{/if}
		</button>

		{#if updateInfo}
			<div class="update-available">
				<h4>Update Available: v{updateInfo.version}</h4>
				{#if updateInfo.body}
					<div class="update-notes">
						<p><strong>Release Notes:</strong></p>
						<div class="notes-content">{updateInfo.body}</div>
					</div>
				{/if}
				<button 
					class="install-button" 
					on:click={handleInstallUpdate}
					disabled={isInstalling}
				>
					{#if isInstalling}
						Installing...
					{:else}
						Install Update
					{/if}
				</button>
			</div>
		{:else if !isChecking && currentVersion}
			<p class="up-to-date">You're running the latest version</p>
		{/if}

		{#if error}
			<div class="error">
				{error}
			</div>
		{/if}
	</div>
</div>

<style lang="scss">
	.updater-section {
		background: var(--card);
		border-radius: 0.5rem;
		padding: 1.5rem;
		margin-bottom: 1rem;
	}

	.section-header {
		margin-bottom: 1rem;
		
		h3 {
			margin: 0 0 0.5rem 0;
			color: var(--text);
			font-size: 1.125rem;
			font-weight: 600;
		}
		
		p {
			margin: 0;
			color: var(--text-muted);
			font-size: 0.875rem;
		}
		
		.version {
			color: var(--primary);
			font-weight: 500;
		}
	}

	.update-controls {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.check-button {
		background: var(--primary);
		color: white;
		border: none;
		border-radius: 0.375rem;
		padding: 0.75rem 1.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		align-self: flex-start;
		
		&:hover:not(:disabled) {
			background: var(--primary-hover, #{'color-mix(in srgb, var(--primary) 90%, black)'});
		}
		
		&:disabled {
			opacity: 0.6;
			cursor: not-allowed;
		}
	}

	.update-available {
		background: var(--container);
		border: 2px solid var(--primary);
		border-radius: 0.5rem;
		padding: 1rem;
		
		h4 {
			margin: 0 0 0.75rem 0;
			color: var(--primary);
			font-size: 1rem;
			font-weight: 600;
		}
	}

	.update-notes {
		margin-bottom: 1rem;
		
		p {
			margin: 0 0 0.5rem 0;
			color: var(--text);
			font-size: 0.875rem;
			font-weight: 500;
		}
		
		.notes-content {
			background: var(--card);
			border-radius: 0.375rem;
			padding: 0.75rem;
			color: var(--text-muted);
			font-size: 0.875rem;
			line-height: 1.5;
			white-space: pre-wrap;
			max-height: 10rem;
			overflow-y: auto;
		}
	}

	.install-button {
		background: var(--status-success);
		color: var(--text-white);
		border: none;
		border-radius: 0.375rem;
		padding: 0.75rem 1.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		
		&:hover:not(:disabled) {
			background: var(--status-success-hover);
		}
		
		&:disabled {
			opacity: 0.6;
			cursor: not-allowed;
		}
	}

	.up-to-date {
		color: var(--status-success);
		font-size: 0.875rem;
		font-weight: 500;
		margin: 0;
		padding: 0.5rem 0;
	}

	.error {
		background: var(--status-error-bg);
		border: 1px solid var(--status-error);
		border-radius: 0.375rem;
		padding: 0.75rem;
		color: var(--status-error);
		font-size: 0.875rem;
		line-height: 1.4;
	}
</style>
