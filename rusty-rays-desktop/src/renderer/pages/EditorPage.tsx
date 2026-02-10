import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Alert, Dialog, Loader } from '@/retro-ui-lib';
import {
  useGetRenderStatusQuery,
  useGetTracerInstanceUuidQuery,
  useLazyGetIntersectedUuidByPixelPosQuery,
  useLazyLoadRenderImageQuery,
  useRenderMutation,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';
import { RenderedImageCanvasWidget } from '@/components';
import { loadLatestRender } from '@/indexed-db-image-cache.ts';

const EditorPage: React.FC = () => {
  const [pollRenderProgress, setPollRenderProgress] = useState(false);
  const { data: tracerUuid } = useGetTracerInstanceUuidQuery(null);
  const { data: renderStatus } = useGetRenderStatusQuery(null, {
    pollingInterval: pollRenderProgress ? 100 : 0,
  });
  const { data: spheresMap } = useGetAllSpheresQuery(null);
  const [triggerLoadRenderImage, { error: loadRenderImageError }] =
    useLazyLoadRenderImageQuery();
  const [triggerRender] = useRenderMutation();
  const [triggerGetIntersectedUuid] =
    useLazyGetIntersectedUuidByPixelPosQuery();

  const [imageData, setImageData] = useState<Uint8Array<ArrayBuffer> | null>(
    null,
  );

  useEffect(() => {
    const execute = async () => {
      if (renderStatus && tracerUuid) {
        if (renderStatus.renderInProgress) {
          setPollRenderProgress(true);
        } else if (renderStatus.renderImageAvailable) {
          try {
            setPollRenderProgress(false);
            await triggerLoadRenderImage(tracerUuid);
          } catch (error) {
            console.error('Failed to load render image:', error);
          }
        }

        const cachedImage = await loadLatestRender(tracerUuid);
        if (cachedImage) {
          setImageData(cachedImage);
        } else {
          setPollRenderProgress(true);
          await triggerRender(null);
        }
      }
    };

    execute().catch(console.error);
  }, [renderStatus, tracerUuid, triggerLoadRenderImage, triggerRender]);

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
      {!renderStatus || renderStatus.renderInProgress || !imageData ? (
        <Loader />
      ) : (
        <div className="w-full h-full">
          {renderStatus.renderErrorMsg || loadRenderImageError ? (
            <Alert>{`An error occurred: ${renderStatus.renderErrorMsg || 'Failed to load render image'}`}</Alert>
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
