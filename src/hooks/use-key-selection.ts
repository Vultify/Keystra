import { useCallback, useMemo, useState } from "react";

import { KEY_LIGHTS } from "@/lib/keyboard-layout";

export type KeySelection = {
  /** Light numbers of the selected keys. */
  selected: ReadonlySet<number>;
  toggle: (light: number) => void;
  selectAll: () => void;
  clear: () => void;
};

/** Keys picked on the keyboard picture, for any editor that works key by key. */
export function useKeySelection(): KeySelection {
  const [selected, setSelected] = useState<ReadonlySet<number>>(new Set());
  const toggle = useCallback((light: number) => {
    setSelected((current) => {
      const next = new Set(current);
      if (!next.delete(light)) next.add(light);
      return next;
    });
  }, []);
  const selectAll = useCallback(() => setSelected(new Set(KEY_LIGHTS)), []);
  const clear = useCallback(() => setSelected(new Set()), []);
  return useMemo(() => ({ selected, toggle, selectAll, clear }), [selected, toggle, selectAll, clear]);
}
