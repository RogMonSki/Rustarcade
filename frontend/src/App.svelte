<script>
  import init, { greet } from 'game-core';
  import wasmUrl from 'game-core/game_core_bg.wasm?url';

  let message = '';

  async function loadWasm() {
    try {
      await init(wasmUrl);
      message = greet();
    } catch (e) {
      message = 'Error: ' + e;
      console.error(e);
    }
  }

  loadWasm()
</script>

<main class="flex items-center justify-center h-screen bg-gray-900">
  <p class="text-white text-2xl">{message}</p>
</main>