import { useCallback, useLayoutEffect, useRef, type RefObject } from "react";

import type { EffectPreview } from "@/lib/effect-motion";
import { lightOf } from "@/lib/key-lights";
import { KEYS } from "@/lib/keyboard-layout";

/** Moving effects redraw at most this often, which is plenty for a preview. */
const FRAME_MS = 33;

type LightLayers = RefObject<(SVGElement | null)[]>[];

/** Whether the system asks for less motion (Windows' "Animation effects" off). */
function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * Draws each key's light by writing straight onto its SVG elements (one per layer, in key order),
 * which React never sets, so no stale frame can stick. While `playing`, moving effects redraw about
 * 30 times a second; when paused or with reduced motion they rest on `restAt`.
 */
export function useKeyLights(
  preview: EffectPreview,
  bases: (string | null)[],
  layers: LightLayers,
  { playing = true, restAt }: { playing?: boolean; restAt?: number } = {},
) {
  const start = useRef(0);
  const currentBases = useRef(bases);
  const currentLayers = useRef(layers);
  currentLayers.current = layers;
  const still = !playing || prefersReducedMotion();

  const paint = useCallback(
    (seconds: number) => {
      KEYS.forEach((key, i) => {
        const { fill, opacity } = lightOf(currentBases.current[i], preview.at(key, seconds));
        for (const layer of currentLayers.current) {
          const element = layer.current[i];
          element?.setAttribute("fill", fill);
          element?.setAttribute("opacity", String(opacity));
        }
      });
    },
    [preview],
  );

  // A new kind of motion starts its loop over (from the rest frame, if there is one); a new color
  // carries on from the same moment.
  useLayoutEffect(() => {
    start.current = performance.now() - (restAt ?? 0) * 1000;
  }, [preview, restAt]);

  useLayoutEffect(() => {
    currentBases.current = bases;
    const seconds = still && restAt !== undefined ? restAt : (performance.now() - start.current) / 1000;
    paint(seconds);
  }, [bases, paint, still, restAt]);

  // A layout effect, so an old loop is always stopped before a new effect's first frame is drawn.
  useLayoutEffect(() => {
    if (still || !preview.moving) return;
    // Playing a resting picture picks up from its rest frame instead of jumping.
    if (restAt !== undefined) start.current = performance.now() - restAt * 1000;
    let frame = 0;
    let last = 0;
    const draw = (now: number) => {
      frame = requestAnimationFrame(draw);
      if (now - last < FRAME_MS) return;
      last = now;
      paint((now - start.current) / 1000);
    };
    frame = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(frame);
  }, [preview, paint, still, restAt]);
}
