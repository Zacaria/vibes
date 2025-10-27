<script lang="ts">
  export let values: number[] = [];
  export let color = '#16a085';

  const max = Math.max(...values, 1);
  const min = Math.min(...values, 0);
  const normalized = values.map((v, i) => {
    const x = (i / Math.max(values.length - 1, 1)) * 100;
    const y = values.length > 0 ? 100 - ((v - min) / (max - min || 1)) * 100 : 50;
    return `${x},${y}`;
  });
</script>

<svg viewBox="0 0 100 100" preserveAspectRatio="none" role="img" aria-label="trend chart">
  <polyline fill="none" stroke={color} stroke-width="4" points={normalized.join(' ')} />
</svg>

<style>
  svg {
    width: 100%;
    height: 60px;
  }
</style>
