// PROTOTYPE — throwaway. Three variants of the workflow step review UI,
// switchable via ?variant=, on this single route. Branch: prototype/map-1-8.
// Stub data honors schema v1: 1:1 steps, shortcut key-downs, element metadata
// with fallback, screenshot triple per event.

export type Classification = 'click' | 'type' | 'wait' | 'assert';

export interface StepData {
  id: string;
  ts: string;
  kind: 'click' | 'key_down';
  classification: Classification;
  title: string;
  description: string;
  app: string;
  windowTitle: string;
  pos: { x: number; y: number } | null;
  key: { chars: string; modifiers: string[] } | null;
  element: { role: string; title: string; source: 'ax' | 'fallback' } | null;
  shots: { full: string; window: string; element: string };
}

const uri = (svg: string) => 'data:image/svg+xml;utf8,' + encodeURIComponent(svg);

function fullShot(hue: number, app: string, click: { x: number; y: number } | null): string {
  const cx = click ? (click.x / 1728) * 1440 : 0;
  const cy = click ? (click.y / 1080) * 900 : 0;
  return uri(`<svg xmlns="http://www.w3.org/2000/svg" width="1440" height="900" viewBox="0 0 1440 900">
  <rect width="1440" height="900" fill="hsl(${hue},18%,26%)"/>
  <rect width="1440" height="28" fill="hsl(${hue},14%,16%)"/>
  <circle cx="18" cy="14" r="5" fill="#999"/><text x="40" y="19" font-family="sans-serif" font-size="13" fill="#ccc">${app}</text>
  <rect x="240" y="90" width="960" height="680" rx="10" fill="hsl(${hue},22%,38%)"/>
  <rect x="240" y="90" width="960" height="34" rx="10" fill="hsl(${hue},20%,30%)"/>
  <circle cx="260" cy="107" r="6" fill="#e0605c"/><circle cx="280" cy="107" r="6" fill="#e0a63c"/><circle cx="300" cy="107" r="6" fill="#61b444"/>
  <rect x="280" y="170" width="620" height="16" rx="4" fill="hsl(${hue},22%,52%)"/>
  <rect x="280" y="210" width="740" height="16" rx="4" fill="hsl(${hue},22%,48%)"/>
  <rect x="280" y="250" width="500" height="16" rx="4" fill="hsl(${hue},22%,50%)"/>
  <rect x="280" y="330" width="300" height="120" rx="8" fill="hsl(${hue},26%,44%)"/>
  <rect x="620" y="330" width="300" height="120" rx="8" fill="hsl(${hue},26%,42%)"/>
  ${click ? `<circle cx="${cx}" cy="${cy}" r="26" fill="none" stroke="#ff5252" stroke-width="5"/><circle cx="${cx}" cy="${cy}" r="6" fill="#ff5252"/>` : ''}
  <text x="20" y="880" font-family="sans-serif" font-size="20" fill="#ffffff66">FULL SCREEN</text>
</svg>`);
}

function windowShot(hue: number, title: string, click: { x: number; y: number } | null): string {
  const cx = click ? 120 + ((click.x % 600)) : 0;
  const cy = click ? 120 + ((click.y % 380)) : 0;
  return uri(`<svg xmlns="http://www.w3.org/2000/svg" width="900" height="620" viewBox="0 0 900 620">
  <rect width="900" height="620" rx="12" fill="hsl(${hue},22%,40%)"/>
  <rect width="900" height="44" rx="12" fill="hsl(${hue},20%,30%)"/>
  <circle cx="24" cy="22" r="7" fill="#e0605c"/><circle cx="48" cy="22" r="7" fill="#e0a63c"/><circle cx="72" cy="22" r="7" fill="#61b444"/>
  <text x="450" y="28" text-anchor="middle" font-family="sans-serif" font-size="15" fill="#eee">${title}</text>
  <rect x="40" y="90" width="640" height="18" rx="4" fill="hsl(${hue},22%,56%)"/>
  <rect x="40" y="130" width="780" height="18" rx="4" fill="hsl(${hue},22%,52%)"/>
  <rect x="40" y="170" width="520" height="18" rx="4" fill="hsl(${hue},22%,54%)"/>
  <rect x="40" y="250" width="280" height="110" rx="8" fill="hsl(${hue},28%,48%)"/>
  <rect x="360" y="250" width="280" height="110" rx="8" fill="hsl(${hue},28%,46%)"/>
  ${click ? `<circle cx="${cx}" cy="${cy}" r="22" fill="none" stroke="#ff5252" stroke-width="5"/><circle cx="${cx}" cy="${cy}" r="5" fill="#ff5252"/>` : ''}
  <text x="16" y="600" font-family="sans-serif" font-size="18" fill="#ffffff66">WINDOW</text>
</svg>`);
}

