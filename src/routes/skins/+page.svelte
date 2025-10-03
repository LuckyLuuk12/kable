<script lang="ts">
    import SkinSelectionMenu from "$lib/components/skins/SkinSelectionMenu.svelte";

    let currentTab: 'selection' | 'editor' = 'selection';
</script>

<div class="skins-page">
    <!-- Tab Navigation -->
    <div class="tab-navigation">
        <button 
            class="tab-btn"
            class:active={currentTab === 'selection'}
            on:click={() => currentTab = 'selection'}
        >
            🎨 Skin Selection
        </button>
        <button 
            class="tab-btn"
            class:active={currentTab === 'editor'}
            on:click={() => currentTab = 'editor'}
        >
            ✏️ Skin Editor
        </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
        {#if currentTab === 'selection'}
            <SkinSelectionMenu />
        {:else if currentTab === 'editor'}
            <div class="editor-placeholder">
                <h2>Skin Editor</h2>
                <p>Advanced skin editing features will be implemented here.</p>
                <p>This will include tools for creating and modifying Minecraft skins.</p>
            </div>
        {/if}
    </div>
</div>

<style lang="scss">
    
    .skins-page {
        max-width: 100vw;
        max-height: 100vh;
        overflow: auto;
        display: flex;
        flex-direction: column;
    }
    
    .tab-navigation {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding-bottom: 0.5rem;
        background: var(--background);
    border-bottom: 1px solid color-mix(in srgb, var(--primary), 8%, transparent);
        
        .tab-btn {
            padding: 0.6rem 1.2rem;
            border: 1px solid var(--dark-600);
            border-radius: 0.5rem;
            background: var(--card);
            color: var(--text);
            font-weight: 500;
            font-size: 0.9em;
            cursor: pointer;
            transition: all 0.15s;
            
            &:hover {
                border-color: var(--primary);
                background: color-mix(in srgb, var(--primary), 5%, transparent);
            }
            
            &.active {
                background: linear-gradient(135deg, var(--primary) 0%, var(--secondary) 100%);
                color: var(--text-white);
                border-color: var(--text-transparent);
                box-shadow: 0 2px 8px color-mix(in srgb, var(--primary), 25%, transparent);
            }
        }
    }
    
    .tab-content {
        flex: 1;
        overflow: hidden;
    }

    .editor-placeholder {
        padding: 4rem 2rem;
        text-align: center;
        color: var(--placeholder);

        h2 {
            color: var(--text);
            margin-bottom: 1rem;
        }

        p {
            margin-bottom: 0.5rem;
        }
    }
    
    @media (max-width: 768px) {
        .tab-navigation {
            flex-direction: column;
            align-items: stretch;
            gap: 0.75rem;
        }
    }
</style>
