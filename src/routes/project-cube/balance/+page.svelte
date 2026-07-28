<script>
  import { base } from '$app/paths';
  import Math from '$lib/Math.svelte';
  import BlogHead from '$lib/BlogHead.svelte';

  // Heavier expressions defined here so the markup stays readable. `String.raw`
  // keeps backslashes literal (so `\text` stays `\text`, not a tab character).
  const tex = String.raw;
  const COM_DEF = tex`\text{COM} = \tfrac{1}{3}\bigl[(1.5, 1.5, 1.5) + (2.5, 1.5, 1.5) + (1.5, 2.5, 1.5)\bigr] = (11/6,\ 11/6,\ 3/2).`;
  const LP_SYSTEM = tex`\begin{aligned}
\text{force balance:}\quad &\sum_{c \in \mathcal{C}(b)} \sigma_{b,c} \sum_{k=1}^{4} F_{c,k} = m_b\, g \\
\text{torque about }y\text{:}\quad &\sum_{c \in \mathcal{C}(b)} \sigma_{b,c} \sum_{k=1}^{4} F_{c,k}\, x_{c,k} = m_b\, g\, x_b^{\text{COM}} \\
\text{torque about }x\text{:}\quad &\sum_{c \in \mathcal{C}(b)} \sigma_{b,c} \sum_{k=1}^{4} F_{c,k}\, y_{c,k} = m_b\, g\, y_b^{\text{COM}}
\end{aligned}`;
  const TORQUE_QUARTER = tex`\tfrac{mg}{4}`;
  const FORCE_BALANCE = tex`4 \cdot \tfrac{mg}{4} = mg`;
</script>

<BlogHead
  title="Balance — Project Cube"
  description="When does a 3×3×3 wooden structure balance? Deciding stability with a linear program over per-block force and torque constraints in Project Cube."
  path="/blog/project-cube/balance/"
/>

<h2>Balance: harder than it looks</h2>

<div class="note">
<strong>TL;DR.</strong> Programming "does this 3-D structure stand?" turned out to be the longest detour of the project. Each simple rule I tried failed on a slightly weirder configuration. In the end, I stuck to a <a href="https://en.wikipedia.org/wiki/Linear_programming" target="_blank" rel="noopener noreferrer">linear program</a> to solve it to some extend at least.
</div>

<p>The main thing I wanted was to establish a rule of balance. Something that can be checked by the computer. Preferably simple, under which every case of balance will fall. This page contains some examples and how the algorithm changed throughout the project.</p>

<aside class="note">
<strong>A note on coordinates.</strong> Cells are indexed by integer <Math expr="(x, y, z)" /> with each axis running 0 to 2. The integer is the <em>edge</em> of the cell, so cell <Math expr="(1, 1, 0)" /> spans <Math expr={tex`x, y \in [1, 2]`} />. The full 3×3×3 grid runs from 0 to 3 on each axis.
</aside>

<h3>An L on a hollow cube</h3>

<figure>
  <iframe class="iframe-tower" src={`${base}/project-cube/tower_view.html?example=l_on_hollow_stable`} loading="lazy" title="L on hollow"></iframe>
  <figcaption><a href="https://en.wikipedia.org/wiki/Center_of_mass" target="_blank" rel="noopener noreferrer">Center of mass</a> (COM) at <Math expr={tex`(11/6,\ 11/6,\ 3/2)`} />, inside the 1×1 support polygon (which spans <Math expr={tex`x, y \in [1, 2]`} />).</figcaption>
</figure>

<p>The L spans three unit cells with centers at <Math expr="(1.5, 1.5, 1.5)" />, <Math expr="(2.5, 1.5, 1.5)" />, and <Math expr="(1.5, 2.5, 1.5)" />. With equal mass per cell, the COM is the mean of the centers:</p>

<Math expr={COM_DEF} display />

<p class="rule-callout"><strong>The rule:</strong> each block's center of mass (COM) has to be over the cells supporting it.</p>

<p><em>"Right, it seems we are done, a COM can be easily calculated for each block, and the cells supporting it are the ones directly beneath, right? Right?!?"</em></p>

<h3>Dropping a 1×1×3 on top</h3>

<figure class="mx-auto mb-[0.6em] mt-[0.2em] w-full sm:float-right sm:ml-[1.2em] sm:w-[260px]">
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

<figure class="mx-auto mb-[0.6em] mt-[0.2em] w-full sm:float-right sm:ml-[1.2em] sm:w-[260px]">
  <iframe src={`${base}/project-cube/tower_view.html?example=l_hugging`} loading="lazy" title="L hugging vertical 1×1×2 (tips)" class="h-[260px]"></iframe>
</figure>

