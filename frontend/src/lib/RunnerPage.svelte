<script>
  import { tick } from 'svelte';

  let { onback } = $props();

  const WORLD_W = 640;
  const WORLD_H = 200;

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
  let RunnerGameCtor = null;

  const JUMP_KEYS = new Set(['Space', 'ArrowUp', 'KeyW']);
  const DUCK_KEYS = new Set(['ArrowDown', 'KeyS']);
  const PREVENT_DEFAULT = new Set(['Space', 'ArrowUp', 'ArrowDown']);

  /** Codes currently held down, to ignore key-repeat auto-fire on jump. */
  let held = new Set();
  let groundOffset = { current: 0 };

  /** @param {KeyboardEvent} e */
  function onKeyDown(e) {
    const code = e.code;
    const isGameKey = JUMP_KEYS.has(code) || DUCK_KEYS.has(code);
    if (!isGameKey) return;
    if (PREVENT_DEFAULT.has(code)) e.preventDefault();

    waiting = false;
    const game = gameRef.current;
    if (!game || isOver) return;

    if (!held.has(code)) {
      held.add(code);
      if (JUMP_KEYS.has(code)) game.jump();
      else if (DUCK_KEYS.has(code)) game.set_ducking(true);
    }
  }

  /** @param {KeyboardEvent} e */
  function onKeyUp(e) {
    held.delete(e.code);
    if (DUCK_KEYS.has(e.code)) {
      gameRef.current?.set_ducking(false);
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {number} x
   * @param {number} y
   * @param {number} w
   * @param {number} h
   */
  function drawCactus(ctx, x, y, w, h) {
    ctx.fillStyle = '#9ca3af';
    ctx.fillRect(x, y, w, h);
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {number} x
   * @param {number} y
   * @param {number} w
   * @param {number} h
   * @param {number} frameCount
   */
  function drawBird(ctx, x, y, w, h, frameCount) {
    ctx.fillStyle = '#e07b00';
    // Body
    ctx.fillRect(x + w * 0.25, y + h * 0.35, w * 0.5, h * 0.4);
    // Wings flap on a frame counter — up or down triangle-ish rect pair
    const wingUp = Math.floor(frameCount / 8) % 2 === 0;
    const wingY = wingUp ? y : y + h * 0.5;
    ctx.fillRect(x, wingY, w * 0.3, h * 0.25);
    ctx.fillRect(x + w * 0.7, wingY, w * 0.3, h * 0.25);
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {number} x
   * @param {number} y
   * @param {number} w
   * @param {number} h
   * @param {boolean} airborne
   * @param {boolean} ducking
   * @param {number} frameCount
   */
  function drawDino(ctx, x, y, w, h, airborne, ducking, frameCount) {
    ctx.fillStyle = '#f3f4f6';
    if (ducking) {
      // Crouched silhouette: low body + head, no visible legs.
      ctx.fillRect(x, y + h * 0.3, w, h * 0.7);
      ctx.fillRect(x + w * 0.6, y, w * 0.5, h * 0.5);
      return;
    }

    // Body + tail
    ctx.fillRect(x, y + h * 0.15, w * 0.75, h * 0.55);
    ctx.fillRect(x - w * 0.25, y + h * 0.2, w * 0.3, h * 0.18); // tail
    // Head + snout
    ctx.fillRect(x + w * 0.45, y, w * 0.5, h * 0.35);
    ctx.fillRect(x + w * 0.75, y + h * 0.15, w * 0.25, h * 0.12); // snout

    // Legs: alternate on a frame counter, both extend straight when airborne.
    const legH = h * 0.3;
    const legY = y + h * 0.7;
    if (airborne) {
      ctx.fillRect(x + w * 0.1, legY, w * 0.18, legH);
      ctx.fillRect(x + w * 0.5, legY, w * 0.18, legH);
    } else {
      const frontUp = Math.floor(frameCount / 6) % 2 === 0;
      ctx.fillRect(x + w * 0.1, legY, w * 0.18, frontUp ? legH * 0.6 : legH);
      ctx.fillRect(x + w * 0.5, legY, w * 0.18, frontUp ? legH : legH * 0.6);
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {object} game
   * @param {number} frameCount
   */
  function render(ctx, game, frameCount) {
    ctx.fillStyle = '#0f0e0b';
    ctx.fillRect(0, 0, WORLD_W, WORLD_H);

    // Ground: a solid line plus scrolling dashes for a sense of motion.
    const groundY = game.ground_y();
    ctx.fillStyle = '#f3f4f6';
    ctx.fillRect(0, groundY, WORLD_W, 2);
    groundOffset.current = (groundOffset.current + game.speed()) % 24;
    ctx.globalAlpha = 0.35;
    for (let x = -groundOffset.current; x < WORLD_W; x += 24) {
      ctx.fillRect(x, groundY + 4, 10, 2);
    }
    ctx.globalAlpha = 1;

    const obs = game.obstacles();
    for (let i = 0; i < obs.length; i += 5) {
      const [ox, oy, ow, oh, kind] = [obs[i], obs[i + 1], obs[i + 2], obs[i + 3], obs[i + 4]];
      if (kind >= 3) drawBird(ctx, ox, oy, ow, oh, frameCount);
      else drawCactus(ctx, ox, oy, ow, oh);
    }

    drawDino(
      ctx,
      game.player_x(),
      game.player_y(),
      game.player_w(),
      game.player_h(),
      game.is_airborne(),
      game.is_ducking(),
      frameCount,
    );
  }

  function startLoop(game, ctx) {
    let frameCount = 0;
    function frame() {
      frameCount++;
      if (!waiting) {
        const alive = game.step();
        score = game.score();
        if (!alive) {
          isOver = true;
          render(ctx, game, frameCount);
          return;
        }
      }
      render(ctx, game, frameCount);
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
    const game = new RunnerGameCtor();
    gameRef.current = game;
    score = 0;
    isOver = false;
    waiting = true;
    held = new Set();
    groundOffset.current = 0;
    const ctx = canvas?.getContext('2d');
    if (ctx) startLoop(game, ctx);
  }

  $effect(() => {
    let cleanedUp = false;

    async function init() {
      const mod = await import('game-core');
      // Some wasm-bindgen versions emit a self-initializing "bundler"
      // module with no default init export; only call it if present.
      if (typeof mod.default === 'function') await mod.default();

      if (cleanedUp) return;

      RunnerGameCtor = mod.RunnerGame;
      const game = new mod.RunnerGame();
      gameRef.current = game;
      isLoading = false;
      await tick(); // wait for Svelte to mount the canvas element

      const ctx = canvas?.getContext('2d');
      if (ctx) startLoop(game, ctx);
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
    <span class="game-title">RUNNER</span>
    <span class="score-display">Score: {score}</span>
  </header>

  <div class="canvas-wrap">
    {#if isLoading}
      <div class="status-overlay">
        <div class="start-hint">Loading…</div>
      </div>
    {:else}
      <canvas bind:this={canvas} width={WORLD_W} height={WORLD_H}></canvas>
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

  <p class="hint">Space or Up to jump, Down to duck</p>
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
    max-width: 640px;
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
    width: 640px;
    height: 200px;
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
