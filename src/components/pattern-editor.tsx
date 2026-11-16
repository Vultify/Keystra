import { useState } from "react";
import { CheckIcon, HistoryIcon, RefreshCwIcon, SaveIcon, UndoIcon } from "lucide-react";

import { ErrorAlert } from "@/components/notices";
import { Alert, AlertAction, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import type { PatternEditor } from "@/hooks/use-pattern-editor";
import { KEY_LIGHTS } from "@/lib/keyboard-layout";
import { COLOR_SWATCHES } from "@/lib/lighting";

const PAINT_COLORS = [...COLOR_SWATCHES, { name: "Off", color: "#000000" }];

// Paints the keyboard's saved pattern: pick keys on the picture, pick a color, then Save pattern.
export function PatternEditorCard({ number, editor, connected }: { number: number; editor: PatternEditor; connected: boolean }) {
  const [custom, setCustom] = useState("#7A30F8");
  const count = editor.selected.size;
  const canPaint = editor.pattern !== null && count > 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Paint pattern {number}
          {editor.busy === "read" && <Spinner />}
        </CardTitle>
        <CardDescription>
          Click keys on the picture to select them, then pick a color. Your keyboard only changes when you press Save
          pattern, since each save writes its memory.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <ErrorAlert
          title="That didn't work"
          message={editor.error}
          action={
            editor.pattern === null && (
              <Button size="sm" variant="outline" disabled={!connected || editor.busy !== null} onClick={editor.readAgain}>
                <RefreshCwIcon data-icon="inline-start" />
                Read again
              </Button>
            )
          }
        />
        {editor.savedCopy && !editor.dirty && (
          <Alert>
            <HistoryIcon />
            <AlertTitle>This pattern changed since Keystra last saved it</AlertTitle>
            <AlertDescription>The keyboard may have been reset, or the pattern changed in another app.</AlertDescription>
            <AlertAction className="flex gap-2">
              <Button size="sm" variant="outline" disabled={editor.busy !== null} onClick={editor.keepKeyboardVersion}>
                <CheckIcon data-icon="inline-start" />
                Keep this version
              </Button>
              <Button size="sm" variant="outline" disabled={editor.busy !== null} onClick={editor.restoreCopy}>
                Restore Keystra's copy
              </Button>
            </AlertAction>
          </Alert>
        )}

        <div className="flex flex-wrap items-center gap-2">
          <span className="mr-auto text-sm text-muted-foreground">
            {count === 0 ? "No keys selected" : count === 1 ? "1 key selected" : `${count} keys selected`}
          </span>
          <Button size="sm" variant="outline" disabled={count === KEY_LIGHTS.length} onClick={editor.selectAll}>
            Select all
          </Button>
          <Button size="sm" variant="outline" disabled={count === 0} onClick={editor.clearSelection}>
            Clear
          </Button>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {PAINT_COLORS.map((swatch) => (
            <Button
              key={swatch.color}
              size="icon-sm"
              variant="outline"
              aria-label={`Paint ${swatch.name}`}
              title={swatch.name}
              disabled={!canPaint}
              onClick={() => editor.paint(swatch.color)}
            >
              <span className="size-4 rounded-full border" style={{ backgroundColor: swatch.color }} />
            </Button>
          ))}
          <Input
            aria-label="Choose a custom color"
            title="Choose a custom color"
            type="color"
            className="h-8 w-12 p-1"
            value={custom}
            onChange={(event) => setCustom(event.target.value)}
          />
          <Button size="sm" variant="outline" disabled={!canPaint} onClick={() => editor.paint(custom)}>
            <span data-icon="inline-start" className="size-4 rounded-full border" style={{ backgroundColor: custom }} />
            Paint custom
          </Button>
        </div>
      </CardContent>
      <CardFooter className="flex flex-wrap items-center gap-2">
        <span className="mr-auto text-sm text-muted-foreground">{editor.dirty ? "Unsaved changes" : "Matches the keyboard"}</span>
        <Button variant="outline" disabled={!editor.dirty || editor.busy !== null} onClick={editor.discard}>
          <UndoIcon data-icon="inline-start" />
          Discard
        </Button>
        <Button disabled={!editor.dirty || !connected || editor.busy !== null} onClick={editor.save}>
          {editor.busy === "save" ? <Spinner data-icon="inline-start" /> : <SaveIcon data-icon="inline-start" />}
          Save pattern
        </Button>
      </CardFooter>
    </Card>
  );
}
