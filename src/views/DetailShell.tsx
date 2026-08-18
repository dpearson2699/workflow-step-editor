// The detail shell: workflow name header plus `‹ Workflows` back
// navigation (AC-002). PR-02 fills the review pane with the step list
// and the screenshot triple.

export interface DetailShellProps {
  workflowName: string;
  onBack: () => void;
}

export function DetailShell(props: DetailShellProps) {
  return (
    <div className="detail-root">
      <header className="app-header">
        <button type="button" className="back-button" onClick={props.onBack}>
          ‹ Workflows
        </button>
        <h1 className="detail-title">{props.workflowName}</h1>
      </header>
      <main className="detail-pane" aria-label="Workflow detail" />
    </div>
  );
}
