<script>
  import { base } from '$app/paths';
</script>

<svelte:head><title>Balance — Project Cube</title></svelte:head>

<!-- Section 3 — Balance: harder than it looks                       -->
<!-- ============================================================== -->

<h2>Balance: harder than it looks</h2>

<div class="note">
<strong>TL;DR.</strong> Programming "does this 3-D structure stand?" turned out to be the longest detour of the project. Each simple rule I tried failed on a slightly weirder configuration. In the end, I stuck to a <a href="https://en.wikipedia.org/wiki/Linear_programming" target="_blank" rel="noopener noreferrer">linear program</a> to solve it to some extend at least.
</div>

<p>The main thing I wanted was to establish a rule of balance. Something that can be checked by the computer. Preferably simple, under which every case of balance will fall. This page contains some examples and how the algorithm changed throughout the project.</p>

<aside class="note">
<strong>A note on coordinates.</strong> Cells are indexed by integer \((x, y, z)\) with each axis running 0 to 2. The integer is the <em>edge</em> of the cell, so cell \((1, 1, 0)\) spans \(x, y \in [1, 2]\). The full 3×3×3 grid runs from 0 to 3 on each axis.
</aside>

<h3>An L on a hollow cube</h3>

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=l_on_hollow_stable`} loading="lazy" title="L on hollow"></iframe>
  <figcaption><a href="https://en.wikipedia.org/wiki/Center_of_mass" target="_blank" rel="noopener noreferrer">Center of mass</a> (COM) at \((11/6,\ 11/6,\ 3/2)\), inside the 1×1 support polygon (which spans \(x, y \in [1, 2]\)).</figcaption>
</figure>

<p>The L spans three unit cells with centers at \((1.5, 1.5, 1.5)\), \((2.5, 1.5, 1.5)\), and \((1.5, 2.5, 1.5)\). With equal mass per cell, the COM is the mean of the centers:</p>

<p>$$\text&lbrace;COM&rbrace; = \tfrac&lbrace;1&rbrace;&lbrace;3&rbrace;\bigl[(1.5, 1.5, 1.5) + (2.5, 1.5, 1.5) + (1.5, 2.5, 1.5)\bigr] = (11/6,\ 11/6,\ 3/2).$$</p>

<p class="rule-callout"><strong>The rule:</strong> each block's center of mass (COM) has to be over the cells supporting it.</p>

<p><em>"Right, it seems we are done, a COM can be easily calculated for each block, and the cells supporting it are the ones directly beneath, right? Right?!?"</em></p>

<h3>Dropping a 1×1×3 on top</h3>

<figure class="float-right w-[260px] mt-[0.2em] mb-[0.6em] ml-[1.2em]">
  <iframe src={`${base}/project-cube/tower_view.html?example=l_on_hollow_plus_triple`} loading="lazy" title="L + 1×1×3 — tips" class="h-[260px]"></iframe>
</figure>

<p>Now we add a long bar to the structure. Our rule says that this will balance, since the long bar's center of mass is above the cell directly beneath it. And for the L piece it was just as before, so that center of mass will also be above the cell supporting it. Yet, this tower will not stand at all. Therefore, our rule was wrong.</p>

<p>Changing the rule, we must account for not only the blocks, but each block resting on top of that as well. The L is now carrying the long bar on top, and the combined center of mass of those two falls past the blocks supporting it.</p>

<div class="clear-both"></div>

<p class="rule-callout"><strong>The rule:</strong> for each block, the COM of <em>that block plus every block resting on it</em> has to be over the cells supporting it.</p>

<p><em>"This is it, COM can be easily calculated still, every block on top will just be directly on top (works transitively) and the cells supporting it haven't changed. This rule works, right?!?"</em></p>

<h3>Bridges</h3>

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=bridge`} loading="lazy" title="bridge"></iframe>
  <figcaption>The long bar's middle cell hangs over nothing.</figcaption>
</figure>

<p>Apply the updated rule: Nothing rests on top of the long bar. So we calculate the COM of the long bar. It is right above the empty middle cell. So it doesn't have any supporting blocks directly beneath it, yet the tower clearly stands.</p>

<p>Now the rule must undergo another change. The issue this time is with the supporting blocks. And we will change this to a rule that the COM should lie above the <a href="https://en.wikipedia.org/wiki/Convex_hull" target="_blank" rel="noopener noreferrer">convex hull</a> of the cells directly beneath it. However, if we do this, we would still have a problem. One that might not be that obvious. What if this structure is on top of something? Again, the long bar will distribute its weight equally over its supports. Currently, the algorithm wouldn't account for that. And then it would have "the block plus every block resting on it" wrong.</p>

