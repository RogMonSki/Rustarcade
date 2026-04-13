<script>
  import { tick } from 'svelte';

  let { onback } = $props();

  const COLS = 20;
  const ROWS = 20;
  const CELL = 20;
  const TICK_INTERVAL = 8; // tick every 8 frames ≈ 7.5 steps/sec at 60fps

  /** @type {HTMLCanvasElement | null} */
  let canvas = $state(null);
  let score = $state(0);
  let isOver = $state(false);
  let isLoading = $state(true);
  let waiting = $state(true); // true until the first keypress

  // Plain object refs — not reactive, used across closures
  let gameRef = { current: null };
  let rafRef = { current: null };
  /** @type {Function | null} */
  let SnakeGameCtor = null;

  const KEY_MAP = {
    ArrowUp: 0, KeyW: 0,
    ArrowRight: 1, KeyD: 1,
    ArrowDown: 2, KeyS: 2,
    ArrowLeft: 3, KeyA: 3,
  };

  const ARROW_KEYS = new Set(['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight']);

  /** @param {KeyboardEvent} e */
  function onKeyDown(e) {
    const dir = KEY_MAP[e.code];
    if (dir !== undefined) {
      if (ARROW_KEYS.has(e.code)) e.preventDefault();
      waiting = false;
      gameRef.current?.set_direction(dir);
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {Uint8Array} cells
   */
  function render(ctx, cells) {
    ctx.fillStyle = '#0f0e0b';
    ctx.fillRect(0, 0, COLS * CELL, ROWS * CELL);
    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        const v = cells[r * COLS + c];
        if (v === 0) continue;
        ctx.fillStyle = v === 3 ? '#e07b00'   // food
                      : v === 2 ? '#f3f4f6'   // head
                      :           '#9ca3af';  // body
        ctx.fillRect(c * CELL + 1, r * CELL + 1, CELL - 2, CELL - 2);
      }
    }
  }

  function startLoop(game, ctx) {
    let frameCount = 0;
    function frame() {
      frameCount++;
      if (!waiting && frameCount % TICK_INTERVAL === 0) {
        const alive = game.step();
        score = game.score();
        if (!alive) {
          isOver = true;
          render(ctx, game.cells());
          return;
        }
      }
      render(ctx, game.cells());
      rafRef.current = requestAnimationFrame(frame);
    }
    rafRef.current = requestAnimationFrame(frame);
  }

  function stopLoop() {
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
  }

  function restart() {
    stopLoop();
    gameRef.current?.free();
    // @ts-ignore
    const game = new SnakeGameCtor(COLS, ROWS);
    gameRef.current = game;
    score = 0;
    isOver = false;
    waiting = true;
    const ctx = canvas?.getContext('2d');
    if (ctx) startLoop(game, ctx);
  }

  $effect(() => {
    let cleanedUp = false;

    async function init() {
      const mod = await import('game-core');
      await mod.default();

      if (cleanedUp) return;

      SnakeGameCtor = mod.SnakeGame;
      const game = new mod.SnakeGame(COLS, ROWS);
      gameRef.current = game;
      isLoading = false;
      await tick(); // wait for Svelte to mount the canvas element

      const ctx = canvas?.getContext('2d');
      if (ctx) startLoop(game, ctx);
    }

    window.addEventListener('keydown', onKeyDown);
    init();

    return () => {
      cleanedUp = true;
      stopLoop();
      window.removeEventListener('keydown', onKeyDown);
      gameRef.current?.free();
      gameRef.current = null;
    };
  });
</script>

<div class="game-page">
  <header class="game-header">
    <button class="back-btn" onclick={onback}>← Back</button>
    <span class="game-title">SNAKE</span>
    <span class="score-display">Score: {score}</span>
  </header>

  <div class="canvas-wrap">
    {#if isLoading}
      <div class="status-overlay">
        <div class="start-hint">Loading…</div>
      </div>
    {:else}
      <canvas bind:this={canvas} width={COLS * CELL} height={ROWS * CELL}></canvas>
      {#if waiting && !isOver}
        <div class="status-overlay">
          <div class="start-hint">Press any key to start</div>
        </div>
      {/if}
      {#if isOver}
        <div class="status-overlay">
          <div class="over-title">Game Over</div>
          <div class="over-score">Score: {score}</div>
          <button class="play-again-btn" onclick={restart}>Play Again</button>
        </div>
      {/if}
    {/if}
  </div>

  <p class="hint">Arrow keys or WASD to move</p>
</div>

<style>
  .game-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding: 40px 24px 80px;
  }

  .game-header {
    display: flex;
    align-items: center;
    gap: 24px;
    width: 100%;
    max-width: 400px;
  }

  .back-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 14px;
    cursor: pointer;
    transition: border-color 0.2s, color 0.2s;
  }

  .back-btn:hover {
    border-color: var(--accent-border);
    color: var(--accent);
  }

  .game-title {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-h);
    flex: 1;
    text-align: center;
  }

  .score-display {
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
    min-width: 70px;
    text-align: right;
  }

  .canvas-wrap {
    position: relative;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    line-height: 0;
    width: 400px;
    height: 400px;
  }

  canvas {
    display: block;
  }

  /* Shared overlay used for loading, waiting, and game-over states */
  .status-overlay {
    position: absolute;
    inset: 0;
    background: rgba(15, 14, 11, 0.82);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
  }

  .start-hint {
    color: var(--text);
    font-size: 15px;
  }

  .over-title {
    font-size: 32px;
    font-weight: 700;
    color: var(--text-h);
    letter-spacing: 0.02em;
  }

  .over-score {
    font-size: 18px;
    color: var(--accent);
    margin-top: 8px;
  }

  .play-again-btn {
    margin-top: 4px;
    background: var(--accent);
    color: #0f0e0b;
    border: none;
    border-radius: 6px;
    padding: 10px 24px;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .play-again-btn:hover {
    opacity: 0.85;
  }

  .hint {
    font-size: 13px;
    color: var(--text);
    opacity: 0.5;
    margin: 0;
  }
</style>
