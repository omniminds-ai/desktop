<script lang="ts">
  import type { Snippet, SvelteComponent } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    class?: string;
    variant?: 'dark' | 'light';
    icon?: Snippet;
  }

  let {
    class: className = '',
    variant = 'dark',
    value = $bindable(''),
    icon,
    ...rest
  }: Props = $props();

  const variantClasses = $derived.by(() => {
    let classes = '';
    switch (variant) {
      case 'dark':
        classes +=
          'bg-gray-800 text-white border-gray-700 focus:ring-secondary-300 focus:border-secondary-300';
        break;
      case 'light':
        classes +=
          'bg-gray-100 text-black border-gray-300 focus:ring-secondary-300 focus:border-secondary-300';
        break;

      default:
        break;
    }
    return classes;
  });
</script>

{#if icon}
  <div class="relative group">
    <div
      class="absolute text-gray-400 group-focus-within:text-secondary-300 inset-y-0 start-0 flex transition-all items-center ps-3 pointer-events-none">
      {@render icon?.()}
    </div>
    <input
      class="{variantClasses} w-full ps-12 py-2 px-3 rounded-lg border focus:outline-none focus:ring transition-all {className}"
      bind:value
      {...rest} />
  </div>
{:else}
  <input
    class="{variantClasses} w-full {icon
      ? 'ps-12'
      : ''} py-2 px-3 rounded-lg border focus:outline-none focus:ring transition-all {className}"
    bind:value
    {...rest} />
{/if}
