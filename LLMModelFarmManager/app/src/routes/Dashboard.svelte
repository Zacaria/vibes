<script lang="ts">
  import type { Readable } from 'svelte/store';
  import KpiCard from '../components/KpiCard.svelte';
  import Sparkline from '../components/Sparkline.svelte';
  import type { GameState } from '../lib/types';

  export let state: GameState;
  export let kpiStore: Readable<{ cash: number; profit: number; tokens: number; uptime: number; carbon: number; power: number; speed: string; paused: boolean }>;

  let recentCash: number[] = [];
  let recentProfit: number[] = [];
  $: recentCash = state.ledger.slice(-20).map((entry) => entry.balance);
  $: recentProfit = state.ledger.slice(-20).map((entry) => entry.delta_cash);
</script>

<section class="dashboard">
  <div class="kpis">
    <KpiCard label="Cash" value={`€${state.cash.toLocaleString(undefined, { maximumFractionDigits: 0 })}`} accent="green" />
    <KpiCard label="Profit/hr" value={`€${state.profit_per_hour.toFixed(0)}`} accent="blue" />
    <KpiCard label="Tokens/hr" value={state.tokens_served_per_hour.toFixed(0)} accent="orange" />
    <KpiCard label="Uptime" value={`${(state.uptime * 100).toFixed(2)}%`} accent="purple" />
  </div>

  <div class="charts">
    <div>
      <h3>Cash Balance</h3>
      <Sparkline values={recentCash} color="#1abc9c" />
    </div>
    <div>
      <h3>Net Cash Flow</h3>
      <Sparkline values={recentProfit} color="#3498db" />
    </div>
    <p class="meta-line">Speed: {$kpiStore.speed}{#if $kpiStore.paused} (Paused){/if}</p>

  <div class="incidents">
    <h3>Active Incidents</h3>
    {#if state.active_incidents.length === 0}
      <p>All systems nominal.</p>
    {:else}
      <ul>
        {#each state.active_incidents as incident}
          <li>
            <strong>{incident.title}</strong>
            <span>{incident.description}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .meta-line { color: var(--muted); margin: 0; }
  .dashboard { display: flex; flex-direction: column; gap: 1.5rem; }
  .kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 1rem; }
  .charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; }
  .charts div { background: var(--panel); padding: 1rem; border-radius: 12px; }
  .charts h3 { margin-top: 0; }
  .incidents { background: var(--panel); padding: 1rem; border-radius: 12px; }
  ul { list-style: none; padding-left: 0; margin: 0; }
  li { display: flex; flex-direction: column; gap: 0.25rem; padding: 0.5rem 0; border-bottom: 1px solid rgba(255,255,255,0.05); }
  li:last-child { border-bottom: none; }
  strong { color: var(--accent); }
</style>
