<script lang="ts">
export let variant: 'default' | 'primary' | 'secondary' = 'default';
export let padding: 'sm' | 'md' | 'lg' = 'md';
export let className = '';

const paddingClasses = {
  sm: 'p-2',
  md: 'p-4',
  lg: 'p-6'
};

const variantClasses = {
  default: 'bg-white/50 border border-gray-200',
  primary: 'bg-[var(--vm-secondary-300)] text-white',
  secondary: 'bg-gray-50 border border-gray-200'
};
</script>

<div 
  class={`
    rounded-lg shadow-sm hover:shadow-md transition-all duration-200
    ${paddingClasses[padding]}
    ${variantClasses[variant]}
    ${className}
    overflow-hidden
  `}
>
  <slot />
</div>
