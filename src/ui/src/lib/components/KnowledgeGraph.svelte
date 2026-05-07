<script lang="ts">
  import { onMount } from "svelte";
  import * as d3 from "d3";
  import { db } from "$lib/state/db";
  import { pdfState } from "$lib/state/pdfState.svelte";

  let svg: SVGSVGElement | undefined = $state();
  let width = $state(400);
  let height = $state(400);

  async function initGraph() {
    if (!svg) return;

    const docs = await db.documents.toArray();
    const entities = await db.entities.toArray();

    const nodes: any[] = [];
    const links: any[] = [];

    // Add Document Nodes
    docs.forEach(doc => {
      nodes.push({ id: doc.path, name: doc.name, type: 'doc' });
    });

    // Add Entity Nodes and Links
    entities.forEach(ent => {
      nodes.push({ id: ent.name, name: ent.name, type: ent.type });
      ent.docPaths.forEach(path => {
        links.push({ source: ent.name, target: path });
      });
    });

    const simulation = d3.forceSimulation(nodes)
      .force("link", d3.forceLink(links).id((d: any) => d.id).distance(50))
      .force("charge", d3.forceManyBody().strength(-100))
      .force("center", d3.forceCenter(width / 2, height / 2));

    const svgD3 = d3.select(svg);
    svgD3.selectAll("*").remove();

    const link = svgD3.append("g")
      .attr("stroke", "#94a3b8")
      .attr("stroke-opacity", 0.3)
      .selectAll("line")
      .data(links)
      .join("line");

    const node = svgD3.append("g")
      .selectAll("g")
      .data(nodes)
      .join("g")
      .call(d3.drag<any, any>()
        .on("start", dragstarted)
        .on("drag", dragged)
        .on("end", dragended) as any
      )
      .on("click", (e, d: any) => {
        if (d.type === 'doc') pdfState.openTab(d.id);
      });

    node.append("circle")
      .attr("r", (d: any) => d.type === 'doc' ? 6 : 4)
      .attr("fill", (d: any) => {
        if (d.type === 'doc') return "#3b82f6";
        if (d.type === 'org') return "#ef4444";
        if (d.type === 'date') return "#10b981";
        return "#f59e0b";
      })
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5);

    node.append("text")
      .text((d: any) => d.name)
      .attr("x", 8)
      .attr("y", 4)
      .style("font-size", "8px")
      .style("font-weight", "bold")
      .style("fill", "#64748b");

    simulation.on("tick", () => {
      link
        .attr("x1", (d: any) => d.source.x)
        .attr("y1", (d: any) => d.source.y)
        .attr("x2", (d: any) => d.target.x)
        .attr("y2", (d: any) => d.target.y);

      node
        .attr("transform", (d: any) => `translate(${d.x}, ${d.y})`);
    });

    function dragstarted(event: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: any) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: any) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }
  }

  onMount(() => {
    initGraph();
  });
</script>

<div class="w-full h-full bg-slate-50/50 dark:bg-slate-900/20 rounded-2xl border border-slate-100 dark:border-slate-800 relative overflow-hidden" bind:clientWidth={width} bind:clientHeight={height}>
  <svg bind:this={svg} width={width} height={height} class="cursor-move"></svg>
  
  <div class="absolute bottom-3 left-3 flex flex-wrap gap-2 pointer-events-none">
     <div class="flex items-center gap-1.5 px-2 py-1 bg-white/80 dark:bg-slate-800/80 rounded-lg border border-slate-100 dark:border-slate-700 shadow-sm backdrop-blur-sm">
        <div class="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
        <span class="text-[7px] font-black uppercase text-slate-500">Document</span>
     </div>
     <div class="flex items-center gap-1.5 px-2 py-1 bg-white/80 dark:bg-slate-800/80 rounded-lg border border-slate-100 dark:border-slate-700 shadow-sm backdrop-blur-sm">
        <div class="w-1.5 h-1.5 rounded-full bg-red-500"></div>
        <span class="text-[7px] font-black uppercase text-slate-500">Organization</span>
     </div>
     <div class="flex items-center gap-1.5 px-2 py-1 bg-white/80 dark:bg-slate-800/80 rounded-lg border border-slate-100 dark:border-slate-700 shadow-sm backdrop-blur-sm">
        <div class="w-1.5 h-1.5 rounded-full bg-green-500"></div>
        <span class="text-[7px] font-black uppercase text-slate-500">Date</span>
     </div>
  </div>
</div>
