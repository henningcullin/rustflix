<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import { Input } from '$lib/components/ui/input/index';
  import * as Card from '$lib/components/ui/card/index';
  import ThemePicker from '$lib/components/ThemePicker.svelte';
  import {
    resetFirstRun,
    setAlias,
    setTheme,
    setTmdbApiKey,
  } from '$lib/api/settings';
  import { loadSettings, setSettings } from '$lib/state/settings.svelte';
  import { applyTheme } from '$lib/state/theme.svelte';
  import { goto } from '$app/navigation';
  import type { Theme } from '$lib/types';
  import { onMount } from 'svelte';

  let alias = $state('');
  let apiKey = $state('');
  let theme = $state<Theme>('system');

  let savingAlias = $state(false);
  let savingKey = $state(false);
  let resetting = $state(false);

  let aliasMsg = $state<{ kind: 'ok' | 'err'; text: string } | null>(null);
  let keyMsg = $state<{ kind: 'ok' | 'err'; text: string } | null>(null);

  onMount(async () => {
    const s = await loadSettings();
    alias = s?.alias ?? '';
    apiKey = s?.tmdb_api_key ?? '';
    theme = s?.theme ?? 'system';
  });

  async function saveAlias() {
    savingAlias = true;
    aliasMsg = null;
    try {
      const next = await setAlias(alias);
      setSettings(next);
      aliasMsg = { kind: 'ok', text: 'Saved.' };
    } catch (err) {
      aliasMsg = { kind: 'err', text: String(err) };
    } finally {
      savingAlias = false;
    }
  }

  async function onThemeChange(t: Theme) {
    theme = t;
    applyTheme(t);
    try {
      const next = await setTheme(t);
      setSettings(next);
    } catch (err) {
      console.error(err);
    }
  }

  async function saveKey() {
    savingKey = true;
    keyMsg = null;
    try {
      const next = await setTmdbApiKey(apiKey);
      setSettings(next);
      keyMsg = { kind: 'ok', text: 'Saved.' };
    } catch (err) {
      keyMsg = { kind: 'err', text: String(err) };
    } finally {
      savingKey = false;
    }
  }

  async function redoSetup() {
    resetting = true;
    try {
      const next = await resetFirstRun();
      setSettings(next);
      await goto('/welcome', { replaceState: true });
    } catch (err) {
      console.error(err);
    } finally {
      resetting = false;
    }
  }
</script>

<div class="p-6 max-w-2xl mx-auto space-y-6">
  <h1 class="text-2xl font-bold">Settings</h1>

  <Card.Root>
    <Card.Header>
      <Card.Title>Profile</Card.Title>
      <Card.Description>How Rustflix should greet you.</Card.Description>
    </Card.Header>
    <Card.Content class="space-y-3">
      <Input placeholder="Your name" bind:value={alias} disabled={savingAlias} />
      {#if aliasMsg}
        <p class={aliasMsg.kind === 'ok' ? 'text-green-600 text-sm' : 'text-red-600 text-sm'}>
          {aliasMsg.text}
        </p>
      {/if}
    </Card.Content>
    <Card.Footer>
      <Button onclick={saveAlias} disabled={savingAlias}>
        {savingAlias ? 'Saving…' : 'Save'}
      </Button>
    </Card.Footer>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>Appearance</Card.Title>
      <Card.Description>Pick a theme. Saved automatically.</Card.Description>
    </Card.Header>
    <Card.Content>
      <ThemePicker bind:value={theme} onchange={onThemeChange} />
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>Services</Card.Title>
      <Card.Description>
        A free TMDb key lets us fetch film metadata and covers.
        Get one at <a class="underline" href="https://www.themoviedb.org/settings/api" target="_blank" rel="noreferrer">themoviedb.org</a>.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-3">
      <Input type="password" placeholder="your-tmdb-api-key" bind:value={apiKey} disabled={savingKey} />
      {#if keyMsg}
        <p class={keyMsg.kind === 'ok' ? 'text-green-600 text-sm' : 'text-red-600 text-sm'}>
          {keyMsg.text}
        </p>
      {/if}
    </Card.Content>
    <Card.Footer>
      <Button onclick={saveKey} disabled={savingKey}>
        {savingKey ? 'Saving…' : 'Save'}
      </Button>
    </Card.Footer>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>Advanced</Card.Title>
      <Card.Description>Re-run the first-time setup.</Card.Description>
    </Card.Header>
    <Card.Footer>
      <Button variant="secondary" onclick={redoSetup} disabled={resetting}>
        {resetting ? 'Resetting…' : 'Run setup again'}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>
