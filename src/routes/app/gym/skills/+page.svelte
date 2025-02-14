<script lang="ts">
    import { onMount } from 'svelte';
    import * as d3 from 'd3';
    import GymHeader from '$lib/components/gym/GymHeader.svelte';

    type NodeData = {
        name: string;
        domain?: string;
        description?: string;
        icon_url?: string;
        children?: NodeData[];
    };

    let svg: SVGSVGElement;
    let mainGroup: d3.Selection<SVGGElement, unknown, null, undefined>;

    const data: NodeData = {
        name: "Desktop",
        icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.microsoft.com&sz=64",
        children: [
            {
                name: "Booking.com",
                domain: "booking.com",
                description: "Travel booking platform",
                icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.booking.com&sz=64",
                children: [
                    { name: "Find 5-star resort in Bahamas" },
                    { name: "Find resort in Cabo" },
                    { name: "Book JFK to LAX flight" }
                ]
            },
            {
                name: "DoorDash",
                domain: "doordash.com",
                description: "Food delivery",
                icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.doordash.com&sz=64",
                children: [
                    { name: "Order flowers and chocolate" },
                    { name: "Order medication" },
                    { name: "Order boba for team" },
                    { name: "Order cooking ingredients" }
                ]
            },
            {
                name: "OpenTable",
                domain: "opentable.com",
                description: "Restaurant reservations",
                icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.opentable.com&sz=64",
                children: [
                    { name: "Book romantic dinner" },
                    { name: "Find sushi tonight" },
                    { name: "Reserve for group" }
                ]
            },
            {
                name: "Telegram",
                domain: "telegram.org",
                description: "Messaging platform",
                icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.telegram.org&sz=64",
                children: [
                    { name: "Create channel" },
                    { name: "Set admin roles" },
                    { name: "Configure anti-spam" }
                ]
            },
            {
                name: "Discord",
                domain: "discord.com",
                description: "Community platform",
                icon_url: "https://s2.googleusercontent.com/s2/favicons?domain=www.discord.com&sz=64",
                children: [
                    { name: "Setup roles" },
                    { name: "Configure verification" },
                    { name: "Setup price bot" }
                ]
            }
        ]
    };
    
    onMount(() => {
        const width = 2000; // Increased canvas size
        const height = 1600;

        // Setup zoom behavior
        const zoom = d3.zoom<SVGSVGElement, unknown>()
            .scaleExtent([0.2, 2]) // Allow more zoom out
            .on('zoom', (event) => {
                mainGroup.attr('transform', event.transform);
            });

        // Create the SVG container with zoom
        const svgElement = d3.select(svg)
            .attr("width", "100%")
            .attr("height", "100%")
            .attr("viewBox", `0 0 ${width} ${height}`)
            .call(zoom);

        // Create main group for zooming
        mainGroup = svgElement.append("g")
            .attr("transform", `translate(${width/2},${height/2})`);

        // Create the hierarchy
        const root = d3.hierarchy(data);
        const nodes = root.descendants();
        const links = root.links();

        // Set initial positions in a radial layout
        const radius = 600; // Much larger radius for the main circle
        nodes.forEach((node, i) => {
            if (node.depth === 0) {
                // Center the root node
                (node as any).x = 0;
                (node as any).y = 0;
            } else if (node.depth === 1) {
                // Position app nodes in a circle around root
                const angle = (i / root.children!.length) * 2 * Math.PI;
                (node as any).x = radius * Math.cos(angle);
                (node as any).y = radius * Math.sin(angle);
            } else {
                // Position task nodes in smaller circles around their parent app
                const parent = node.parent!;
                const siblings = parent.children!;
                const angle = ((siblings.indexOf(node) + 0.5) / siblings.length) * 2 * Math.PI;
                const parentX = (parent as any).x;
                const parentY = (parent as any).y;
                const childRadius = 300; // Much larger radius for task nodes
                (node as any).x = parentX + childRadius * Math.cos(angle);
                (node as any).y = parentY + childRadius * Math.sin(angle);
            }
        });

        // Create force simulation with initial positions
        const simulation = d3.forceSimulation(nodes)
            .force("link", d3.forceLink(links)
                .id(d => (d as any).id)
                .distance(d => {
                    // Much longer distance between app nodes and root
                    const source = d.source as d3.HierarchyNode<NodeData>;
                    const target = d.target as d3.HierarchyNode<NodeData>;
                    if (source.depth === 0 || target.depth === 0) return 350;
                    // Longer distance between app nodes and their tasks
                    return 300;
                })
                .strength(0.5)) // Reduced strength to allow more natural spacing
            .force("charge", d3.forceManyBody()
                .strength(d => (d as d3.HierarchyNode<NodeData>).depth === 0 ? -3000 : -2000)) // Much stronger repulsion
            .force("collide", d3.forceCollide()
                .radius(d => ((d as d3.HierarchyNode<NodeData>).children ? 150 : 120)) // Much larger collision radius
                .strength(1)) // Maximum collision strength
            .force("x", d3.forceX().strength(0.02)) // Very weak centering force
            .force("y", d3.forceY().strength(0.02))
            .on("tick", ticked);

        // Add links
        const link = mainGroup.selectAll(".skill-tree-link")
            .data(links)
            .join("path")
            .attr("class", "skill-tree-link");

        // Add nodes
        const node = mainGroup.selectAll(".skill-tree-node")
            .data(nodes)
            .join("foreignObject")
            .attr("class", "skill-tree-node")
            .attr("width", d => d.children ? 160 : 140)
            .attr("height", d => d.children ? 160 : 140)
            .html(d => {
                if (d.children) {
                    // App node
                    const template = document.getElementById('app-node-template')?.cloneNode(true) as HTMLElement;
                    if (template) {
                        const icon = template.querySelector('.app-icon') as HTMLImageElement;
                        if (icon && d.data.icon_url) {
                            icon.src = d.data.icon_url;
                        }
                        const nameEl = template.querySelector('.font-medium');
                        if (nameEl) {
                            nameEl.textContent = d.data.name;
                        }
                        const descEl = template.querySelector('.text-xs');
                        if (descEl) {
                            descEl.textContent = d.data.description || '';
                        }
                        return template.outerHTML;
                    }
                } else {
                    // Task node
                    const template = document.getElementById('task-node-template')?.cloneNode(true) as HTMLElement;
                    if (template) {
                        const textEl = template.querySelector('.text-md');
                        if (textEl) {
                            textEl.textContent = d.data.name;
                        }
                        // Find parent's icon URL
                        const parentIcon = d.parent?.data.icon_url;
                        const icon = template.querySelector('.app-icon') as HTMLImageElement;
                        if (icon && parentIcon) {
                            icon.src = parentIcon;
                        }
                        // Set href for chat link
                        if (template instanceof HTMLAnchorElement) {
                            template.href = `/app/gym/chat?prompt=${encodeURIComponent(d.data.name)}`;
                        }
                        return template.outerHTML;
                    }
                }
                return '';
            });

        // Update positions on each tick
        function ticked() {
            link.attr("d", d => {
                const sourceX = (d.source as any).x;
                const sourceY = (d.source as any).y;
                const targetX = (d.target as any).x;
                const targetY = (d.target as any).y;
                
                return `M${sourceX},${sourceY}
                        C${(sourceX + targetX) / 2},${sourceY}
                        ${(sourceX + targetX) / 2},${targetY}
                        ${targetX},${targetY}`;
            });

            node.attr("transform", d => {
                const x = (d as any).x - (d.children ? 80 : 70);
                const y = (d as any).y - (d.children ? 80 : 70);
                return `translate(${x},${y})`;
            });
        }
    });
