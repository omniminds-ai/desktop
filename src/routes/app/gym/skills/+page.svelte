<script lang="ts">
  import { onMount } from 'svelte';
  import * as d3 from 'd3';
  import { getAppsForSkills } from '$lib/api/forge';
  import { svgToCssTransform, getPlatform } from '$lib/utils';

  type NodeData = {
    name: string;
    domain?: string;
    description?: string;
    icon_url?: string;
    children?: NodeData[];
    seen?: boolean;
    completed?: boolean;
    recordingId?: string;
    score?: number;
  };

  let container: HTMLDivElement | SVGSVGElement;
  let mainGroup: d3.Selection<any, unknown, null, undefined>;
  let isMacOS = false;
  let treeData: NodeData | null = null;

  // Initialize SVG-based implementation
  async function initSvgImplementation(data: NodeData) {
    const svg = container as SVGSVGElement;
    const width = 2000;
    const height = 1600;

    // Setup zoom behavior
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.2, 5])
      .on('zoom', (event) => {
        mainGroup.attr('transform', event.transform.toString());
      });

    // Create the SVG container with zoom
    const svgElement = d3
      .select(svg)
      .attr('width', '100%')
      .attr('height', '100%')
      .attr('viewBox', `${-width / 2} ${-height / 2} ${width} ${height}`)
      .call(zoom);

    // Create main group for zooming
    mainGroup = svgElement.append('g');

    // Create the hierarchy and setup force simulation
    const { nodes, links } = setupForceSimulation(data);

    // Add links
    const link = mainGroup
      .selectAll('.skill-tree-link')
      .data(links)
      .join('path')
      .attr('class', (d) => {
        const target = d.target as d3.HierarchyNode<NodeData>;
        return `skill-tree-link${target.data.completed ? ' completed' : ''}`;
      });

    // Add nodes
    const node = mainGroup
      .selectAll('.skill-tree-node')
      .data(nodes)
      .join('foreignObject')
      .attr('class', 'skill-tree-node')
      .attr('width', (d) => (d.children ? 160 : 140))
      .attr('height', (d) => (d.children ? 160 : 140))
      .html((d) => createNodeContent(d));

    // Update positions on each tick
    return () => {
      link.attr('d', (d) => {
        const sourceX = (d.source as any).x;
        const sourceY = (d.source as any).y;
        const targetX = (d.target as any).x;
        const targetY = (d.target as any).y;

        return `M${sourceX},${sourceY}
                C${(sourceX + targetX) / 2},${sourceY}
                ${(sourceX + targetX) / 2},${targetY}
                ${targetX},${targetY}`;
      });

      node.attr('transform', (d: any) => {
        const x = d.x - (d.children ? 80 : 70);
        const y = d.y - (d.children ? 80 : 70);
        return `translate(${x},${y})`;
      });
    };
  }

  // Initialize div-based implementation
  async function initDivImplementation(data: NodeData) {
    const div = container as HTMLDivElement;
    const width = 2000;
    const height = 1600;

    // Setup zoom behavior
    const zoom = d3
      .zoom<HTMLDivElement, unknown>()
      .scaleExtent([0.2, 5])
      .on('zoom', (event) => {
        mainGroup.style('transform', svgToCssTransform(event.transform.toString()));
      });

    // Create the container with zoom
    d3.select(div)
      .style('width', '100%')
      .style('height', '100%')
      .style('position', 'relative')
      .style('overflow', 'hidden')
      .call(zoom);

    // Create main group for zooming
    const mainGroupDiv = document.createElement('div');
    mainGroupDiv.style.position = 'absolute';
    mainGroupDiv.style.left = '50%';
    mainGroupDiv.style.top = '50%';
    mainGroupDiv.style.transform = 'translate(-50%, -50%)';
    div.appendChild(mainGroupDiv);
    mainGroup = d3.select(mainGroupDiv);

    // Create the hierarchy and setup force simulation
    const { nodes, links } = setupForceSimulation(data);

    // Create a container for links
    const linksContainer = document.createElement('div');
    linksContainer.style.position = 'absolute';
    linksContainer.style.inset = '0';
    linksContainer.style.pointerEvents = 'none';
    mainGroup.node()?.appendChild(linksContainer);

    // Add links as div elements
    const linkElements = links.map((l) => {
      const link = document.createElement('div');
      const target = l.target as d3.HierarchyNode<NodeData>;
      link.className = `skill-tree-link${target.data.completed ? ' completed' : ''}`;
      linksContainer.appendChild(link);
      return link;
    });

    // Add nodes as absolutely positioned divs
    const nodeElements = nodes.map((d) => {
      const node = document.createElement('div');
      node.className = 'skill-tree-node';
      node.style.position = 'absolute';
      node.style.width = d.children ? '160px' : '140px';
      node.style.height = d.children ? '160px' : '140px';
      node.style.transform = 'translate(-50%, -50%)';
      node.innerHTML = createNodeContent(d);
      mainGroup.node()?.appendChild(node);
      return node;
    });

    // Update positions on each tick
    return () => {
      // Update link positions
      linkElements.forEach((link, i) => {
        const linkData = links[i];
        const source = linkData.source as d3.HierarchyNode<NodeData>;
        const target = linkData.target as d3.HierarchyNode<NodeData>;
        const sourceX = (source as any).x;
        const sourceY = (source as any).y;
        const targetX = (target as any).x;
        const targetY = (target as any).y;

        const angle = Math.atan2(targetY - sourceY, targetX - sourceX) * (180 / Math.PI);
        const distance = Math.sqrt(Math.pow(targetX - sourceX, 2) + Math.pow(targetY - sourceY, 2));

        link.style.position = 'absolute';
        link.style.width = `${distance}px`;
        link.style.height = '6px';
        link.style.left = `${sourceX}px`;
        link.style.top = `${sourceY}px`;
        link.style.transformOrigin = '0 50%';
        link.style.transform = `rotate(${angle}deg)`;
      });

      // Update node positions
      nodeElements.forEach((node, i) => {
        const nodeData = nodes[i];
        node.style.left = `${(nodeData as any).x}px`;
        node.style.top = `${(nodeData as any).y}px`;
      });
    };
  }

  // Setup force simulation (shared between both implementations)
  function setupForceSimulation(data: NodeData) {
    const root = d3.hierarchy(data);
    const nodes = root.descendants();
    const links = root.links();

    // Set initial positions in a radial layout
    const radius = 600;
    nodes.forEach((node, i) => {
      if (node.depth === 0) {
        node.x = 0;
        node.y = 0;
      } else if (node.depth === 1) {
        const angle = (i / root.children!.length) * 2 * Math.PI;
        node.x = radius * Math.cos(angle);
        node.y = radius * Math.sin(angle);
      } else {
        const parent = node.parent!;
        const siblings = parent.children!;
        const angle = ((siblings.indexOf(node) + 0.5) / siblings.length) * 2 * Math.PI;
        const parentX = parent.x || 0;
        const parentY = parent.y || 0;
        const childRadius = 300;
        node.x = parentX + childRadius * Math.cos(angle);
        node.y = parentY + childRadius * Math.sin(angle);
      }
    });

    // Create force simulation
    d3.forceSimulation(nodes)
      .force(
        'link',
        d3
          .forceLink(links)
          .id((d) => (d as any).id)
          .distance((d) => {
            const source = d.source as d3.HierarchyNode<NodeData>;
            const target = d.target as d3.HierarchyNode<NodeData>;
            if (source.depth === 0 || target.depth === 0) return 350;
            return 300;
          })
          .strength(0.5)
      )
      .force(
        'charge',
        d3
          .forceManyBody()
          .strength((d) => ((d as d3.HierarchyNode<NodeData>).depth === 0 ? -3000 : -2000))
      )
      .force(
        'collide',
        d3
          .forceCollide()
          .radius((d) => ((d as d3.HierarchyNode<NodeData>).children ? 150 : 120))
          .strength(1)
      )
      .force('x', d3.forceX().strength(0.02))
      .force('y', d3.forceY().strength(0.02));

    return { nodes, links };
  }

  // Create node content (shared between both implementations)
  function createNodeContent(d: d3.HierarchyNode<NodeData>): string {
    if (d.children) {
      // App node
      const template = document.getElementById('app-node-template')?.cloneNode(true) as HTMLElement;
      if (template) {
        const icon = template.querySelector('.app-icon') as HTMLImageElement;
        if (icon && d.data.icon_url) {
          icon.src = d.data.icon_url;
        }
        const nameEl = template.querySelector('.app-name');
        if (nameEl) {
          nameEl.textContent = d.data.name;
        }
        if (d.data.seen) {
          template.className = template.className.replace(
            'border-neutral-200',
            'border-neutral-800'
          );
        }
        return template.outerHTML;
      }
    } else {
      // Task node
      const template = document
        .getElementById('task-node-template')
        ?.cloneNode(true) as HTMLElement;
      if (template) {
        const textEl = template.querySelector('.task-text');
        if (textEl) {
          textEl.textContent = d.data.name;
        }
        const parentIcon = d.parent?.data.icon_url;
        const icon = template.querySelector('.app-icon') as HTMLImageElement;
        if (icon && parentIcon) {
          icon.src = parentIcon;
        }
        if (template instanceof HTMLAnchorElement) {
          if (d.data.completed) {
            template.href = `/app/gym/history/${d.data.recordingId}`;
            const scoreEl = template.querySelector('.task-score');
            const unclaimedEl = template.querySelector('.task-unclaimed');
            if (d.data.score) {
              if (scoreEl) {
                scoreEl.textContent = `${d.data.score}%`;
              }
            } else if (unclaimedEl) {
              unclaimedEl.textContent = 'unclaimed';
            }
          } else {
            const parent = d.parent?.data;
            const appType = parent?.domain ? 'website' : 'executable';
            const appInfo = {
              type: appType,
              name: parent?.name || '',
              ...(appType === 'website' ? { url: `https://${parent?.domain}` } : {})
            };
            template.href = `/app/gym/chat?prompt=${encodeURIComponent(d.data.name)}&app=${encodeURIComponent(JSON.stringify(appInfo))}`;
          }
        }
        const border = template.querySelector('[data-border]');
        if (border) {
          if (d.data.completed) {
            border.className =
              'absolute inset-0 border-4 border-[#f0e0fc] hover:border-secondary-300 rounded-2xl transition-colors';
          } else {
            border.className =
              'absolute inset-0 border-4 border-[#f7edfd] bg-[#f7edfd]/50! hover:border-secondary-300 hover:bg-transparent! rounded-2xl transition-colors';
          }
        }
        return template.outerHTML;
      }
    }
    return '';
  }

  onMount(async () => {
    try {
      // Check platform
      isMacOS = (await getPlatform()) === 'macos';

      // Get apps and tasks with seen/completed info
      const apps = await getAppsForSkills();
      treeData = {
        name: 'Desktop',
        icon_url: 'https://viralmind.ai/favicon.png',
        children: apps.map((app) => ({
          name: app.name,
          domain: app.domain,
          description: app.description,
          icon_url: `https://s2.googleusercontent.com/s2/favicons?domain=${app.domain}&sz=64`,
          seen: app.seen,
          children: app.tasks.map((task) => ({
            name: task.prompt,
            completed: task.completed,
            recordingId: task.recordingId,
            score: task.score
          }))
        }))
      };

      if (!treeData) return;

      // Initialize based on platform
      const tickFn = isMacOS
        ? await initDivImplementation(treeData)
        : await initSvgImplementation(treeData);

      // Set up tick function for force simulation
      d3.forceSimulation(d3.hierarchy(treeData).descendants()).on('tick', tickFn);
    } catch (error) {
      console.error('Failed to initialize skill tree:', error);
    }
  });
