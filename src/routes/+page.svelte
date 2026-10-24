<script lang="ts">
import { AdvertisementBanner, PlayButton, ProfilesList, RamAllocation } from "$lib";
</script>

<div class="page">
  <div class="page-body">
    <AdvertisementBanner />
    <ProfilesList isGrid isSmall />
    <PlayButton />
    <RamAllocation />
  </div>
</div>

<style lang="scss">
.page {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.page-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
}
</style>
