<script>
    export let text;
    export let color = 'default'; // Default color
    export let size = 'md'; // Default size
    export let removable = false; // Option to include a close button
    export let onRemove = () => {
    }; // Callback for removal
</script>

<div
        class="inline-flex items-center justify-center
         rounded-full px-3 py-1 font-100
         text-white
         {color === 'default' ? 'bg-primary-300/70' : ''}
         {color === 'blue' ? 'bg-blue-100 text-blue-800' : ''}
         {color === 'green' ? 'bg-green-100 text-green-800' : ''}
         {color === 'red' ? 'bg-red-100 text-red-800' : ''}
         {size === 'sm' ? 'text-xs px-2 py-0.5' : ''}
         {size === 'lg' ? 'text-base px-4 py-1.5' : ''}"
>
    {text}
    {#if removable}
        <button
                on:click={onRemove}
                class="ml-2 text-gray-400 hover:text-gray-600 focus:outline-none"
        >
            &times;
        </button>
    {/if}
</div>