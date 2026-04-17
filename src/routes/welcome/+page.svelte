<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import { Input } from '$lib/components/ui/input/index';
  import * as Card from '$lib/components/ui/card/index';
  import ThemePicker from '$lib/components/ThemePicker.svelte';
  import {
    completeFirstRun,
    setAlias,
    setTheme,
  } from '$lib/api/settings';
  import { addDirectory, scanDirectory } from '$lib/api/directories';
  import { loadSettings, setSettings } from '$lib/state/settings.svelte';
  import { applyTheme } from '$lib/state/theme.svelte';
  import { goto } from '$app/navigation';
  import type { Theme } from '$lib/types';
  import { ChevronLeft, ChevronRight, FolderOpen, Sparkles } from '$lib/lucide';

  let step = $state(0);
  const total = 4;

  let alias = $state('');
  let theme = $state<Theme>('system');
  let folderPath = $state<string | null>(null);
  let scanSummary = $state<string | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function pickFolder() {
    const { open } = await import('@tauri-apps/plugin-dialog');
    busy = true;
    error = null;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        folderPath = selected;
        const dir = await addDirectory(selected);
        const result = await scanDirectory(dir.id);
        scanSummary = `Found ${result.matched.length} matched, ${result.unmatched.length} unmatched`;
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function next() {
    if (step === 1 && alias.trim()) {
      try { await setAlias(alias.trim()); } catch (err) { error = String(err); }
    }
    if (step === 2) {
      try { await setTheme(theme); applyTheme(theme); } catch (err) { error = String(err); }
    }
    if (step < total - 1) {
      step += 1;
    } else {
      await finish();
    }
  }

  async function skip() {
    if (step < total - 1) {
      step += 1;
    } else {
      await finish();
    }
  }

  function back() {
    if (step > 0) step -= 1;
  }

  async function finish() {
    busy = true;
    try {
      const next = await completeFirstRun();
      setSettings(next);
      await goto('/', { replaceState: true });
    } catch (err) {
      error = String(err);
      busy = false;
    }
  }

  function onThemeChange(t: Theme) {
    theme = t;
    applyTheme(t);
  }

  // Preload settings so applied state matches on wizard entry
  void loadSettings().then((s) => {
    if (s?.theme) {
      theme = s.theme;
      applyTheme(s.theme);
    }
    if (s?.alias) alias = s.alias;
  });
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-6 bg-background">
  <div class="w-full max-w-lg space-y-6">
    <div class="flex justify-center gap-2" aria-label="Progress">
      {#each Array(total) as _, i}
        <div
          class="h-1.5 rounded-full transition-all {i === step ? 'w-8 bg-primary' : i < step ? 'w-4 bg-primary/60' : 'w-4 bg-muted'}"
        ></div>
      {/each}
    </div>

    <Card.Root class="min-h-[320px]">
      <Card.Content class="pt-8 pb-4 space-y-6">
        {#if step === 0}
          <div class="text-center space-y-4">
            <div class="mx-auto size-14 rounded-full bg-accent flex items-center justify-center">
              <Sparkles class="size-7" />
            </div>
            <div class="space-y-2">
              <h1 class="text-2xl font-bold">Welcome to Rustflix</h1>
              <p class="text-sm text-muted-foreground">
                A quick setup so your films feel at home. You can skip anything.
              </p>
            </div>
          </div>
        {:else if step === 1}
          <div class="space-y-4">
            <div class="space-y-1">
              <h1 class="text-xl font-semibold">What should we call you?</h1>
              <p class="text-sm text-muted-foreground">Just a first name — used to greet you.</p>
            </div>
            <Input
              placeholder="Your name"
              bind:value={alias}
              autofocus
            />
          </div>
        {:else if step === 2}
          <div class="space-y-4">
            <div class="space-y-1">
              <h1 class="text-xl font-semibold">Pick a theme</h1>
              <p class="text-sm text-muted-foreground">You can change this later in Settings.</p>
            </div>
            <ThemePicker bind:value={theme} onchange={onThemeChange} />
          </div>
        {:else if step === 3}
          <div class="space-y-4">
            <div class="space-y-1">
              <h1 class="text-xl font-semibold">Where do your films live?</h1>
              <p class="text-sm text-muted-foreground">
                Point us at a folder and we'll look for films inside it.
              </p>
            </div>
            {#if folderPath}
              <div class="rounded-md border bg-muted/30 p-3 space-y-1">
                <div class="flex items-center gap-2 text-sm font-medium">
                  <FolderOpen class="size-4" />
                  <span class="truncate">{folderPath}</span>
                </div>
                {#if scanSummary}
                  <p class="text-xs text-muted-foreground">{scanSummary}</p>
                {/if}
              </div>
            {/if}
            <Button onclick={pickFolder} disabled={busy} variant="secondary">
              {busy ? 'Scanning…' : folderPath ? 'Pick a different folder' : 'Pick a folder'}
            </Button>
          </div>
        {/if}

        {#if error}
          <p class="text-red-600 text-sm">{error}</p>
        {/if}
      </Card.Content>
      <Card.Footer class="flex justify-between pb-6">
        <div>
          {#if step > 0}
            <Button variant="ghost" onclick={back} disabled={busy}>
              <ChevronLeft class="size-4" /> Back
            </Button>
          {/if}
        </div>
        <div class="flex gap-2">
          {#if step > 0 && step < total}
            <Button variant="ghost" onclick={skip} disabled={busy}>Skip</Button>
          {/if}
          <Button onclick={next} disabled={busy}>
            {step === 0 ? 'Get started' : step === total - 1 ? 'Finish' : 'Next'}
            {#if step !== total - 1}
              <ChevronRight class="size-4" />
            {/if}
          </Button>
        </div>
      </Card.Footer>
    </Card.Root>
  </div>
</div>