</script>

<!-- Hidden templates for D3 to use -->
<div class="hidden">
  <!-- App node template -->
  <div class="" id="app-node-template">
    <div class="relative w-[160px] h-[160px] bg-white border-4 border-neutral-200 rounded-2xl p-4">
      <img class="app-icon absolute bottom-2 left-2 w-8 h-8" src="" alt="app icon" />
      <div class="h-[calc(100%-2rem)] flex flex-col">
        <div
          class="app-name font-medium text-neutral-800 text-balance line-clamp-2 text-[min(1.25rem,4vw)]">
        </div>
        <div
          class="app-desc text-neutral-600 mt-1 text-balance line-clamp-3 text-[min(0.875rem,3vw)]">
        </div>
      </div>
    </div>
  </div>

  <!-- Task node template -->
  <a id="task-node-template" class="" href="#">
    <div
      class="relative w-[140px] h-[140px] bg-white rounded-2xl p-4 hover:bg-gray-50
      transition-colors no-underline block">
      <div
        class="task-text font-medium text-neutral-800 text-balance line-clamp-4 text-[min(1rem,3.5vw)] h-[calc(100%-2rem)]">
      </div>
      <div class="absolute bottom-2 left-2 flex items-center gap-2">
        <img class="app-icon w-6 h-6" src="" alt="app icon" />
        <div class="task-score text-sm font-medium text-secondary-300"></div>
        <div class="task-unclaimed text-xs font-medium text-secondary-300 bg-[#f7edfd] px-1.5 py-0.5 rounded"></div>
      </div>
      <div class="absolute inset-0 border-4 rounded-2xl transition-colors" data-border></div>
    </div>
  </a>