<p>Thus, our rule is changing again. Before we formulate how the rule would need to change, let's think of other cases that might be special or broken before we continue. Otherwise, we might have to start all over again.</p>

<p class="rule-callout"><strong>The rule:</strong> for each block, the COM of <em>that block plus every block resting on it</em> has to be over the convex hull of the cells supporting it. Plus somehow account for bridges and weight splitting.</p>

<p><em>"So we already have a rule that's not properly defined, if this is it, we can probably make it working. This IS it, right?!?"</em></p>

<h3>An L wrapping a column</h3>

<figure class="float-right w-[260px] mt-[0.2em] mb-[0.6em] ml-[1.2em]">
  <iframe src={`${base}/project-cube/tower_view.html?example=l_hugging`} loading="lazy" title="L hugging vertical 1×1×2 (tips)" class="h-[260px]"></iframe>
</figure>

<p>The L's COM is at \(x_\text&lbrace;COM&rbrace; = 1/2\), \(y_\text&lbrace;COM&rbrace; = 7/6\). Since the COM is outside its support, will this block fall? Well, imagine that the blocks were made of rubber. The center of mass of the whole structure says it will stand. And yet the LP actually has it slide off. There is friction, especially when the pieces are made of rubber. So then it would be pretty logical that this tower would stand. Do we somehow need to account for friction? In the end, I chose not to do this because it would just create a lot more difficulty.</p>

<p><em>"The rule remains unchanged, but now also has to account for torque, and we also said that we won't implement friction. Does this have to become a full-blown physics simulator?"</em></p>

<h3>Does weight matter?</h3>

<p>What about weight? Can we find positions with different stabilities depending on the weight of the blocks? The actual blocks weigh roughly <strong>15 g (full)</strong> and <strong>5 g (hollow)</strong>. A 3:1 ratio. See below. There are two situations with the same blocks but with different stability depending on the weight ratio.</p>

<div class="split-figs">
  <figure>
    <iframe src={`${base}/project-cube/tower_view.html?example=weight_actual`} loading="lazy" title="weight matters — game ratio"></iframe>
    <figcaption><strong>3:1 (the game).</strong> The hollow piece counterweights the L. Stable.</figcaption>
  </figure>
  <figure>
    <iframe src={`${base}/project-cube/tower_view.html?example=weight_extreme`} loading="lazy" title="weight matters — hollow much lighter"></iframe>
    <figcaption><strong>15:1 (hollow nearly weightless).</strong> Same blocks, same positions. Counterweight gone. Tips.</figcaption>
  </figure>
</div>

<p><em>"Thus a COM of a block plus every block resting on it has to be over the convex hull supporting it + account for bridges and weight dividing + torque + the actual weights of pieces matter, that would be a lot of rules, we need something stronger to fix this!"</em></p>

<h3 class="text-[1.6em] mt-12">The actual stability check</h3>

