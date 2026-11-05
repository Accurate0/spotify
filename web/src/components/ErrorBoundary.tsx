import { Component, type ReactNode } from "react";

type Props = {
  children: ReactNode;
  /** Changing this (e.g. on navigation) clears a caught error. */
  resetKey?: string;
};

type State = { error: Error | null; resetKey?: string };

/** Shows a failed query (or render) in place instead of blanking the whole app. */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, resetKey: this.props.resetKey };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  static getDerivedStateFromProps(props: Props, state: State): Partial<State> | null {
    return props.resetKey !== state.resetKey ? { error: null, resetKey: props.resetKey } : null;
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="bg-card rounded-xl border px-6 py-8 text-center">
        <p className="font-medium">Something went wrong loading this page.</p>
        <p className="text-muted-foreground mt-1 text-sm">{this.state.error.message}</p>
        <button
          type="button"
          onClick={() => this.setState({ error: null })}
          className="hover:bg-accent mt-4 rounded-md border px-3 py-1.5 text-sm"
        >
          Try again
        </button>
      </div>
    );
  }
}
