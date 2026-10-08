(function () {
  const canvas = document.getElementById("glcanvas");
  const frame = document.getElementById("game-frame");
  const fullscreenButton = document.getElementById("fullscreen");
  const status = document.getElementById("game-status");
  let pauseRequested = false;
  let stopSound;
  const playedSounds = new Set();

  miniquad_add_plugin({
    name: "wirespace_web",
    version: 1,
    register_plugin(imports) {
      imports.env.wirespace_take_pause_request = () => {
        const requested = pauseRequested;
        pauseRequested = false;
        return Number(requested);
      };
      const playSound = imports.env.audio_play_buffer;
      stopSound = imports.env.audio_source_stop;
      imports.env.audio_play_buffer = (sound, volume, repeat) => {
        playedSounds.add(sound);
        return playSound(sound, volume, repeat);
      };
    },
  });

  const pauseHidden = () => {
    pauseRequested = true;
    if (stopSound) {
      for (const sound of playedSounds) stopSound(sound);
      playedSounds.clear();
    }
  };
  window.addEventListener("blur", pauseHidden);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) pauseHidden();
  });

  canvas.addEventListener("pointerdown", () => canvas.focus());

  window.addEventListener("keydown", (event) => {
    if (document.activeElement === canvas &&
        ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Space", "Tab"].includes(event.code)) {
      event.preventDefault();
    }
  }, { capture: true });

  fullscreenButton.addEventListener("click", async () => {
    try {
      if (document.fullscreenElement) {
        await document.exitFullscreen();
      } else {
        await frame.requestFullscreen();
      }
      canvas.focus();
    } catch (error) {
      status.textContent = `Fullscreen unavailable: ${error.message}`;
    }
  });

  document.addEventListener("fullscreenchange", () => {
    fullscreenButton.textContent = document.fullscreenElement ? "Exit fullscreen" : "Fullscreen ↗";
  });

  window.addEventListener("error", (event) => {
    status.textContent = `Game error: ${event.message}`;
  });

  status.textContent = "Loading game...";
  if (typeof load === "function") {
    load("elite.wasm");
    const startedAt = performance.now();
    const reportReady = () => {
      if (canvas.width !== 300 || canvas.height !== 150) {
        status.textContent = "Click the game to focus. Keyboard controls are below.";
      } else if (performance.now() - startedAt > 10000) {
        status.textContent = "Game did not start. Check that WebGL is enabled, then reload.";
      } else {
        requestAnimationFrame(reportReady);
      }
    };
    requestAnimationFrame(reportReady);
  } else {
    status.textContent = "Game loader unavailable. Reload the page.";
  }
})();
