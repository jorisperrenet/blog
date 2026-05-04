// Single source of truth for the eight section pages: nav order, slugs,
// short titles for the header nav, and one-line teasers for the index
// landing cards.
export const sections = [
  { slug: '',           num: 1, title: 'Intro',             teaser: 'The puzzle and the question that started this.' },
  { slug: 'rules',      num: 2, title: 'The rules',         teaser: 'Pieces, cards, and the three rules.' },
  { slug: 'balance',    num: 3, title: 'Balance',           teaser: 'Why "does it stand?" was the longest detour.' },
  { slug: 'cards',      num: 4, title: 'The cards',         teaser: '15 printed cards represent 12 shapes.' },
  { slug: 'pairs',      num: 5, title: 'The deck',          teaser: 'Pairs of cards and which extensions hold up.' },
  { slug: 'decks',      num: 6, title: 'All decks',         teaser: 'Compatibility graphs and maximal cliques.' },
  { slug: 'conclusion', num: 7, title: 'Conclusion',        teaser: 'Wrap-up and open questions.' },
];

export function findSection(pathname) {
  // Pathname is "/", "/puzzle", "/puzzle/" etc. Strip leading + trailing slashes.
  const slug = pathname.replace(/^\/|\/$/g, '');
  const idx = sections.findIndex((s) => s.slug === slug);
  if (idx < 0) return null;
  return {
    current: sections[idx],
    prev: idx > 0 ? sections[idx - 1] : null,
    next: idx < sections.length - 1 ? sections[idx + 1] : null,
  };
}