</script>

<!-- Hidden templates for D3 to use -->
<div class="hidden">
    <!-- App node template -->
    <div id="app-node-template" class="relative w-[160px] h-[160px] bg-white border-4 border-black rounded-2xl p-4">
        <img class="app-icon absolute bottom-2 left-2 w-8 h-8" src="" alt="app icon">
        <div>
            <div class="font-medium text-xl text-neutral-800"></div>
            <div class="text-xs text-neutral-600 mt-1"></div>
        </div>
    </div>

    <!-- Task node template -->
    <a id="task-node-template" href="#" class="relative w-[140px] h-[140px] bg-white border-4 border-[#f7edfd] hover:border-[#bc59fa] rounded-2xl p-4 hover:bg-gray-50 transition-colors no-underline block">
        <div class="text-md text-neutral-800 font-medium break-words"></div>
        <img class="app-icon absolute bottom-2 left-2 w-6 h-6" src="" alt="app icon">
    </a>
</div>

<div class="w-full h-full bg-white text-neutral-800 p-4 overflow-hidden">
    <GymHeader title="Skill Tree" />
    <div class="relative w-full h-[calc(100vh-8rem)] border border-neutral-200 rounded-lg">
        <svg bind:this={svg} class="w-full h-full">
            <defs>
            </defs>
        </svg>
    </div>
</div>

<style>
    :global(.skill-tree-link) {
        fill: none;
        stroke: #bc59fa;
        stroke-width: 6px;
        stroke-opacity: 0.2;
        stroke-linecap: round;
        stroke-linejoin: round;
        transition: stroke-opacity 0.3s;
    }

    :global(.skill-tree-link:hover) {
        stroke-opacity: 1;
    }

    :global(.skill-tree-node) {
        transition: transform 0.1s;
    }
</style>
