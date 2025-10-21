<script lang="ts">
  import type { DataCenter } from '../lib/types';

  export let dataCenters: DataCenter[] = [];
</script>

<ul class="datacenters">
  {#each dataCenters as dc}
    <li>
      <details open>
        <summary>
          <span class="title">{dc.name}</span>
          <span class="meta">PUE {dc.pue.toFixed(2)} · {dc.region}</span>
        </summary>
        <ul>
          {#each dc.racks as rack}
            <li>
              <details>
                <summary>
                  <span class="title">{rack.name}</span>
                  <span class="meta">{rack.gpus.length}/{rack.slots} GPUs</span>
                </summary>
                <ul class="gpu-list">
                  {#each rack.gpus as gpu}
                    <li>
                      <span>{gpu.name}</span>
                      <span class="meta">{Math.round(gpu.utilization * 100)}% util · {gpu.hw.vram_gb}GB</span>
                    </li>
                  {/each}
                </ul>
              </details>
            </li>
          {/each}
        </ul>
      </details>
    </li>
  {/each}
</ul>

<style>
  ul { list-style: none; padding-left: 0; margin: 0; }
  .datacenters > li { margin-bottom: 1rem; }
  summary { cursor: pointer; display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0; }
  .title { font-weight: 600; }
  .meta { color: var(--muted); font-size: 0.85rem; margin-left: 0.5rem; }
  .gpu-list li { display: flex; justify-content: space-between; padding: 0.25rem 0; }
</style>
