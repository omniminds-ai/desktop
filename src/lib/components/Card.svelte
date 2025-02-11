<script lang="ts">
  export let variant: 'default' | 'primary' | 'secondary' = 'default';
  export let padding: 'sm' | 'md' | 'lg' = 'md';
  export let className = '';

  const paddingClasses = {
    sm: 'p-2 sm:p-3',
    md: 'p-3 sm:p-4 md:p-5',
    lg: 'p-4 sm:p-5 md:p-6'
  };

  const variantClasses = {
    default: 'bg-white/50 border border-gray-300/75',
    primary: 'bg-secondary-300 text-white',
    secondary: 'bg-gray-50 border border-gray-300/75'
  };
</script>

<div
  class={`
    rounded-lg shadow-md transition-all duration-200
    ${paddingClasses[padding]}
    ${variantClasses[variant]}
    ${className}
    overflow-hidden
    w-full max-w-full
    break-words
    flex flex-col
  `}>
  <slot />
</div>