<p>This is the algorithm that I went for in the end. For every block, two things must balance — force and torque. If there exist forces such that all are counteracted (Newton's third law) and there is no net torque in the whole structure, then there is a stable solution. This can be solved using a <a href="https://en.wikipedia.org/wiki/Linear_programming" target="_blank" rel="noopener noreferrer">linear program</a>.</p>

<p>Torque can be calculated from the forces acting on an object and its center of mass. Thus, we need to solve for all the forces. Every face of every block is split up into four corners, each with a possible force perpendicular to the surface.</p>

<h4 class="mb-[0.4em]">Instance 1: one block, one contact</h4>

<figure class="my-4">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 180" width="100%" class="max-w-[540px] block mx-auto font-sans">
  <defs>
    <marker id="gA" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#222"/>
    </marker>
    <marker id="rA" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#2b7d2b"/>
    </marker>
  </defs>
  <!-- LEFT: balanced — 1×3 block centred on a 1×1 cube -->
  <g transform="translate(20, 20)">
    <text x="125" y="0" font-size="13" font-weight="bold" fill="#2b7d2b" text-anchor="middle">balanced ✓</text>
    <!-- top block (1×3, 3 cells of 50px each) -->
    <rect x="50" y="25" width="150" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="100" y1="25" x2="100" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <line x1="150" y1="25" x2="150" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <!-- supporting cube under the MIDDLE cell — touches block's bottom -->
    <rect x="100" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <!-- contact corners (where the two blocks meet) -->
    <circle cx="100" cy="75" r="4" fill="#d35400"/>
    <circle cx="150" cy="75" r="4" fill="#d35400"/>
    <!-- COM at the centre of the top block -->
    <circle cx="125" cy="50" r="5" fill="#2266cc"/>
    <text x="133" y="54" font-size="11" fill="#2266cc">COM</text>
    <!-- gravity arrow: long, from COM straight down through the cube -->
    <line x1="125" y1="50" x2="125" y2="125" stroke="#222" stroke-width="2.5" marker-end="url(#gA)"/>
    <text x="133" y="95" font-size="12" fill="#222">m·g</text>
    <!-- two corner reactions, each half the gravity length, pushing up into the block -->
    <line x1="100" y1="75" x2="100" y2="38" stroke="#2b7d2b" stroke-width="2.5" marker-end="url(#rA)"/>
    <line x1="150" y1="75" x2="150" y2="38" stroke="#2b7d2b" stroke-width="2.5" marker-end="url(#rA)"/>
    <text x="92" y="60" font-size="12" fill="#2b7d2b" text-anchor="end">m·g/2</text>
    <text x="158" y="60" font-size="12" fill="#2b7d2b">m·g/2</text>
  </g>
  <!-- RIGHT: tips — same block on a cube under the LEFT cell -->
  <g transform="translate(290, 20)">
    <text x="125" y="0" font-size="13" font-weight="bold" fill="#cc4444" text-anchor="middle">tips ✗</text>
    <!-- same top block -->
    <rect x="50" y="25" width="150" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="100" y1="25" x2="100" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <line x1="150" y1="25" x2="150" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <!-- supporting cube under the LEFT cell -->
    <rect x="50" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <circle cx="50" cy="75" r="4" fill="#d35400"/>
    <circle cx="100" cy="75" r="4" fill="#d35400"/>
    <!-- COM at block centre — past cube's right edge -->
    <circle cx="125" cy="50" r="5" fill="#2266cc"/>
    <text x="133" y="54" font-size="11" fill="#2266cc">COM</text>
    <!-- gravity from COM down through block bottom into empty space below (no cube under this column) -->
    <line x1="125" y1="53" x2="125" y2="125" stroke="#222" stroke-width="2.5" marker-end="url(#gA)"/>
    <text x="133" y="100" font-size="12" fill="#222">m·g</text>
  </g>
</svg>
<figcaption><em>Left:</em> the force of gravity of the long bar can be counteracted with forces at the four corners of the smaller block. <em>Right:</em> the long bar overhangs the face of the small block. Any force on the small block would induce a torque on the long bar. Therefore this is unstable.</figcaption>
</figure>

<h4 class="mb-[0.4em]">Instance 2: two contacts</h4>

<p>Same algorithm. The LP automatically splits the load wherever a non-negative split satisfies all the equations — no special "bridge rule" needed.</p>

<figure class="my-4">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 700 280" width="100%" class="max-w-[700px] block mx-auto font-sans">
  <defs>
    <marker id="gC" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#222"/>
    </marker>
    <marker id="rC" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#2b7d2b"/>
    </marker>
  </defs>
  <!-- left 1×1 cube (70×70) -->
  <rect x="245" y="100" width="70" height="70" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
  <!-- right 1×1 cube -->
  <rect x="385" y="100" width="70" height="70" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
  <!-- 1×1×3 beam (210×70) sitting on both cubes -->
  <rect x="245" y="30" width="210" height="70" fill="#fff7e6" stroke="#222" stroke-width="2"/>
  <line x1="315" y1="30" x2="315" y2="100" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
  <line x1="385" y1="30" x2="385" y2="100" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
  <!-- contact corners (each contact face has 2 visible corners in side view) -->
  <circle cx="245" cy="100" r="5" fill="#d35400"/>
  <circle cx="315" cy="100" r="5" fill="#d35400"/>
  <circle cx="385" cy="100" r="5" fill="#d35400"/>
  <circle cx="455" cy="100" r="5" fill="#d35400"/>
  <!-- COM at beam centre (over the unsupported middle cell) -->
  <circle cx="350" cy="65" r="6" fill="#2266cc"/>
  <text x="360" y="70" font-size="14" fill="#2266cc">COM</text>
  <!-- gravity arrow (= mg): long, from COM down through beam bottom into the gap -->
  <line x1="350" y1="71" x2="350" y2="170" stroke="#222" stroke-width="3" marker-end="url(#gC)"/>
  <text x="360" y="130" font-size="15" fill="#222">m·g</text>
  <!-- four corner reactions, each m·g/4, pushing up into the beam -->
  <line x1="245" y1="100" x2="245" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="315" y1="100" x2="315" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="385" y1="100" x2="385" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="455" y1="100" x2="455" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <!-- labels on the outer sides of the cubes -->
  <text x="240" y="140" font-size="14" fill="#2b7d2b" text-anchor="end">m·g/4 each</text>
  <text x="460" y="140" font-size="14" fill="#2b7d2b">m·g/4 each</text>
</svg>
<figcaption>Bridge, side view. Two separate 1×1 contacts, COM over the gap. Each of the four corners can push independently; one balanced solution puts \(\tfrac&lbrace;mg&rbrace;&lbrace;4&rbrace;\) at every corner. Force balance: \(4 \cdot \tfrac&lbrace;mg&rbrace;&lbrace;4&rbrace; = mg\) ✓. Torque balance: the four equal pushes are symmetric around the COM, so their torques cancel ✓.</figcaption>
</figure>

<h4 class="mb-[0.4em]">The algorithm, written out</h4>

<p>Each face of every block is split into 1×1 parts. This ensures that a block like the long bar can be used in the same way and have the same counteracting forces. Since we check for torque, this ensures that the long bar is actually connected because it has a single center of mass. Also, to slightly simplify the algorithm and lower the running time, I only added vertical forces here. Yes, you could also do side forces, but side forces need to be counteracted, and there is no wall and no friction to do this. If one were to extend this algorithm with friction, side forces should also be implemented.</p>

<p>Notation, before the LP equations:</p>

<ul>
<li>\(\mathcal&lbrace;C&rbrace;\) — the set of all contact faces in the tower (block-on-block, and block-on-ground). \(\mathcal&lbrace;C&rbrace;(b)\) — the subset that touches block \(b\).</li>
<li>\(F_&lbrace;c,k&rbrace; \geq 0\) — the upward push at corner \(k \in \&lbrace;1,2,3,4\&rbrace;\) of contact \(c\). Four non-negative variables per face.</li>
<li>\((x_&lbrace;c,k&rbrace;, y_&lbrace;c,k&rbrace;)\) — the horizontal position of that corner.</li>
<li>\(\sigma_&lbrace;b,c&rbrace;\) — sign convention: \(+1\) if block \(b\) sits above contact \(c\) (contact pushes \(b\) up), \(-1\) if \(b\) sits below (reaction presses \(b\) down).</li>
<li>\(m_b, g, (x_b^&lbrace;\text&lbrace;COM&rbrace;&rbrace;, y_b^&lbrace;\text&lbrace;COM&rbrace;&rbrace;)\) — block \(b\)'s mass, gravity, and the horizontal coordinates of its centre of mass.</li>
</ul>

<p>$$
\begin&lbrace;aligned&rbrace;
\text&lbrace;force balance:&rbrace;\quad &\sum_&lbrace;c \in \mathcal&lbrace;C&rbrace;(b)&rbrace; \sigma_&lbrace;b,c&rbrace; \sum_&lbrace;k=1&rbrace;^&lbrace;4&rbrace; F_&lbrace;c,k&rbrace; = m_b\, g \\
\text&lbrace;torque about &rbrace;y\text&lbrace;:&rbrace;\quad &\sum_&lbrace;c \in \mathcal&lbrace;C&rbrace;(b)&rbrace; \sigma_&lbrace;b,c&rbrace; \sum_&lbrace;k=1&rbrace;^&lbrace;4&rbrace; F_&lbrace;c,k&rbrace;\, x_&lbrace;c,k&rbrace; = m_b\, g\, x_b^&lbrace;\text&lbrace;COM&rbrace;&rbrace; \\
\text&lbrace;torque about &rbrace;x\text&lbrace;:&rbrace;\quad &\sum_&lbrace;c \in \mathcal&lbrace;C&rbrace;(b)&rbrace; \sigma_&lbrace;b,c&rbrace; \sum_&lbrace;k=1&rbrace;^&lbrace;4&rbrace; F_&lbrace;c,k&rbrace;\, y_&lbrace;c,k&rbrace; = m_b\, g\, y_b^&lbrace;\text&lbrace;COM&rbrace;&rbrace;
\end&lbrace;aligned&rbrace;
$$</p>

<p>No objective: it's a pure <em>feasibility</em> LP. Some \(F \geq 0\) satisfies every equation ⇒ <span class="text-[#2b7d2b]"><strong>stable</strong></span>; no such \(F\) exists ⇒ <span class="text-[#cc4444]"><strong>tips</strong></span>.</p>

<h4 class="mb-[0.4em]">Strict vs marginal balance.</h4>

<p>What about a 1×1×2 block balancing on a single block? In a perfect world, this would balance because the center of mass is directly above the edge of the face below. However, in the real world, this will likely not be possible. The pieces are not perfectly manufactured, and somehow I just don't like that solution — barely balancing, or somehow cheating and shifting the block a little more than it has to. Thus I made two categories of balancing: one called strict and one called marginal.</p>

<p>To determine between strict and marginal stability, one can run the linear program with each face's corner positions shrunk inward by a small δ. Towers that survive are <strong>strictly stable</strong>. Ones that don't are <strong>marginally stable</strong>.</p>

<figure class="my-4">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 540 200" width="100%" class="max-w-[540px] block mx-auto font-sans">
  <defs>
    <marker id="gD" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#222"/>
    </marker>
    <marker id="rDok" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#dba434"/>
    </marker>
    <marker id="rDfail" viewBox="0 0 10 10" refX="8" refY="5" markerUnits="strokeWidth" markerWidth="5" markerHeight="6" orient="auto">
      <path d="M0,0 L10,5 L0,10 z" fill="#cc4444"/>
    </marker>
  </defs>
  <!-- LEFT: marginal — 1×2 block sitting on a 1×1 cube; block centre over cube's right edge -->
  <g transform="translate(20, 20)">
    <text x="120" y="0" font-size="13" font-weight="bold" fill="#dba434" text-anchor="middle">δ = 0 → MARGINAL</text>
    <!-- 1×2 block (2 cells of 50px) -->
    <rect x="70" y="25" width="100" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="120" y1="25" x2="120" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <!-- 1×1 cube under the LEFT cell, touching block bottom -->
    <rect x="70" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <!-- contact corners (face = top of cube) -->
    <circle cx="70" cy="75" r="4" fill="#d35400"/>
    <circle cx="120" cy="75" r="4" fill="#d35400"/>
    <!-- COM at block centre = cube's right edge -->
    <circle cx="120" cy="50" r="5" fill="#2266cc"/>
    <text x="128" y="54" font-size="11" fill="#2266cc">COM</text>
    <!-- gravity inside the block -->
    <line x1="115" y1="50" x2="115" y2="75" stroke="#222" stroke-width="2.5" marker-end="url(#gD)"/>
    <text x="105" y="65" font-size="12" fill="#222" text-anchor="end">m·g</text>
    <!-- reaction: tail ON the surface, at face's right edge (= COM column) -->
    <line x1="120" y1="75" x2="120" y2="50" stroke="#dba434" stroke-width="2.5" marker-end="url(#rDok)"/>
    <text x="128" y="65" font-size="11" fill="#dba434">F on edge</text>
  </g>
  <!-- RIGHT: strict — same blocks; LP shrinks the face by δ -->
  <g transform="translate(290, 20)">
    <text x="120" y="0" font-size="13" font-weight="bold" fill="#cc4444" text-anchor="middle">δ &gt; 0 → UNSTABLE</text>
    <rect x="70" y="25" width="100" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="120" y1="25" x2="120" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <rect x="70" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <!-- original face boundary dashed -->
    <line x1="70" y1="75" x2="120" y2="75" stroke="#888" stroke-width="1.5" stroke-dasharray="3,3"/>
    <!-- shrunk face solid -->
    <line x1="74" y1="75" x2="116" y2="75" stroke="#222" stroke-width="3"/>
    <circle cx="74" cy="75" r="4" fill="#d35400"/>
    <circle cx="116" cy="75" r="4" fill="#d35400"/>
    <circle cx="120" cy="50" r="5" fill="#2266cc"/>
    <text x="128" y="54" font-size="11" fill="#2266cc">COM</text>
    <line x1="115" y1="50" x2="115" y2="75" stroke="#222" stroke-width="2.5" marker-end="url(#gD)"/>
    <text x="105" y="65" font-size="12" fill="#222" text-anchor="end">m·g</text>
    <!-- attempted reaction at COM column (x=120) — past the shrunk face's right edge (x=116) -->
    <line x1="120" y1="75" x2="120" y2="50" stroke="#cc4444" stroke-width="2.5" stroke-dasharray="4,3" marker-end="url(#rDfail)"/>
    <text x="128" y="65" font-size="11" fill="#cc4444">F outside ✗</text>
  </g>
</svg>
<figcaption><em>Left:</em> δ = 0. This is the case as before. If the force at the right corner is exactly the gravity from the center of mass, this will balance. <em>Right:</em> if we trimmed the face a little, then the center of mass now lies outside the contact face. So there are no possible forces that would counteract the weight.</figcaption>
</figure>

<!-- ============================================================== -->
