<script lang="ts">
  import * as Card from '$lib/components/ui/card/index';
</script>

<div
  class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 p-4"
>
  <a href="/persons" class="block">
    <Card.Root class="pt-2 pb-8 hover:bg-gray-100 transition">
      <Card.Header>
        <Card.Title>Persons</Card.Title>
        <Card.Description>View Persons</Card.Description>
      </Card.Header>
    </Card.Root>
  </a>

  <a href="/films" class="block">
    <Card.Root class="pt-2 pb-8 hover:bg-gray-100 transition">
      <Card.Header>
        <Card.Title>Films</Card.Title>
        <Card.Description>View Films</Card.Description>
      </Card.Header>
    </Card.Root>
  </a>

  <!-- More cards... -->
</div>
