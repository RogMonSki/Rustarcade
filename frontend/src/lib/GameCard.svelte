<script>
  let { title, description, onclick = null, thumbnail = null } = $props();
</script>

{#snippet thumbArea()}
  <div class="thumbnail" class:thumbnail--img={thumbnail}>
    {#if thumbnail}
      <img src={thumbnail} alt="{title} preview" />
    {:else}
      <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <rect x="2" y="6" width="20" height="12" rx="2"/>
        <path d="M8 12h2m1-1v2"/>
        <circle cx="16" cy="11.5" r="0.5" fill="currentColor"/>
        <circle cx="18" cy="12.5" r="0.5" fill="currentColor"/>
      </svg>
    {/if}
  </div>
{/snippet}

{#if onclick}
  <button class="card" {onclick}>
    {@render thumbArea()}
    <div class="info">
      <h3>{title}</h3>
      <p>{description}</p>
      <span class="badge badge--play">Play</span>
    </div>
  </button>
{:else}
  <div class="card">
    {@render thumbArea()}
    <div class="info">
      <h3>{title}</h3>
      <p>{description}</p>
      <span class="badge">Coming Soon</span>
    </div>
  </div>
{/if}

<style>
  .card {
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: border-color 0.2s, box-shadow 0.2s;
    /* Reset button styles when used as button */
    font: inherit;
    color: inherit;
    text-align: left;
    padding: 0;
    cursor: pointer;
    width: 100%;
  }

  .card:hover {
    border-color: var(--accent-border);
    box-shadow: 0 0 0 1px var(--accent-border), var(--shadow);
  }

  .thumbnail {
    aspect-ratio: 16 / 9;
    background: var(--bg);
    border-bottom: 1px dashed var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
    opacity: 0.5;
  }

  .thumbnail--img {
    opacity: 1;
    border-bottom-style: solid;
  }

  .thumbnail--img img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .info {
    padding: 16px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-h);
    font-family: var(--heading);
  }

  p {
    font-size: 14px;
    color: var(--text);
    line-height: 1.5;
  }

  .badge {
    margin-top: 4px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--accent);
    background: var(--accent-bg);
    border: 1px solid var(--accent-border);
    border-radius: 4px;
    padding: 2px 8px;
  }

  .badge--play {
    color: #f3f4f6;
    background: rgba(224, 123, 0, 0.3);
    border-color: var(--accent);
  }
</style>
