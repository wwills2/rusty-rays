import React, { useCallback, useRef, useState } from 'react';
import { Alert, Dialog, Loader } from '@/retro-ui-lib';
import {
  useGetIntersectedUuidByPixelPosMutation,
  useRenderQuery,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';
import { RenderedImageCanvasWidget } from '@/components';

const EditorPage: React.FC = () => {
  const {
    data: imageData,
    isLoading: isRendering,
    error: renderError,
  } = useRenderQuery(null);
  const { data: spheresMap } = useGetAllSpheresQuery(null);

  const [getIntersectedUuidByPixelPos] =
    useGetIntersectedUuidByPixelPosMutation();

  const dialogTriggerRef = useRef<HTMLButtonElement | null>(null);
  const [dialogBody, setDialogBody] = useState<string | null>(null);

  const handlePixelClick = useCallback(
    (x: number, y: number) => {
      const execute = async () => {
        try {
          const uuid = await getIntersectedUuidByPixelPos({ x, y }).unwrap();

          if (!uuid) {
            // no intersection — do not open dialog
            return;
          }

          const sphere = spheresMap ? spheresMap[uuid] : undefined;
          if (sphere) {
            setDialogBody(JSON.stringify(sphere, null, 2));
          } else {
            setDialogBody('The object information could not be retrieved.');
          }

          // open dialog
          dialogTriggerRef.current?.click();
        } catch {
          // on IPC error, show retrieval message
          setDialogBody('The object information could not be retrieved.');
          dialogTriggerRef.current?.click();
        }
      };

      execute().catch(console.error);
    },
    [getIntersectedUuidByPixelPos, spheresMap],
  );

  return (
    <div className="w-full h-full items-center justify-center">
      {isRendering || !imageData ? (
        <Loader />
      ) : (
        <div className="w-full h-full">
          {renderError ? (
            <Alert>An error occurred</Alert>
          ) : (
            <div className="h-full w-full">
              <RenderedImageCanvasWidget
                imageData={imageData}
                onClickImagePixel={handlePixelClick}
              />
              <Dialog>
                <Dialog.Trigger asChild>
                  <button className="hidden" ref={dialogTriggerRef} />
                </Dialog.Trigger>
                <Dialog.Content>
                  <Dialog.Header>Sphere info</Dialog.Header>
                  {dialogBody ? (
                    <pre
                      style={{
                        whiteSpace: 'pre-wrap',
                        fontSize: 12,
                        lineHeight: 1.4,
                        fontFamily: 'monospace',
                      }}
                    >
                      {dialogBody}
                    </pre>
                  ) : null}
                </Dialog.Content>
              </Dialog>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export { EditorPage };
