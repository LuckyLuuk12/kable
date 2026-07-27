<!--
@component 
A simple card component providing basic information and actions for a not-installed project. Used in the @ProjectsBrowser component.
Information shown includes:
- Name
- Author
- Cut-off/short description
Actions include:
- Install (to selected profile)
- Like/unlike 
- View versions (opens @ProjectVersionsModal)
- View details (opens @ProjectModal)
- (if shader/resourcepack type) View gallery (opens @ProjectGalleryModal)
-->
<script lang="ts">
let { project } = $props();
</script>

<div class="project-card"></div>

<style lang="scss">
</style>
