interface FeedbackProps {
  error?: string | null;
  loading?: boolean;
  loadingText?: string;
}

export function Feedback({ error, loading, loadingText }: FeedbackProps) {
  if (loading) {
    return <p role="status">{loadingText ?? "Verarbeite…"}</p>;
  }
  if (error) {
    return (
      <p role="alert" className="error">
        {error}
      </p>
    );
  }
  return null;
}
