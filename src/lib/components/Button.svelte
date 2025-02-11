<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

  type Props = (
    | (Omit<HTMLButtonAttributes, 'children'> & { href?: HTMLAnchorAttributes['href'] })
    | (Omit<HTMLAnchorAttributes, 'children'> & { type?: HTMLButtonAttributes['type'] })
  ) & {
    children: Snippet;
    variant?: 'primary' | 'destroy' | 'warning' | 'secondary';
  };

  let { variant = 'primary', href, type, class: className, children, ...rest }: Props = $props();

  const variantClasses = $derived.by(() => {
    let classes = '';
    switch (variant) {
      case 'primary':
        classes =
          'bg-secondary-300 text-white border-2 hover:bg-white hover:text-secondary-300 border-secondary-300';
        break;
      case 'secondary':
        classes =
          'bg-gray-200 text-gray-700 border-2 hover:bg-gray-300 border-gray-200 hover:border-gray-300';
        break;
      case 'warning':
        classes =
          'bg-yellow-500 text-white border-2 hover:bg-white hover:text-yellow-500 border-yellow-300';
        break;
      case 'destroy':
        classes = 'bg-red-500 text-white border-2 hover:bg-white hover:text-red-500 border-red-500';
        break;

      default:
        break;
    }
    return classes;
  });
</script>

{#if href}
  <a
    {href}
    {...rest as HTMLAnchorAttributes}
    class="{variantClasses} p-3 inline-block rounded disabled:bg-gray-300 disabled:border-gray-300 cursor-pointer disabled:cursor-default disabled:shadow-sm disabled:text-white transition-colors shadow-sm {className}">
    {@render children()}
  </a>
{:else}
  <button
    aria-label="button"
    {type}
    {...rest as HTMLButtonAttributes}
    class="{variantClasses} p-3 inline-block rounded disabled:bg-gray-300 disabled:border-gray-300 cursor-pointer disabled:cursor-default disabled:shadow-sm disabled:text-white transition-colors shadow-sm {className}">
    {@render children()}
  </button>
{/if}
