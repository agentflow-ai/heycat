import { type ReactNode } from "react";

export interface MainContentProps {
  children: ReactNode;
  /** Additional CSS classes */
  className?: string;
}

export function MainContent({ children, className = "" }: MainContentProps) {
  return (
    <main
      className={`
        flex-1 overflow-auto
        bg-surface
        ${className}
      `}
      role="main"
    >
      <div
        className="
          max-w-[900px] mx-auto
          p-8
        "
      >
        {children}
      </div>
    </main>
  );
}
