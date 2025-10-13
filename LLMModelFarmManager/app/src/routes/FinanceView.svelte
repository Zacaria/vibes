<script lang="ts">
  import Sparkline from '../components/Sparkline.svelte';
  import type { GameState } from '../lib/types';

  export let state: GameState;

  let balances: number[] = [];
  $: balances = state.ledger.slice(-50).map((entry) => entry.balance);
</script>

<section class="finance">
  <h2>Cash Flow</h2>
  <Sparkline values={balances} color="#8e44ad" />
  <table>
    <thead>
      <tr>
        <th>Time</th>
        <th>Description</th>
        <th>Delta</th>
        <th>Balance</th>
      </tr>
    </thead>
    <tbody>
      {#each [...state.ledger].reverse().slice(0, 25) as entry}
        <tr>
          <td>{(entry.ts_minutes / 60).toFixed(1)}h</td>
          <td>{entry.description}</td>
          <td class:positive={entry.delta_cash >= 0}>€{entry.delta_cash.toFixed(2)}</td>
          <td>€{entry.balance.toFixed(2)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>

<style>
  .finance { display: flex; flex-direction: column; gap: 1rem; }
  table { width: 100%; border-collapse: collapse; }
  th, td { padding: 0.75rem; text-align: left; border-bottom: 1px solid rgba(255,255,255,0.08); }
  th { text-transform: uppercase; font-size: 0.75rem; letter-spacing: 0.08em; color: var(--muted); }
  .positive { color: #2ecc71; }
</style>