</div>

<div
  class="w-full max-w-7xl mx-auto h-full rounded-lg shadow-md border-gray-300/75 border bg-white text-neutral-800 p-4 overflow-hidden"
  class:macos={isMacOS}>
  <div class="relative w-full h-[calc(100vh-10rem)] border border-neutral-200 rounded-lg">
    {#if isMacOS}
      <div bind:this={container} class="w-full h-full"></div>
    {:else}
      <svg bind:this={container} class="w-full h-full">
        <defs></defs>
      </svg>
    {/if}
  </div>
</div>

<style>
  /* Link styles for macOS (div-based) */
  :global(.macos .skill-tree-link) {
    background: #bc59fa;
    opacity: 0.1;
    border-radius: 3px;
    transition: opacity 0.3s;
  }

  :global(.macos .skill-tree-link.completed) {
    opacity: 0.1;
  }

  :global(.macos .skill-tree-link:hover) {
    opacity: 1;
  }

  /* Link styles for other platforms (SVG-based) */
  :global(:not(.macos) .skill-tree-link) {
    fill: none;
    stroke: #bc59fa;
    stroke-width: 6px;
    stroke-opacity: 0.1;
    stroke-linecap: round;
    stroke-linejoin: round;
    transition: stroke-opacity 0.3s;
  }

  :global(:not(.macos) .skill-tree-link.completed) {
    stroke-opacity: 0.1;
  }

  :global(:not(.macos) .skill-tree-link:hover) {
    stroke-opacity: 1;
  }

  :global(.skill-tree-node) {
    transition: transform 0.1s;
  }

  :global(.app-name),
  :global(.app-desc),
  :global(.task-text) {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    overflow: hidden;
    word-break: break-word;
  }
</style>
