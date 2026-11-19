<script lang="ts">
  type InlinePart = {
    kind: 'text' | 'strong' | 'link';
    text: string;
    href?: string;
  };

  type ReleaseBlock =
    | { kind: 'heading'; level: number; content: InlinePart[] }
    | { kind: 'paragraph'; content: InlinePart[] }
    | { kind: 'list-item'; depth: number; content: InlinePart[] }
    | { kind: 'image'; alt: string; src: string };

  let { notes }: { notes: string } = $props();

  function parseInline(text: string): InlinePart[] {
    const parts: InlinePart[] = [];
    const markdownPattern = /\*\*([^*]+)\*\*|\[([^\]]+)\]\((https:\/\/[^)\s]+)\)/g;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = markdownPattern.exec(text)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ kind: 'text', text: text.slice(lastIndex, match.index) });
      }

      if (match[1]) {
        parts.push({ kind: 'strong', text: match[1] });
      } else {
        parts.push({ kind: 'link', text: match[2], href: match[3] });
      }

      lastIndex = markdownPattern.lastIndex;
    }

    if (lastIndex < text.length) {
      parts.push({ kind: 'text', text: text.slice(lastIndex) });
    }

    return parts;
  }

  function parseNotes(markdown: string): ReleaseBlock[] {
    const blocks: ReleaseBlock[] = [];
    let paragraphLines: string[] = [];

    function flushParagraph() {
      if (paragraphLines.length > 0) {
        blocks.push({ kind: 'paragraph', content: parseInline(paragraphLines.join(' ')) });
        paragraphLines = [];
      }
    }

    for (const line of markdown.split(/\r?\n/)) {
      if (!line.trim()) {
        flushParagraph();
        continue;
      }

      const heading = /^(#{1,3})\s+(.+)$/.exec(line);
      if (heading) {
        flushParagraph();
        blocks.push({
          kind: 'heading',
          level: heading[1].length,
          content: parseInline(heading[2])
        });
        continue;
      }

      const image = /^!\[([^\]]*)\]\((https:\/\/[^)\s]+)\)$/.exec(line.trim());
      if (image) {
        flushParagraph();
        blocks.push({ kind: 'image', alt: image[1], src: image[2] });
        continue;
      }

      const listItem = /^(\s*)-\s+(.+)$/.exec(line);
      if (listItem) {
        flushParagraph();
        blocks.push({
          kind: 'list-item',
          depth: Math.floor(listItem[1].length / 2),
          content: parseInline(listItem[2])
        });
        continue;
      }

      paragraphLines.push(line.trim());
    }

    flushParagraph();
    return blocks;
  }

  let blocks = $derived(parseNotes(notes));
</script>

<div class="release-notes-preview">
  {#each blocks as block}
    {#if block.kind === 'heading'}
      <svelte:element this={`h${block.level}`} class="release-notes-heading">
        {#each block.content as part}
          {#if part.kind === 'strong'}
            <strong>{part.text}</strong>
          {:else}
            {part.text}
          {/if}
        {/each}
      </svelte:element>
    {:else if block.kind === 'paragraph'}
      <p>
        {#each block.content as part}
          {#if part.kind === 'strong'}
            <strong>{part.text}</strong>
          {:else if part.kind === 'link'}
            <span class="release-notes-link" title={part.href}>{part.text}</span>
          {:else}
            {part.text}
          {/if}
        {/each}
      </p>
    {:else if block.kind === 'list-item'}
      <div class="release-notes-list-item" style={`margin-left: ${block.depth * 16}px`}>
        <span aria-hidden="true">•</span>
        <span>
          {#each block.content as part}
            {#if part.kind === 'strong'}
              <strong>{part.text}</strong>
            {:else if part.kind === 'link'}
              <span class="release-notes-link" title={part.href}>{part.text}</span>
            {:else}
              {part.text}
            {/if}
          {/each}
        </span>
      </div>
    {:else}
      <img src={block.src} alt={block.alt} loading="lazy" referrerpolicy="no-referrer" />
    {/if}
  {/each}
</div>

<style>
  .release-notes-preview {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 54vh;
    overflow: auto;
    padding: 12px;
    border-radius: var(--r-sm);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 0.86rem;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }

  .release-notes-preview :global(p) {
    margin: 0;
  }

  .release-notes-heading {
    margin: 6px 0 0;
    color: var(--text-primary);
    font-size: 1rem;
  }

  h1.release-notes-heading {
    font-size: 1.15rem;
  }

  .release-notes-list-item {
    display: flex;
    gap: 8px;
  }

  .release-notes-link {
    color: var(--accent);
    text-decoration: underline;
    text-decoration-color: var(--accent-muted);
  }

  img {
    display: block;
    width: auto;
    max-width: 100%;
    max-height: 280px;
    object-fit: contain;
    border-radius: var(--r-sm);
  }
</style>
