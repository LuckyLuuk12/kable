<script lang="ts">
import { ShaderBrowser } from "$lib";
import type { KableInstallation, ShaderDownload } from "$lib";

// Handle shader download from browser
async function handleShaderDownload(event: {
  shader: ShaderDownload;
  installation: KableInstallation | null;
}) {
  const { shader, installation } = event;

  try {
    const { ShadersService } = await import("$lib");

    if (installation) {
      // Download to specific installation (dedicated mode)
      await ShadersService.downloadShaderToDedicated(shader, installation);
      console.log(
        `Successfully downloaded shader ${shader.name} to ${installation.name}`,
      );
    } else {
      // Download globally
      await ShadersService.downloadShaderGlobal(shader);
      console.log(`Successfully downloaded shader ${shader.name} globally`);
    }
  } catch (error) {
    console.error("Failed to download shader:", error);
    alert(`Failed to download shader: ${error}`);
  }
}
</script>

<div class="shaders-page">
  <ShaderBrowser ondownload={handleShaderDownload} />
</div>

<style lang="scss">
.shaders-page {
  height: 100%;
  width: 100%;
  box-sizing: border-box;
}
</style>
