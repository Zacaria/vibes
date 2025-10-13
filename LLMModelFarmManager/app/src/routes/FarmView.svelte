<script lang="ts">
  import FarmTree from '../components/FarmTree.svelte';
  import type { GameState } from '../lib/types';

  export let state: GameState;
</script>

<section class="farm-view">
  <h2>Data Centers</h2>
  <FarmTree dataCenters={state.data_centers} />
</section>

<style>
  .farm-view { display: flex; flex-direction: column; gap: 1rem; }
  h2 { margin: 0; }
</style>
