<script lang="ts">
  import type { GameState } from '../lib/types';

  export let state: GameState;
</script>

<section class="events">
  <h2>Event Log</h2>
  <ul>
    {#each [...state.events].reverse() as event}
      <li>
        <strong>{event.title}</strong>
        <span>{event.description}</span>
        <span class="timestamp">T+{event.ts_minutes}m</span>
      </li>
    {/each}
  </ul>
</section>

<style>
  .events { display: flex; flex-direction: column; gap: 1rem; }
  ul { list-style: none; padding-left: 0; margin: 0; display: grid; gap: 0.75rem; }
  li { background: var(--panel); padding: 0.75rem; border-radius: 10px; display: flex; flex-direction: column; gap: 0.25rem; }
  .timestamp { font-size: 0.75rem; color: var(--muted); }
</style>
