<script lang="ts">
  export let label: string;
  export let value: string;
  export let accent: 'green' | 'blue' | 'orange' | 'purple' = 'green';
</script>

<div class={`kpi-card ${accent}`}>
  <h3>{label}</h3>
  <p>{value}</p>
</div>

<style>
  .kpi-card {
    padding: 1rem;
    border-radius: 12px;
    color: white;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 140px;
  }
  .kpi-card.green { background: linear-gradient(135deg, #1abc9c, #16a085); }
  .kpi-card.blue { background: linear-gradient(135deg, #3498db, #2980b9); }
  .kpi-card.orange { background: linear-gradient(135deg, #f39c12, #d35400); }
  .kpi-card.purple { background: linear-gradient(135deg, #9b59b6, #8e44ad); }
  h3 { margin: 0; font-size: 0.9rem; text-transform: uppercase; letter-spacing: 0.08em; }
  p { margin: 0; font-size: 1.4rem; font-weight: 600; }
</style>
