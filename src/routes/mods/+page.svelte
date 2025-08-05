<script lang="ts">
    import InstallationMods from "$lib/components/InstallationMods.svelte";


</script>
<div class="mods">
    <InstallationMods />
</div>

<style lang="scss">
    @use "@kablan/clean-ui/scss/_variables.scss" as *;
    .mods {
        max-width: 100vw;
        display: flex;
        flex-wrap: wrap;
        height: 100vh;
    }
</style>

