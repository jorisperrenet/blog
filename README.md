# Blog

(Yes, AI was used to generate this README, but the blog is human-written)

Source for [jorisperrenet.github.io/blog](https://jorisperrenet.github.io/blog).
A single SvelteKit + Tailwind v4 project that prerenders to a static site.
The blog landing lists posts; each post lives at `src/routes/<slug>/`. Posts
that need their own backing analysis (Rust, generated data) keep that code
in a sibling directory like `project-cube/`.

## Layout

```
.
├── package.json, svelte.config.js, src/, static/   # the SvelteKit blog
│   └── src/routes/
│       ├── +page.svelte                            # blog landing
│       └── project-cube/                           # Project Cube post
│           ├── +page.svelte, +layout.svelte, post.css
│           ├── balance/, cards/, pairs/, decks/, rules/, conclusion/, solver/
│           └── …
├── static/project-cube/                            # iframe assets (built by Rust)
│   ├── tower_view.html
│   └── assets/                                     # populated by `generate_blog_examples`
├── .github/workflows/deploy.yml                    # builds + deploys to GH Pages
└── project-cube/                                   # the Rust analysis for the post
    ├── Cargo.toml, Cargo.lock, src/
    ├── cards.txt, cards_original.txt
    └── view_counts.txt, view_counts_all.txt        # generated
```

## Local development

```sh
npm install
npm run dev          # blog landing only; project-cube needs the Rust step below
```

To build the Project Cube post's interactive assets (3-D towers, compat
graphs, manifests) into `static/project-cube/assets/`:

```sh
cd project-cube
cargo build --release

# Generate view-count tables (~1 minute each).
PROJECT_CUBE_ALL_PIECES=0 PROJECT_CUBE_OUTPUT=view_counts.txt     ./target/release/project_cube
PROJECT_CUBE_ALL_PIECES=1 PROJECT_CUBE_OUTPUT=view_counts_all.txt ./target/release/project_cube

# Build the solver database + every other generated asset.
./target/release/build_solver_db
./target/release/generate_blog_examples
```

Then back at the blog root, `npm run dev` (or `npm run build && npm run preview`)
will serve the post at `/project-cube/`.

Total wall time from a clean tree: roughly three minutes.

Prerequisites: Rust (stable), Node 20+ and npm.

## Deploy

`.github/workflows/deploy.yml` runs the Rust pipeline + the SvelteKit
build on every push to `main` and deploys the result to GitHub Pages. The
view-count `.txt` files (slow to generate) are cached so they only re-run
when the Rust source changes.

In the repo's *Settings → Pages*, set "Build and deployment" source to
**GitHub Actions**.

## Adding a new post

1. Add a route directory under `src/routes/<slug>/` with `+page.svelte`
   (and `+layout.svelte` if the post needs its own nav, etc.).
2. If the post needs static assets, drop them in `static/<slug>/`.
3. If the post has a backing analysis, mirror what `project-cube/` does:
   sibling directory, generated assets land in `static/<slug>/assets/`.
4. Add an entry to the `posts` array in `src/routes/+page.svelte`.

## Project Cube — note

> **Project Cube** is © Alain Rivollet, all rights reserved. The
> `project-cube/` directory is an analysis only. If anything in it
> interests you, please buy the actual game; it's a different (and
> better) experience than reading about it.

The post itself walks through the analysis end-to-end. See
`src/routes/project-cube/` for the published prose and `project-cube/src/`
for the Rust source.

## License

MIT for the code in this repository; see [`LICENSE`](LICENSE). **Project
Cube itself is © Alain Rivollet** — the analysis isn't an authorisation
to redistribute the game's content.
