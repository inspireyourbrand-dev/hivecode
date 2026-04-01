import React, { ReactNode, ErrorInfo } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * ErrorBoundary - A React class component that catches rendering errors
 * and displays a branded error fallback UI with recovery options.
 *
 * Features:
 * - Catches render errors via componentDidCatch and getDerivedStateFromError
 * - Shows HiveCode branded error fallback with gradient hexagon logo
 * - Displays error message and full stack trace in console
 * - Provides "Reload" and "Try Again" recovery options
 * - Uses dark theme consistent with HiveCode design system
 */
export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error details to console with full stack trace
    console.error("=== ErrorBoundary Caught Error ===");
    console.error("Error:", error);
    console.error("Component Stack:", errorInfo.componentStack);
    console.error("Stack Trace:", error.stack);
    console.error("==================================");

    // Update state with error info
    this.setState({
      errorInfo,
    });
  }

  handleReload = () => {
    window.location.reload();
  };

  handleTryAgain = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen w-screen items-center justify-center bg-hive-bg text-white overflow-hidden">
          {/* Background gradient effect */}
          <div className="absolute inset-0 bg-gradient-to-br from-hive-bg via-hive-surface to-hive-bg opacity-50" />

          {/* Error content container */}
          <div className="relative z-10 max-w-md w-full mx-4 text-center">
            {/* HiveCode Hexagon Logo with Gradient */}
            <div className="mb-8 flex justify-center">
              <svg
                width="120"
                height="120"
                viewBox="0 0 120 120"
                className="drop-shadow-lg"
                aria-hidden="true"
              >
                <defs>
                  <linearGradient
                    id="hexagon-gradient"
                    x1="0%"
                    y1="0%"
                    x2="100%"
                    y2="100%"
                  >
                    <stop offset="0%" stopColor="#3ebaf4" />
                    <stop offset="100%" stopColor="#df30ff" />
                  </linearGradient>
                </defs>
                {/* Hexagon outline */}
                <polygon
                  points="60,10 110,35 110,85 60,110 10,85 10,35"
                  fill="none"
                  stroke="url(#hexagon-gradient)"
                  strokeWidth="2"
                />
                {/* Inner hexagon for visual interest */}
                <polygon
                  points="60,25 100,45 100,75 60,95 20,75 20,45"
                  fill="none"
                  stroke="url(#hexagon-gradient)"
                  strokeWidth="1"
                  opacity="0.5"
                />
              </svg>
            </div>

            {/* Error heading */}
            <h1 className="text-4xl font-bold mb-4 bg-gradient-to-r from-cyan-400 to-pink-500 bg-clip-text text-transparent">
              Something went wrong
            </h1>

            {/* Error message */}
            <div className="mb-8">
              <p className="text-gray-300 text-lg mb-4">
                An unexpected error occurred in HiveCode.
              </p>
              {this.state.error && (
                <div className="bg-hive-surface rounded-lg p-4 text-left max-h-48 overflow-y-auto">
                  <p className="text-sm font-mono text-red-400 break-words whitespace-pre-wrap">
                    {this.state.error.toString()}
                  </p>
                </div>
              )}
            </div>

            {/* Action buttons */}
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              {/* Try Again button - resets error state */}
              <button
                onClick={this.handleTryAgain}
                className="px-6 py-3 bg-gradient-to-r from-cyan-500 to-cyan-600 hover:from-cyan-600 hover:to-cyan-700 text-white font-semibold rounded-lg transition-all duration-200 ease-out active:scale-95 focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:ring-offset-2 focus:ring-offset-hive-bg"
                aria-label="Try again"
              >
                Try Again
              </button>

              {/* Reload button - full page reload */}
              <button
                onClick={this.handleReload}
                className="px-6 py-3 bg-gradient-to-r from-magenta-500 to-magenta-600 hover:from-magenta-600 hover:to-magenta-700 text-white font-semibold rounded-lg transition-all duration-200 ease-out active:scale-95 focus:outline-none focus:ring-2 focus:ring-magenta-400 focus:ring-offset-2 focus:ring-offset-hive-bg"
                aria-label="Reload the application"
              >
                Reload
              </button>
            </div>

            {/* Help text */}
            <p className="text-gray-500 text-sm mt-8">
              If the problem persists, please check the browser console for
              more details.
            </p>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