<p>The L's COM is at <Math expr={tex`x_\text{COM} = 1/2`} />, <Math expr={tex`y_\text{COM} = 7/6`} />. Since the COM is outside its support, will this block fall? Well, imagine that the blocks were made of rubber. The center of mass of the whole structure says it will stand. And yet the LP actually has it slide off. There is friction, especially when the pieces are made of rubber. So then it would be pretty logical that this tower would stand. Do we somehow need to account for friction? In the end, I chose not to do this because it would just create a lot more difficulty.</p>

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
  <g transform="translate(20, 20)">
    <text x="125" y="0" font-size="13" font-weight="bold" fill="#2b7d2b" text-anchor="middle">balanced ✓</text>
    <rect x="50" y="25" width="150" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="100" y1="25" x2="100" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <line x1="150" y1="25" x2="150" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <rect x="100" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <circle cx="100" cy="75" r="4" fill="#d35400"/>
    <circle cx="150" cy="75" r="4" fill="#d35400"/>
    <circle cx="125" cy="50" r="5" fill="#2266cc"/>
    <text x="133" y="54" font-size="11" fill="#2266cc">COM</text>
    <line x1="125" y1="50" x2="125" y2="125" stroke="#222" stroke-width="2.5" marker-end="url(#gA)"/>
    <text x="133" y="95" font-size="12" fill="#222">m·g</text>
    <line x1="100" y1="75" x2="100" y2="38" stroke="#2b7d2b" stroke-width="2.5" marker-end="url(#rA)"/>
    <line x1="150" y1="75" x2="150" y2="38" stroke="#2b7d2b" stroke-width="2.5" marker-end="url(#rA)"/>
    <text x="92" y="60" font-size="12" fill="#2b7d2b" text-anchor="end">m·g/2</text>
    <text x="158" y="60" font-size="12" fill="#2b7d2b">m·g/2</text>
  </g>
  <g transform="translate(290, 20)">
    <text x="125" y="0" font-size="13" font-weight="bold" fill="#cc4444" text-anchor="middle">tips ✗</text>
    <rect x="50" y="25" width="150" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="100" y1="25" x2="100" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <line x1="150" y1="25" x2="150" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <rect x="50" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <circle cx="50" cy="75" r="4" fill="#d35400"/>
    <circle cx="100" cy="75" r="4" fill="#d35400"/>
    <circle cx="125" cy="50" r="5" fill="#2266cc"/>
    <text x="133" y="54" font-size="11" fill="#2266cc">COM</text>
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
  <rect x="245" y="100" width="70" height="70" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
  <rect x="385" y="100" width="70" height="70" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
  <rect x="245" y="30" width="210" height="70" fill="#fff7e6" stroke="#222" stroke-width="2"/>
  <line x1="315" y1="30" x2="315" y2="100" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
  <line x1="385" y1="30" x2="385" y2="100" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
  <circle cx="245" cy="100" r="5" fill="#d35400"/>
  <circle cx="315" cy="100" r="5" fill="#d35400"/>
  <circle cx="385" cy="100" r="5" fill="#d35400"/>
  <circle cx="455" cy="100" r="5" fill="#d35400"/>
  <circle cx="350" cy="65" r="6" fill="#2266cc"/>
  <text x="360" y="70" font-size="14" fill="#2266cc">COM</text>
  <line x1="350" y1="71" x2="350" y2="170" stroke="#222" stroke-width="3" marker-end="url(#gC)"/>
  <text x="360" y="130" font-size="15" fill="#222">m·g</text>
  <line x1="245" y1="100" x2="245" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="315" y1="100" x2="315" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="385" y1="100" x2="385" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <line x1="455" y1="100" x2="455" y2="75" stroke="#2b7d2b" stroke-width="3" marker-end="url(#rC)"/>
  <text x="240" y="140" font-size="14" fill="#2b7d2b" text-anchor="end">m·g/4 each</text>
  <text x="460" y="140" font-size="14" fill="#2b7d2b">m·g/4 each</text>
</svg>
<figcaption>Bridge, side view. Two separate 1×1 contacts, COM over the gap. Each of the four corners can push independently; one balanced solution puts <Math expr={TORQUE_QUARTER} /> at every corner. Force balance: <Math expr={FORCE_BALANCE} /> ✓. Torque balance: the four equal pushes are symmetric around the COM, so their torques cancel ✓.</figcaption>
</figure>

<h4 class="mb-[0.4em]">The algorithm, written out</h4>

<p>Each face of every block is split into 1×1 parts. This ensures that a block like the long bar can be used in the same way and have the same counteracting forces. Since we check for torque, this ensures that the long bar is actually connected because it has a single center of mass. Also, to slightly simplify the algorithm and lower the running time, I only added vertical forces here. Yes, you could also do side forces, but side forces need to be counteracted, and there is no wall and no friction to do this. If one were to extend this algorithm with friction, side forces should also be implemented.</p>

