<script lang="ts">
  import type { HTMLTextareaAttributes } from 'svelte/elements';

  interface Props extends HTMLTextareaAttributes {
    class?: string;
    variant?: 'dark' | 'light';
  }

  let { class: className, variant = 'dark', value = $bindable(''), ...rest }: Props = $props();

  const variantClasses = $derived.by(() => {
    let classes = '';
    switch (variant) {
      case 'dark':
        classes =
          'bg-gray-800 text-white border-gray-700 focus:ring-secondary-300 focus:border-secondary-300';
        break;

      case 'light':
        classes =
          'bg-gray-100 text-black border-gray-300 focus:ring-secondary-300 focus:border-secondary-300';
        break;

      default:
        break;
    }
    return classes;
  });
</script>

<textarea
  class="{variantClasses} w-full h-32 p-2 rounded-lg border focus:outline-none resize-none {className}"
  bind:value
  {...rest}>
</textarea>
