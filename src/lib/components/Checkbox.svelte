<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends Omit<HTMLInputAttributes, 'type'> {
    class?: string;
    variant?: 'dark' | 'light';
    label?: string;
  }

  let {
    class: className = '',
    variant = 'dark',
    checked = $bindable(false),
    label = '',
    id = '',
    ...rest
  }: Props = $props();

  const variantClasses = $derived.by(() => {
    let classes = '';
    switch (variant) {
      case 'dark':
        classes = 'text-white';
        break;
      case 'light':
        classes = 'text-gray-700';
        break;
      default:
        break;
    }
    return classes;
  });
</script>

<div class="flex items-center">
  <input
    type="checkbox"
    {id}
    class="mr-2 h-4 w-4 text-secondary-600 rounded focus:ring-secondary-300 {className}"
    bind:checked
    {...rest} />
  {#if label}
    <label for={id} class="text-sm {variantClasses}">
      {label}
    </label>
  {/if}
</div>
