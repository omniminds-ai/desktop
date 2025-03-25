<script lang="ts">
  export let text: string;

  function parseText(text: string) {
    const parts = [];
    let currentText = '';
    let inAppTag = false;
    let appName = '';

    for (let i = 0; i < text.length; i++) {
      if (text[i] === '<' && text.slice(i, i + 5) === '<app>') {
        if (currentText) {
          parts.push({ type: 'text', content: currentText });
          currentText = '';
        }
        inAppTag = true;
        i += 4; // Skip the <app> tag
      } else if (inAppTag && text.slice(i, i + 6) === '</app>') {
        parts.push({ type: 'app', content: appName.trim() });
        appName = '';
        inAppTag = false;
        i += 5; // Skip the </app> tag
      } else if (inAppTag) {
        appName += text[i];
      } else {
        currentText += text[i];
      }
    }

    if (currentText) {
      parts.push({ type: 'text', content: currentText });
    }

    return parts;
  }
</script>

<span class="inline-flex items-center gap-1">
  {#each parseText(text) as part}
    {#if part.type === 'text'}
      {part.content}
    {:else if part.type === 'app'}
      <span
        class="inline-flex items-center gap-1 px-1.5 py-0.5 border border-neutral-500/50 font-bold rounded-full text-sm">
        <img
          src="https://www.google.com/s2/favicons?domain={part.content.toLowerCase()}.com"
          alt="{part.content} icon"
          class="w-4 h-4" />
        {part.content}
      </span>
    {/if}
  {/each}
</span>
