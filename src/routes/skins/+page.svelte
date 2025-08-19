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
    @use "@kablan/clean-ui/scss/_variables.scss" as *;
    
    .skins-page {
        max-width: 100vw;
        max-height: 100vh;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }
    
    .tab-navigation {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding-bottom: 0.5rem;
        background: $background;
        border-bottom: 1px solid rgba($primary, 0.08);
        
        .tab-btn {
            padding: 0.6rem 1.2rem;
            border: 1px solid $dark-600;
            border-radius: 0.5rem;
            background: $card;
            color: $text;
            font-weight: 500;
            font-size: 0.9em;
            cursor: pointer;
            transition: all 0.15s;
            
            &:hover {
                border-color: $primary;
                background: rgba($primary, 0.05);
            }
            
            &.active {
                background: linear-gradient(135deg, $primary 0%, $secondary 100%);
                color: white;
                border-color: transparent;
                box-shadow: 0 2px 8px rgba($primary, 0.25);
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
        color: $placeholder;

        h2 {
            color: $text;
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