function elementShot(hue: number, label: string): string {
  return uri(`<svg xmlns="http://www.w3.org/2000/svg" width="360" height="140" viewBox="0 0 360 140">
  <rect width="360" height="140" fill="hsl(${hue},22%,42%)"/>
  <rect x="60" y="40" width="240" height="60" rx="10" fill="hsl(${hue},48%,56%)"/>
  <text x="180" y="76" text-anchor="middle" font-family="sans-serif" font-size="20" fill="#fff">${label}</text>
  <circle cx="180" cy="70" r="30" fill="none" stroke="#ff5252" stroke-width="4"/>
  <text x="10" y="128" font-family="sans-serif" font-size="13" fill="#ffffff66">ELEMENT</text>
</svg>`);
}

function step(
  n: number, ts: string, kind: 'click' | 'key_down', cls: Classification, title: string,
  app: string, windowTitle: string, pos: { x: number; y: number } | null,
  key: { chars: string; modifiers: string[] } | null,
  element: { role: string; title: string; source: 'ax' | 'fallback' } | null,
  hue: number, description = '',
): StepData {
  const elLabel = element ? (element.title || element.role) : 'at click';
  return {
    id: `step_${String(n).padStart(2, '0')}`, ts, kind, classification: cls, title, description,
    app, windowTitle, pos, key, element,
    shots: {
      full: fullShot(hue, app, pos),
      window: windowShot(hue, windowTitle, pos),
      element: elementShot(hue, elLabel),
    },
  };
}

export const initialSteps: StepData[] = [
  step(1, '2026-08-16T22:31:05.123Z', 'click', 'click', 'Click "New Document" — TextEdit', 'TextEdit', 'Untitled',
    { x: 512, y: 384 }, null, { role: 'AXButton', title: 'New Document', source: 'ax' }, 210),
  step(2, '2026-08-16T22:31:08.410Z', 'key_down', 'type', 'Press H — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 'H', modifiers: ['Shift'] }, null, 215),
  step(3, '2026-08-16T22:31:08.720Z', 'key_down', 'type', 'Press e — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 'e', modifiers: [] }, null, 220),
  step(4, '2026-08-16T22:31:08.990Z', 'key_down', 'type', 'Press l — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 'l', modifiers: [] }, null, 225),
  step(5, '2026-08-16T22:31:09.180Z', 'key_down', 'type', 'Press l — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 'l', modifiers: [] }, null, 230),
  step(6, '2026-08-16T22:31:09.400Z', 'key_down', 'type', 'Press o — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 'o', modifiers: [] }, null, 235),
  step(7, '2026-08-16T22:31:12.050Z', 'key_down', 'type', 'Press Cmd+S — TextEdit', 'TextEdit', 'Untitled',
    null, { chars: 's', modifiers: ['Cmd'] }, null, 265,
    'Save the document before switching apps.'),
  step(8, '2026-08-16T22:31:14.300Z', 'click', 'click', 'Click "Save" — TextEdit', 'TextEdit', 'Save As',
    { x: 905, y: 612 }, null, { role: 'AXButton', title: 'Save', source: 'ax' }, 275),
  step(9, '2026-08-16T22:31:19.940Z', 'click', 'click', 'Click at (912, 388) — Chrome', 'Chrome', 'Invoices — Acme ERP',
    { x: 912, y: 388 }, null, { role: 'AXWebArea', title: '', source: 'fallback' }, 20,
    'Element detection fell back to a fixed crop (web content).'),
  step(10, '2026-08-16T22:31:23.310Z', 'click', 'assert', 'Click "Paid" — Chrome', 'Chrome', 'Invoices — Acme ERP',
    { x: 1204, y: 442 }, null, { role: 'AXStaticText', title: 'Paid', source: 'ax' }, 30,
    'Verify the invoice status shows Paid after submission.'),
];

export const CLASSIFICATIONS: Classification[] = ['click', 'type', 'wait', 'assert'];

export const fmtTime = (ts: string) => new Date(ts).toLocaleTimeString('en-US', { hour12: false }) +
  '.' + String(new Date(ts).getMilliseconds()).padStart(3, '0');

export interface Permissions { input: boolean; accessibility: boolean; screen: boolean }

export interface VariantProps {
  steps: StepData[];
  updateStep: (id: string, patch: Partial<StepData>) => void;
  deleteStep: (id: string) => void;
  recording: boolean;
  toggleRecording: () => void;
  perms: Permissions;
  requestPerm: (k: keyof Permissions) => void;
}
