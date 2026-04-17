<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import {
    Maximize,
    Minimize,
    Pause,
    Play,
    Volume2,
    VolumeX,
  } from '$lib/lucide';
  import { setLeftOffPoint, streamUrl } from '$lib/api/films';
  import { onDestroy, onMount } from 'svelte';

  let {
    filmId,
    resumeFrom = 0,
  }: { filmId: number; resumeFrom?: number } = $props();

  let video = $state<HTMLVideoElement | null>(null);
  let container = $state<HTMLDivElement | null>(null);

  let playing = $state(false);
  let muted = $state(false);
  let volume = $state(1);
  let current = $state(0);
  let duration = $state(0);
  let fullscreen = $state(false);
  let controlsVisible = $state(true);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  let lastSaved = 0;

  function formatTime(seconds: number) {
    if (!isFinite(seconds) || seconds < 0) return '0:00';
    const s = Math.floor(seconds % 60);
    const m = Math.floor((seconds / 60) % 60);
    const h = Math.floor(seconds / 3600);
    if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
    return `${m}:${String(s).padStart(2, '0')}`;
  }

  function togglePlay() {
    if (!video) return;
    if (video.paused) void video.play();
    else video.pause();
  }

  function toggleMute() {
    if (!video) return;
    video.muted = !video.muted;
    muted = video.muted;
  }

  function onVolumeInput(e: Event) {
    const v = Number((e.target as HTMLInputElement).value);
    volume = v;
    if (video) video.volume = v;
  }

  function onSeek(e: Event) {
    const t = Number((e.target as HTMLInputElement).value);
    if (video) video.currentTime = t;
  }

  async function toggleFullscreen() {
    if (!container) return;
    if (!document.fullscreenElement) {
      await container.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  }

  function onFsChange() {
    fullscreen = !!document.fullscreenElement;
  }

  function bumpControls() {
    controlsVisible = true;
    if (hideTimer) clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (playing) controlsVisible = false;
    }, 2000);
  }

  async function saveProgress() {
    if (!video) return;
    const t = Math.floor(video.currentTime);
    if (t > 15 && Math.abs(t - lastSaved) >= 2) {
      lastSaved = t;
      try {
        await setLeftOffPoint(filmId, t);
      } catch {
        // best-effort
      }
    }
  }

  onMount(() => {
    if (!video) return;
    video.volume = volume;
    if (resumeFrom > 0) {
      const handler = () => {
        if (video && video.readyState >= 1) {
          video.currentTime = Math.max(0, resumeFrom - 2);
          video.removeEventListener('loadedmetadata', handler);
        }
      };
      video.addEventListener('loadedmetadata', handler);
    }
    document.addEventListener('fullscreenchange', onFsChange);
  });

  onDestroy(() => {
    document.removeEventListener('fullscreenchange', onFsChange);
    if (hideTimer) clearTimeout(hideTimer);
    void saveProgress();
  });
</script>

<div
  bind:this={container}
  class="relative w-full bg-black rounded-lg overflow-hidden aspect-video group"
  onmousemove={bumpControls}
  onmouseleave={() => { if (playing) controlsVisible = false; }}
  role="region"
  aria-label="Video player"
>
  <video
    bind:this={video}
    class="w-full h-full"
    src={streamUrl(filmId)}
    preload="metadata"
    onclick={togglePlay}
    onplay={() => { playing = true; bumpControls(); }}
    onpause={() => { playing = false; controlsVisible = true; }}
    ontimeupdate={() => {
      if (video) {
        current = video.currentTime;
        void saveProgress();
      }
    }}
    onloadedmetadata={() => {
      if (video) duration = video.duration;
    }}
    ondurationchange={() => {
      if (video && isFinite(video.duration)) duration = video.duration;
    }}
  ></video>

  <div
    class="absolute inset-x-0 bottom-0 p-3 bg-gradient-to-t from-black/80 to-transparent transition-opacity {controlsVisible ? 'opacity-100' : 'opacity-0'}"
  >
    <input
      type="range"
      min="0"
      max={duration || 0}
      step="0.1"
      value={current}
      oninput={onSeek}
      class="w-full accent-white"
      aria-label="Seek"
    />
    <div class="flex items-center gap-2 mt-2 text-white">
      <Button variant="ghost" size="icon" onclick={togglePlay} class="text-white hover:bg-white/10">
        {#if playing}
          <Pause class="size-5" />
        {:else}
          <Play class="size-5" />
        {/if}
      </Button>
      <Button variant="ghost" size="icon" onclick={toggleMute} class="text-white hover:bg-white/10">
        {#if muted}
          <VolumeX class="size-5" />
        {:else}
          <Volume2 class="size-5" />
        {/if}
      </Button>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={volume}
        oninput={onVolumeInput}
        class="w-24 accent-white"
        aria-label="Volume"
      />
      <span class="text-sm tabular-nums">
        {formatTime(current)} / {formatTime(duration)}
      </span>
      <div class="flex-1"></div>
      <Button variant="ghost" size="icon" onclick={toggleFullscreen} class="text-white hover:bg-white/10">
        {#if fullscreen}
          <Minimize class="size-5" />
        {:else}
          <Maximize class="size-5" />
        {/if}
      </Button>
    </div>
  </div>
</div>
