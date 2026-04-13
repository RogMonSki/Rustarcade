<script>
  import GameCard from './lib/GameCard.svelte';
  import SnakePage from './lib/SnakePage.svelte';

  /** @type {string | null} */
  let currentView = $state(null);

  const games = [
    {
      id: 'snake',
      title: 'Snake',
      description: 'The classic snake game. Eat food, grow longer, don\'t hit the walls.',
      thumbnail: '/thumbnails/snake.png',
    },
    {
      id: null,
      title: 'Coming Soon',
      description: 'A puzzle game with challenging levels powered by native-speed Rust logic.',
    },
    {
      id: null,
      title: 'Coming Soon',
      description: 'An action platformer running at full speed in your browser via WASM.',
    },
  ];
</script>

{#if currentView === 'snake'}
  <SnakePage onback={() => currentView = null} />
{:else}
  <div class="page">
    <!-- Hero -->
    <header class="hero">
      <h1>RUSTARCADE</h1>
      <p class="tagline">Play Rust-powered games right in your browser.</p>
      <div class="divider"></div>
    </header>

    <!-- About -->
    <section class="about">
      <h2>What is Rustarcade?</h2>
      <p>
        Rustarcade is a collection of games written in <span class="highlight">Rust</span> and
        compiled to <span class="highlight">WebAssembly</span>, so they run at near-native speed
        without any plugins or installs — just open the page and play.
      </p>
    </section>

    <!-- Games -->
    <section class="games">
      <h2>Games</h2>
      <div class="grid">
        {#each games as game}
          <GameCard
            title={game.title}
            description={game.description}
            thumbnail={game.thumbnail ?? null}
            onclick={game.id ? () => currentView = game.id : null}
          />
        {/each}
      </div>
    </section>
  </div>
{/if}

<style>
  .page {
    max-width: 960px;
    margin: 0 auto;
    padding: 0 24px 80px;
    display: flex;
    flex-direction: column;
    gap: 64px;
  }

  /* Hero */
  .hero {
    text-align: center;
    padding-top: 80px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .hero h1 {
    font-size: 72px;
    font-weight: 800;
    letter-spacing: 0.08em;
    color: var(--text-h);
    margin: 0;
    line-height: 1;
    text-shadow: 0 0 60px rgba(224, 123, 0, 0.25);
  }

  @media (max-width: 600px) {
    .hero h1 {
      font-size: 48px;
    }
  }

  .tagline {
    font-size: 20px;
    color: var(--text);
    margin: 0;
  }

  .divider {
    width: 64px;
    height: 3px;
    background: var(--accent);
    border-radius: 2px;
    margin-top: 8px;
    box-shadow: 0 0 16px var(--accent);
  }

  /* About */
  .about {
    max-width: 600px;
    margin: 0 auto;
    text-align: center;
  }

  .about h2 {
    margin-bottom: 12px;
  }

  .about p {
    color: var(--text);
    line-height: 1.7;
  }

  .highlight {
    color: var(--accent);
    font-weight: 600;
  }

  /* Games */
  .games h2 {
    margin-bottom: 24px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  @media (max-width: 768px) {
    .grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 480px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
