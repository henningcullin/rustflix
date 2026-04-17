<script lang="ts">
  import { Button } from '$lib/components/ui/button/index';
  import { Input } from '$lib/components/ui/input/index';
  import * as Card from '$lib/components/ui/card/index';
  import { setTmdbApiKey } from '$lib/api/settings';
  import { loadSettings, setSettings } from '$lib/state/settings.svelte';
  import { onMount } from 'svelte';

  let apiKey = $state('');
  let saving = $state(false);
  let message = $state<{ kind: 'ok' | 'err'; text: string } | null>(null);

  onMount(async () => {
    const s = await loadSettings();
    apiKey = s?.tmdb_api_key ?? '';
  });

  async function save() {
    saving = true;
    message = null;
    try {
      const next = await setTmdbApiKey(apiKey);
      setSettings(next);
      message = { kind: 'ok', text: 'Saved.' };
    } catch (err) {
      message = { kind: 'err', text: String(err) };
    } finally {
      saving = false;
    }
  }
</script>

<div class="p-6 max-w-2xl">
  <h1 class="text-2xl font-bold mb-4">Settings</h1>

  <Card.Root>
    <Card.Header>
      <Card.Title>TMDb API key</Card.Title>
      <Card.Description>
        Used to fetch film metadata and cover art. Get a free key at
        <a class="underline" href="https://www.themoviedb.org/settings/api"
           target="_blank" rel="noreferrer">themoviedb.org</a>.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-4">
      <Input
        type="password"
        placeholder="your-tmdb-api-key"
        bind:value={apiKey}
        disabled={saving}
      />
      {#if message}
        <p class={message.kind === 'ok' ? 'text-green-600 text-sm' : 'text-red-600 text-sm'}>
          {message.text}
        </p>
      {/if}
    </Card.Content>
    <Card.Footer>
      <Button onclick={save} disabled={saving}>
        {saving ? 'Saving…' : 'Save'}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>
