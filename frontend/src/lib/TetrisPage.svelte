<script>
  import { tick } from 'svelte';

  let { onback } = $props();

  const COLS = 10;
  const ROWS = 20;
  const CELL = 24;
  const NEXT_CELL = 18;

  // Auto-repeat timing for held movement keys, in logical frames (60/sec).
  const DAS_FRAMES = 10; // delay before repeat kicks in
  const REPEAT_FRAMES = 3; // frames between repeats once held
  const STEP_MS = 1000 / 60; // fixed logical frame duration; game-core's gravity_frames() assumes 60fps steps
  const MAX_STEPS_PER_FRAME = 5; // cap catch-up work after e.g. a backgrounded tab

  /** @type {HTMLCanvasElement | null} */
  let canvas = $state(null);
  /** @type {HTMLCanvasElement | null} */
  let nextCanvas = $state(null);
  let score = $state(0);
  let lines = $state(0);
  let level = $state(1);
  let isOver = $state(false);
  let isLoading = $state(true);
  let waiting = $state(true); // true until the first keypress

  // Plain object refs — not reactive, used across closures
  let gameRef = { current: null };
  let rafRef = { current: null };
  /** @type {Function | null} */
  let TetrisGameCtor = null;

  const COLORS = [
    '#3fd7e8', // I - cyan
    '#e0c93f', // O - yellow
    '#b06bd6', // T - purple
    '#4cc466', // S - green
    '#e0524b', // Z - red
    '#4d7fe0', // J - blue
    '#e07b00', // L - orange (site accent)
  ];

  const MOVE_LEFT = new Set(['ArrowLeft', 'KeyA']);
  const MOVE_RIGHT = new Set(['ArrowRight', 'KeyD']);
  const SOFT_DROP = new Set(['ArrowDown', 'KeyS']);
  const ROTATE_KEYS = new Set(['ArrowUp', 'KeyW', 'KeyX']);
  const HARD_DROP_KEYS = new Set(['Space']);
  const PREVENT_DEFAULT = new Set([
    'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space',
  ]);

  /** Codes currently held down, for DAS-style auto-repeat. */
  let held = new Set();
  let dasCounter = { current: 0 };
  let activeRepeatKey = { current: null };

  /** @param {KeyboardEvent} e */
  function onKeyDown(e) {
    const code = e.code;
    const isGameKey =
      MOVE_LEFT.has(code) || MOVE_RIGHT.has(code) || SOFT_DROP.has(code) ||
      ROTATE_KEYS.has(code) || HARD_DROP_KEYS.has(code);
    if (!isGameKey) return;
    if (PREVENT_DEFAULT.has(code)) e.preventDefault();

    waiting = false;
    const game = gameRef.current;
    if (!game || isOver) return;

    if (!held.has(code)) {
      held.add(code);
      if (MOVE_LEFT.has(code) || MOVE_RIGHT.has(code) || SOFT_DROP.has(code)) {
        dasCounter.current = 0;
        activeRepeatKey.current = code;
      }
      applyMove(game, code);
    }
  }

  /** @param {KeyboardEvent} e */
  function onKeyUp(e) {
    held.delete(e.code);
    if (activeRepeatKey.current === e.code) {
      activeRepeatKey.current = null;
    }
  }

  function applyMove(game, code) {
    if (MOVE_LEFT.has(code)) game.move_left();
    else if (MOVE_RIGHT.has(code)) game.move_right();
    else if (SOFT_DROP.has(code)) game.soft_drop();
    else if (ROTATE_KEYS.has(code)) game.rotate();
    else if (HARD_DROP_KEYS.has(code)) game.hard_drop();
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
        const x = c * CELL + 1;
        const y = r * CELL + 1;
        const w = CELL - 2;
        if (v >= 8) {
          // Ghost piece: dim outline only.
          ctx.strokeStyle = COLORS[v - 8];
          ctx.globalAlpha = 0.5;
          ctx.strokeRect(x + 1.5, y + 1.5, w - 3, w - 3);
          ctx.globalAlpha = 1;
        } else {
          ctx.fillStyle = COLORS[v - 1];
          ctx.fillRect(x, y, w, w);
        }
      }
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {Uint8Array} cells
   */
  function renderNext(ctx, cells) {
    ctx.fillStyle = '#0f0e0b';
    ctx.fillRect(0, 0, 4 * NEXT_CELL, 4 * NEXT_CELL);
    for (let r = 0; r < 4; r++) {
      for (let c = 0; c < 4; c++) {
        const v = cells[r * 4 + c];
        if (v === 0) continue;
        ctx.fillStyle = COLORS[v - 1];
        ctx.fillRect(c * NEXT_CELL + 1, r * NEXT_CELL + 1, NEXT_CELL - 2, NEXT_CELL - 2);
      }
    }
  }

  function startLoop(game, ctx, nextCtx) {
    let frameCount = 0;
    let accumulator = 0;
    let lastTime = null;

    function frame(timestamp) {
      if (lastTime === null) lastTime = timestamp;
      accumulator = Math.min(accumulator + (timestamp - lastTime), STEP_MS * MAX_STEPS_PER_FRAME);
      lastTime = timestamp;

      let alive = true;
      while (accumulator >= STEP_MS) {
        accumulator -= STEP_MS;
        frameCount++;

        if (!waiting) {
          // Gravity, paced by the level-dependent interval Rust computes.
          if (frameCount % game.gravity_frames() === 0) {
            alive = game.step();
            score = game.score();
            lines = game.lines();
            level = game.level();
            if (!alive) {
              isOver = true;
              break;
            }
          }

          // Auto-repeat for held movement keys (DAS).
          const code = activeRepeatKey.current;
          if (code !== null) {
            dasCounter.current++;
            if (
              dasCounter.current === DAS_FRAMES ||
              (dasCounter.current > DAS_FRAMES &&
                (dasCounter.current - DAS_FRAMES) % REPEAT_FRAMES === 0)
            ) {
              applyMove(game, code);
              score = game.score();
            }
          }
        }
      }

      render(ctx, game.cells());
      renderNext(nextCtx, game.next_cells());
      if (!alive) return;
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
    const game = new TetrisGameCtor();
    gameRef.current = game;
    score = 0;
    lines = 0;
    level = 1;
    isOver = false;
    waiting = true;
    held = new Set();
    activeRepeatKey.current = null;
    const ctx = canvas?.getContext('2d');
    const nextCtx = nextCanvas?.getContext('2d');
    if (ctx && nextCtx) startLoop(game, ctx, nextCtx);
  }

  $effect(() => {
    let cleanedUp = false;

    async function init() {
      const mod = await import('game-core');
      // Some wasm-bindgen versions emit a self-initializing "bundler"
      // module with no default init export; only call it if present.
      if (typeof mod.default === 'function') await mod.default();

      if (cleanedUp) return;

      TetrisGameCtor = mod.TetrisGame;
      const game = new mod.TetrisGame();
      gameRef.current = game;
      isLoading = false;
      await tick(); // wait for Svelte to mount the canvas elements

      const ctx = canvas?.getContext('2d');
      const nextCtx = nextCanvas?.getContext('2d');
      if (ctx && nextCtx) startLoop(game, ctx, nextCtx);
    }

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    init();

    return () => {
      cleanedUp = true;
      stopLoop();
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
      gameRef.current?.free();
      gameRef.current = null;
    };
  });
</script>

<div class="game-page">
  <header class="game-header">
    <button class="back-btn" onclick={onback}>← Back</button>
    <span class="game-title">TETRIS</span>
    <span class="score-display">Score: {score}</span>
  </header>

  <div class="play-area">
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

    {#if !isLoading}
      <aside class="side-panel">
        <div class="panel-block">
          <span class="panel-label">Next</span>
          <canvas
            class="next-canvas"
            bind:this={nextCanvas}
            width={4 * NEXT_CELL}
            height={4 * NEXT_CELL}
          ></canvas>
        </div>
        <div class="panel-block">
          <span class="panel-label">Score</span>
          <span class="panel-value">{score}</span>
        </div>
        <div class="panel-block">
          <span class="panel-label">Lines</span>
          <span class="panel-value">{lines}</span>
        </div>
        <div class="panel-block">
          <span class="panel-label">Level</span>
          <span class="panel-value">{level}</span>
        </div>
      </aside>
    {/if}
  </div>

  <p class="hint">Arrow keys or WASD to move/rotate, Space to hard drop</p>
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

  .play-area {
    display: flex;
    align-items: flex-start;
    gap: 20px;
  }

  .canvas-wrap {
    position: relative;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    line-height: 0;
    width: 240px;
    height: 480px;
  }

  canvas {
    display: block;
  }

  .side-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 120px;
  }

  .panel-block {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
  }

  .panel-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text);
    opacity: 0.6;
  }

  .panel-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-h);
  }

  .next-canvas {
    background: #0f0e0b;
    border-radius: 4px;
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

  @media (max-width: 460px) {
    .play-area {
      flex-direction: column;
      align-items: center;
    }

    .side-panel {
      flex-direction: row;
      flex-wrap: wrap;
      width: 240px;
      justify-content: center;
    }

    .panel-block {
      flex: 1;
      min-width: 70px;
    }
  }
</style>
