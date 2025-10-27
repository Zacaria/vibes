<script lang="ts">
  import type { GameState } from '../lib/types';

  export let state: GameState;

  let autosave = true;
  let language = 'en';
  let colorblind = false;
</script>

<section class="settings">
  <h2>Settings</h2>
  <form>
    <label>
      <span>Autosave</span>
      <input type="checkbox" bind:checked={autosave} />
    </label>
    <label>
      <span>Language</span>
      <select bind:value={language}>
        <option value="en">English</option>
        <option value="fr">Français</option>
      </select>
    </label>
    <label>
      <span>Colorblind Mode</span>
      <input type="checkbox" bind:checked={colorblind} />
    </label>
    <label>
      <span>Game Speed</span>
      <select bind:value={state.speed} disabled>
        <option value={state.speed}>{state.speed}</option>
      </select>
    </label>
  </form>
</section>

<style>
  .settings { display: flex; flex-direction: column; gap: 1rem; }
  form { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; background: var(--panel); padding: 1rem; border-radius: 12px; }
  label { display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.9rem; }
  span { font-weight: 600; }
</style>
