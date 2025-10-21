<script lang="ts">
  import type { GameState } from '../lib/types';

  export let state: GameState;
</script>

<section class="workloads">
  <h2>Workload Queues</h2>
  <table>
    <thead>
      <tr>
        <th>Name</th>
        <th>Type</th>
        <th>Tokens/hr</th>
        <th>SLA (ms)</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody>
      {#each state.workloads as workload}
        <tr>
          <td>{state.models.find((m) => m.id === workload.model_id)?.family ?? 'Model'}</td>
          <td>{workload.kind}</td>
          <td>{workload.tokens_per_hour_target.toLocaleString()}</td>
          <td>{workload.qos_sla_ms}</td>
          <td>{workload.active ? 'Active' : 'Paused'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>

<style>
  .workloads { display: flex; flex-direction: column; gap: 1rem; }
  table { width: 100%; border-collapse: collapse; }
  th, td { padding: 0.75rem; text-align: left; border-bottom: 1px solid rgba(255,255,255,0.08); }
  th { text-transform: uppercase; font-size: 0.75rem; letter-spacing: 0.08em; color: var(--muted); }
</style>
