import { useEffect, useRef } from "react";

export function useAutoScroll(dependency: unknown) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: "smooth", block: "end" });
    }
  }, [dependency]);

  return scrollRef;
}
