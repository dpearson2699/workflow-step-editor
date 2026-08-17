// PROTOTYPE — throwaway. Review-UI variants (?variant=) plus the D app flow:
// landing → record (live streaming rows) → stop → naming dialog → review.
// Branch: prototype/map-1-8. Wayfinder ticket: dpearson2699/workflow-step-editor#8.
import { useRef, useState } from 'react';
import type { Permissions, StepData } from './data';
import { initialSteps } from './data';
import Home from './Home';
import PrototypeSwitcher from './PrototypeSwitcher';
import VariantA from './variants/VariantA';
import VariantB from './variants/VariantB';
import VariantC from './variants/VariantC';
import VariantD from './variants/VariantD';

const VARIANTS = ['A', 'B', 'C', 'D'];

function NameModal(props: { stepCount: number; onSave: (name: string) => void }) {
  const [name, setName] = useState(() =>
    'Recording — ' + new Date().toLocaleString('en-US', { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' }));
  return (
    <div className="modal-scrim">
      <div className="modal">
        <h3>Save recording</h3>
        <p>{props.stepCount} steps captured. Events are already on disk — naming finishes the save, and edits save automatically from here on.</p>
        <input autoFocus value={name} onChange={e => setName(e.target.value)} onFocus={e => e.target.select()} />
        <button className="modal-save" onClick={() => props.onSave(name)}>Save</button>
      </div>
    </div>
  );
}

export default function App() {
  const [variant, setVariantState] = useState(() =>
    new URLSearchParams(window.location.search).get('variant')?.toUpperCase() ?? 'A');
  const setVariant = (v: string) => {
    const url = new URL(window.location.href);
    url.searchParams.set('variant', v);
    history.replaceState(null, '', url);
    setVariantState(v);
  };

  const [view, setViewState] = useState(() =>
    new URLSearchParams(window.location.search).get('view') ?? 'home');
  const setView = (w: string) => {
    const url = new URL(window.location.href);
    url.searchParams.set('view', w);
    history.replaceState(null, '', url);
    setViewState(w);
  };

  const [steps, setSteps] = useState<StepData[]>(initialSteps);
  const [wfName, setWfName] = useState('Approve invoice');
  const [recording, setRecording] = useState(false);
  const [draft, setDraft] = useState(false);
  const [naming, setNaming] = useState(false);
  const [perms, setPerms] = useState<Permissions>({ input: true, accessibility: true, screen: false });
  const recTimer = useRef<number | null>(null);

  const startRecording = () => {
    setSteps([]);
    setWfName('New Recording');
    setView('detail');
    setRecording(true);
    let i = 0;
    recTimer.current = window.setInterval(() => {
      i += 1;
      if (i <= initialSteps.length) setSteps(ss => [...ss, initialSteps[i - 1]]);
      if (i >= initialSteps.length && recTimer.current) clearInterval(recTimer.current);
    }, 1400);
  };
  const stopRecording = () => {
    if (recTimer.current) clearInterval(recTimer.current);
    setRecording(false);
    setDraft(true);
  };
  const discardDraft = () => {
    if (window.confirm('Discard this recording? Its captured events and screenshots will be removed.')) {
      setDraft(false);
      setSteps([]);
      setView('home');
    }
  };

  const p = {
    steps,
    updateStep: (id: string, patch: Partial<StepData>) =>
      setSteps(ss => ss.map(s => (s.id === id ? { ...s, ...patch } : s))),
    deleteStep: (id: string) => setSteps(ss => ss.filter(s => s.id !== id)),
    recording,
    toggleRecording: () => (recording ? stopRecording() : startRecording()),
    perms,
    requestPerm: (k: keyof Permissions) =>
      setTimeout(() => setPerms(pp => ({ ...pp, [k]: true })), 600) && undefined,
  };

  const openWorkflow = () => {
    setSteps(initialSteps);
    setWfName('Approve invoice');
    setView('detail');
  };

  const v =
    variant === 'D' && view === 'home'
      ? <Home openWorkflow={openWorkflow} recording={recording}
          toggleRecording={p.toggleRecording} perms={perms} requestPerm={p.requestPerm} />
      : variant === 'B' ? <VariantB {...p} />
      : variant === 'C' ? <VariantC {...p} />
      : variant === 'D' ? <VariantD {...p} wfName={wfName} draft={draft}
          onSaveDraft={() => setNaming(true)} onDiscardDraft={discardDraft}
          onBack={() => setView('home')} />
      : <VariantA {...p} />;
  return (
    <>
      {v}
      {naming && <NameModal stepCount={steps.length} onSave={n => { setWfName(n); setNaming(false); setDraft(false); }} />}
      <PrototypeSwitcher variants={VARIANTS} current={VARIANTS.includes(variant) ? variant : 'A'} onChange={setVariant} />
    </>
  );
}
