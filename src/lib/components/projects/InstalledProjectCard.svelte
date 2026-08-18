<!--
@component 
This card differs from the @ProjectCard component in that it is used to display an already-installed project.
This means it will provide different actions:
- Uninstall (from selected profile)
- View versions (opens @ProjectVersionsModal, this button will display "update" if a newer version is available)
- View details (opens @ProjectModal)
- enable/disable (for selected profile)
- (if shader/resourcepack type) View gallery (opens @ProjectGalleryModal)
It also displays less information than the @ProjectCard component, as it is assumed that the user has already seen the project details before installing it.
So we only show the name, version (and if it fits) the release date of the project.
-->
<script lang="ts">
import { app, type KableProfile, type KableProject_Deserialize } from "$lib";
let { profile, project }: { profile: KableProfile | null; project: KableProject_Deserialize } = $props();

// let isEnabled = $derived(async () => await app.projectsService.isEnabled(profile, project));
let isEnabled = $state(false);

$effect(() => {
  if (!profile || !project) return;
  let cancelled = false;

  app.projectsService.isEnabled(profile, project).then((value) => {
    if (!cancelled) {
      isEnabled = value;
    }
  });

  return () => {
    cancelled = true;
  };
});

async function toggle(event: Event | undefined = undefined) {
  if (event instanceof KeyboardEvent && event.key !== "Enter" && event.key !== " ") return;
  // if undefined event it is just the onclick event and we want to toggle the project enabled state
  if (!profile) return;
  await app.projectsService.toggle(profile, project);
  isEnabled = await app.projectsService.isEnabled(profile, project);
}
</script>

<div
  class="installed-project-card"
  title="Click to {isEnabled ? 'disable' : 'enable'} {project.project.title}"
  onclick={async (e) => await toggle(e)}
  onkeydown={async (e) => await toggle(e)}
  role="button"
  tabindex="0"
>
  <h3>{project.project.title}</h3>
  <p>Version: {project.version_id}</p>
  <p>Release Date: {project.project.versions.find((v) => v.id === project.version_id)?.date_published}</p>
</div>

<style lang="scss">
</style>
