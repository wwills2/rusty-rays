import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Alert, Dialog, Loader } from '@/retro-ui-lib';
import {
  useGetTracerInstanceUuidQuery,
  useIsRenderInProgressQuery,
  useLazyGetIntersectedUuidByPixelPosQuery,
  useLazyLoadRenderImageQuery,
  useRenderMutation,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';
import { RenderedImageCanvasWidget } from '@/components';
import { loadLatestRender } from '@/indexed-db-image-cache.ts';

const EditorPage: React.FC = () => {
  const [pollRenderProgress, setPollRenderProgress] = useState(true);
  const { data: tracerUuid, isLoading: tracerUuidLoading } =
    useGetTracerInstanceUuidQuery(null);
  const { data: isRendering } = useIsRenderInProgressQuery(null, {
    pollingInterval: pollRenderProgress ? 100 : 0,
  });
  const { data: spheresMap } = useGetAllSpheresQuery(null);
  const [triggerLoadRenderImage] = useLazyLoadRenderImageQuery();
  const [triggerRender] = useRenderMutation();
  const [triggerGetIntersectedUuid] =
    useLazyGetIntersectedUuidByPixelPosQuery();

  const [imageData, setImageData] = useState<Uint8Array<ArrayBuffer> | null>(
    null,
  );

  // check if a cached render image is available, or trigger render if not
  useEffect(() => {
    const execute = async () => {
      if (!tracerUuid) {
        console.error(
          'Tracer instance UUID is not available. Ensure a model is loaded.',
        );
        return;
      }

      const cachedImage = await loadLatestRender(tracerUuid);
      if (cachedImage) {
        setImageData(cachedImage);
      } else {
        await triggerRender(null);
        setPollRenderProgress(true);
      }
    };

    if (!tracerUuidLoading && !isRendering && !imageData) {
      execute().catch(console.error);
    }
  }, [imageData, isRendering, tracerUuid, tracerUuidLoading, triggerRender]);

  useEffect(() => {
    if (!isRendering) {
      setPollRenderProgress(false);
    }
  }, []);

  const dialogTriggerRef = useRef<HTMLButtonElement | null>(null);
  const [dialogBody, setDialogBody] = useState<string | null>(null);

  const handlePixelClick = useCallback(
    (x: number, y: number) => {
      const execute = async () => {
        try {
          const uuid = await triggerGetIntersectedUuid({ x, y }).unwrap();

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
    [triggerGetIntersectedUuid, spheresMap],
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
