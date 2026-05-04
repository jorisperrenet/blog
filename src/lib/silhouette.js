// Render a 3×3 silhouette grid as an inline SVG string. Input format is the
// same one used throughout the post: three rows separated by `|`, each row
// three characters of `#` (filled) or `.` (empty). Returns null on bad input
// so callers can fall back to plain text.
export function silhouetteSvg(text, sizePx = 22) {
  const cleaned = String(text ?? '').replace(/\s/g, '');
  if (!/^[#.]+(\|[#.]+)*$/.test(cleaned)) return null;
  const rows = cleaned.split('|');
  if (rows.length !== 3 || rows.some(r => r.length !== 3)) return null;
  const cell = (sizePx - 4) / 3;
  const rects = [];
  for (let z = 0; z < 3; z++) {
    const row = rows[2 - z];
    for (let a = 0; a < 3; a++) {
      if (row[a] === '#') {
        const x = 2 + a * cell;
        const y = 2 + (2 - z) * cell;
        rects.push(`<rect x="${x}" y="${y}" width="${cell}" height="${cell}" fill="#3a3a3a"/>`);
      }
    }
  }
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${sizePx}" height="${sizePx}" viewBox="0 0 ${sizePx} ${sizePx}" style="border:1px solid #888;border-radius:2px;background:#fffbf0">${rects.join('')}</svg>`;
}

// 9-bit silhouette mask → text format. Bit `a*3 + z` is column `a`, row `z`.
export function maskToText(mask) {
  let s = '';
  for (let z = 2; z >= 0; z--) {
    for (let a = 0; a < 3; a++) {
      s += ((mask >> (a * 3 + z)) & 1) ? '#' : '.';
    }
    if (z > 0) s += '|';
  }
  return s;
}
