<script lang="ts">
import { AdvertisementBanner, PlayButton, ProfilesList, RamAllocation } from "$lib";
</script>

<div class="home-page">
  <AdvertisementBanner />
  <ProfilesList isGrid isSmall />
  <PlayButton />
  <RamAllocation />
</div>

<style lang="scss">
.home-page {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  max-height: 100%;
  background: $color-background;
  overflow: hidden;
}
</style>
