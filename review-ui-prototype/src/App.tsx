// PROTOTYPE — throwaway. Three variants of the workflow step review UI,
// switchable via ?variant= on this single route. Branch: prototype/map-1-8.
// Wayfinder ticket: dpearson2699/workflow-step-editor#8.
import { useState } from 'react';
import type { Permissions, StepData } from './data';
import { initialSteps } from './data';
import PrototypeSwitcher from './PrototypeSwitcher';
import VariantA from './variants/VariantA';
import VariantB from './variants/VariantB';
import VariantC from './variants/VariantC';

const VARIANTS = ['A', 'B', 'C'];

export default function App() {
  const [variant, setVariantState] = useState(() =>
    new URLSearchParams(window.location.search).get('variant')?.toUpperCase() ?? 'A');
  const setVariant = (v: string) => {
    const url = new URL(window.location.href);
    url.searchParams.set('variant', v);
    history.replaceState(null, '', url);
    setVariantState(v);
  };

  const [steps, setSteps] = useState<StepData[]>(initialSteps);
  const [recording, setRecording] = useState(false);
  const [perms, setPerms] = useState<Permissions>({ input: true, accessibility: true, screen: false });

  const p = {
    steps,
    updateStep: (id: string, patch: Partial<StepData>) =>
      setSteps(ss => ss.map(s => (s.id === id ? { ...s, ...patch } : s))),
    deleteStep: (id: string) => setSteps(ss => ss.filter(s => s.id !== id)),
    recording,
    toggleRecording: () => setRecording(r => !r),
    perms,
    requestPerm: (k: keyof Permissions) =>
      setTimeout(() => setPerms(pp => ({ ...pp, [k]: true })), 600) && undefined,
  };

  const v = variant === 'B' ? <VariantB {...p} /> : variant === 'C' ? <VariantC {...p} /> : <VariantA {...p} />;
  return (
    <>
      {v}
      <PrototypeSwitcher variants={VARIANTS} current={VARIANTS.includes(variant) ? variant : 'A'} onChange={setVariant} />
    </>
  );
}
