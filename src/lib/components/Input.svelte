<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    class?: string;
    variant?: 'dark' | 'light';
  }

  const {
    class: className = '',
    variant = 'dark',
    value = $bindable(''),
    ...rest
  }: Props = $props();

  console.log(variant);
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

<input
  class="{variantClasses} w-full py-2 px-3 rounded-lg border focus:outline-none focus:ring transition-all {className}"
  {...rest} />