<p>Notation, before the LP equations:</p>

<ul>
<li><Math expr={tex`\mathcal{C}`} /> — the set of all contact faces in the tower (block-on-block, and block-on-ground). <Math expr={tex`\mathcal{C}(b)`} /> — the subset that touches block <Math expr="b" />.</li>
<li><Math expr={tex`F_{c,k} \geq 0`} /> — the upward push at corner <Math expr={tex`k \in \{1,2,3,4\}`} /> of contact <Math expr="c" />. Four non-negative variables per face.</li>
<li><Math expr={tex`(x_{c,k}, y_{c,k})`} /> — the horizontal position of that corner.</li>
<li><Math expr={tex`\sigma_{b,c}`} /> — sign convention: <Math expr="+1" /> if block <Math expr="b" /> sits above contact <Math expr="c" /> (contact pushes <Math expr="b" /> up), <Math expr="-1" /> if <Math expr="b" /> sits below (reaction presses <Math expr="b" /> down).</li>
<li><Math expr={tex`m_b, g, (x_b^{\text{COM}}, y_b^{\text{COM}})`} /> — block <Math expr="b" />'s mass, gravity, and the horizontal coordinates of its centre of mass.</li>
</ul>

<Math expr={LP_SYSTEM} display />

<p>No objective: it's a pure <em>feasibility</em> LP. Some <Math expr={tex`F \geq 0`} /> satisfies every equation ⇒ <span class="text-[#2b7d2b]"><strong>stable</strong></span>; no such <Math expr="F" /> exists ⇒ <span class="text-[#cc4444]"><strong>tips</strong></span>.</p>

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
  <g transform="translate(20, 20)">
    <text x="120" y="0" font-size="13" font-weight="bold" fill="#dba434" text-anchor="middle">δ = 0 → MARGINAL</text>
    <rect x="70" y="25" width="100" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="120" y1="25" x2="120" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <rect x="70" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <circle cx="70" cy="75" r="4" fill="#d35400"/>
    <circle cx="120" cy="75" r="4" fill="#d35400"/>
    <circle cx="120" cy="50" r="5" fill="#2266cc"/>
    <text x="128" y="54" font-size="11" fill="#2266cc">COM</text>
    <line x1="115" y1="50" x2="115" y2="75" stroke="#222" stroke-width="2.5" marker-end="url(#gD)"/>
    <text x="105" y="65" font-size="12" fill="#222" text-anchor="end">m·g</text>
    <line x1="120" y1="75" x2="120" y2="50" stroke="#dba434" stroke-width="2.5" marker-end="url(#rDok)"/>
    <text x="128" y="65" font-size="11" fill="#dba434">F on edge</text>
  </g>
  <g transform="translate(290, 20)">
    <text x="120" y="0" font-size="13" font-weight="bold" fill="#cc4444" text-anchor="middle">δ &gt; 0 → UNSTABLE</text>
    <rect x="70" y="25" width="100" height="50" fill="#fff7e6" stroke="#222" stroke-width="2"/>
    <line x1="120" y1="25" x2="120" y2="75" stroke="#222" stroke-width="0.5" stroke-dasharray="3,2" opacity="0.4"/>
    <rect x="70" y="75" width="50" height="50" fill="#dde3ec" stroke="#222" stroke-width="1.5"/>
    <line x1="70" y1="75" x2="120" y2="75" stroke="#888" stroke-width="1.5" stroke-dasharray="3,3"/>
    <line x1="74" y1="75" x2="116" y2="75" stroke="#222" stroke-width="3"/>
    <circle cx="74" cy="75" r="4" fill="#d35400"/>
    <circle cx="116" cy="75" r="4" fill="#d35400"/>
    <circle cx="120" cy="50" r="5" fill="#2266cc"/>
    <text x="128" y="54" font-size="11" fill="#2266cc">COM</text>
    <line x1="115" y1="50" x2="115" y2="75" stroke="#222" stroke-width="2.5" marker-end="url(#gD)"/>
    <text x="105" y="65" font-size="12" fill="#222" text-anchor="end">m·g</text>
    <line x1="120" y1="75" x2="120" y2="50" stroke="#cc4444" stroke-width="2.5" stroke-dasharray="4,3" marker-end="url(#rDfail)"/>
    <text x="128" y="65" font-size="11" fill="#cc4444">F outside ✗</text>
  </g>
</svg>
<figcaption><em>Left:</em> δ = 0. This is the case as before. If the force at the right corner is exactly the gravity from the center of mass, this will balance. <em>Right:</em> if we trimmed the face a little, then the center of mass now lies outside the contact face. So there are no possible forces that would counteract the weight.</figcaption>
</figure>
