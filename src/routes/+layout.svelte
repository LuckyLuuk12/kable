<script lang="ts">
  import '$lib/styles/global.scss';
  import { NavBar, TitleBar } from '$lib';
</script>

<TitleBar>
  <NavBar>
    <slot />
  </NavBar>
</TitleBar>

