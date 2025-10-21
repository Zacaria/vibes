<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Dashboard from './routes/Dashboard.svelte';
  import FarmView from './routes/FarmView.svelte';
  import WorkloadsView from './routes/WorkloadsView.svelte';
  import MarketView from './routes/MarketView.svelte';
  import FinanceView from './routes/FinanceView.svelte';
  import EventsView from './routes/EventsView.svelte';
  import SettingsView from './routes/SettingsView.svelte';
  import { cleanup, error, gameState, initGame, isLoading, kpiStore, saveSlots, setPaused, setSpeed } from './lib/store';
  import type { GameSpeed } from './lib/types';

  const tabs = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'farm', label: 'Farm' },
    { id: 'workloads', label: 'Workloads' },
    { id: 'market', label: 'Market' },
    { id: 'finance', label: 'Finance' },
    { id: 'events', label: 'Events' },
    { id: 'settings', label: 'Settings' }
  ];

  let activeTab = 'dashboard';

  onMount(() => {
    initGame();
  });

  onDestroy(() => cleanup());

  function handleSpeedChange(speed: GameSpeed) {
    setSpeed(speed);
  }

  function handlePauseToggle(paused: boolean) {
    setPaused(paused);
  }
</script>

<svelte:window on:keydown={(event) => {
  if (event.key === ' ') {
    handlePauseToggle(!$gameState.paused);
    event.preventDefault();
  }
}} />

<main class="app-shell">
  <aside class="sidebar">
    <h1>LLM Farm</h1>
    <nav>
      {#each tabs as tab}
        <button
          class:active={activeTab === tab.id}
          on:click={() => (activeTab = tab.id)}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
    <section class="controls">
      <h2>Game Speed</h2>
      <div class="speed-buttons">
        <button on:click={() => handleSpeedChange('Normal')} class:active={$gameState.speed === 'Normal'}>1x</button>
        <button on:click={() => handleSpeedChange('Double')} class:active={$gameState.speed === 'Double'}>2x</button>
        <button on:click={() => handleSpeedChange('Quadruple')} class:active={$gameState.speed === 'Quadruple'}>4x</button>
      </div>
      <button class="pause" on:click={() => handlePauseToggle(!$gameState.paused)}>
        {$gameState.paused ? 'Resume' : 'Pause'}
      </button>
      <div class="saves">
        <h3>Saves</h3>
        <ul>
          {#each $saveSlots as slot}
            <li>
              <span>{slot.name}</span>
              <span class="muted">{new Date(slot.created_at).toLocaleString()}</span>
            </li>
          {/each}
        </ul>
      </div>
    </section>
  </aside>

  <section class="content">
    {#if $isLoading}
      <div class="loading">Loading...</div>
    {:else}
      {#if activeTab === 'dashboard'}
        <Dashboard {kpiStore} state={$gameState} />
      {:else if activeTab === 'farm'}
        <FarmView state={$gameState} />
      {:else if activeTab === 'workloads'}
        <WorkloadsView state={$gameState} />
      {:else if activeTab === 'market'}
        <MarketView state={$gameState} />
      {:else if activeTab === 'finance'}
        <FinanceView state={$gameState} />
      {:else if activeTab === 'events'}
        <EventsView state={$gameState} />
      {:else if activeTab === 'settings'}
        <SettingsView state={$gameState} />
      {/if}
    {/if}
    {#if $error}
      <p class="error">{$error}</p>
    {/if}
  </section>
</main>
