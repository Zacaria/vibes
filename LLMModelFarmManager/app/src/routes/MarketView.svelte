<script lang="ts">
  import { buyGpu } from '../lib/store';
  import type { GameState, Gpu } from '../lib/types';

  export let state: GameState;

  let gpuTemplates: Gpu[] = [];
  $: gpuTemplates = state.data_centers
    .flatMap((dc) => dc.racks.flatMap((rack) => rack.gpus))
    .reduce((acc: Gpu[], gpu) => {
      if (!acc.find((g) => g.name === gpu.name)) {
        acc.push(gpu);
      }
      return acc;
    }, []);
</script>

<section class="market">
  <h2>GPU Marketplace</h2>
  <div class="cards">
    {#each gpuTemplates as gpu}
      <article>
        <header>
          <h3>{gpu.name}</h3>
          <span class="price">€{gpu.cost_capex.toLocaleString()}</span>
        </header>
        <ul>
          <li>{gpu.hw.vram_gb} GB VRAM · {gpu.hw.arch}</li>
          <li>{Math.round(gpu.efficiency_tok_per_joule * 1000)} tok/kWh</li>
          <li>{(gpu.failure_rate * 100).toFixed(2)}% failure rate</li>
        </ul>
        <button on:click={() => buyGpu(state.data_centers[0]?.racks[0]?.id ?? '', gpu)}>Buy for Rack 1</button>
      </article>
    {/each}
  </div>
</section>

<style>
  .market { display: flex; flex-direction: column; gap: 1rem; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; }
  article { background: var(--panel); padding: 1rem; border-radius: 12px; display: flex; flex-direction: column; gap: 0.75rem; }
  header { display: flex; justify-content: space-between; align-items: center; }
  .price { font-weight: 600; }
  button { align-self: flex-start; }
</style>
