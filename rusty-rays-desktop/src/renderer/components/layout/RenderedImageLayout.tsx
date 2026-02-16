import { Alert, Card, Dialog, Loader } from '@/retro-ui-lib';
import { RenderedImageCanvasWidget } from '@/components';
import React, { useCallback, useEffect, useState } from 'react';

import {
  useGetRenderStatusQuery,
  useGetTracerInstanceUuidQuery,
  useLazyGetIntersectedUuidByPixelPosQuery,
  useLazyLoadRenderImageQuery,
  useRenderMutation,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';
import { loadLatestRender } from '@/indexed-db-image-cache.ts';

const RenderedImageLayout: React.FC = () => {
  const [pollRenderProgress, setPollRenderProgress] = useState(false);
  const { data: tracerInstanceUuid, isLoading: tracerInstanceUuidLoading } =
    useGetTracerInstanceUuidQuery(null);
  const { data: renderStatus } = useGetRenderStatusQuery(null, {
    pollingInterval: pollRenderProgress ? 100 : 0,
    skip: !tracerInstanceUuid || tracerInstanceUuidLoading,
  });
  const { data: spheresMap } = useGetAllSpheresQuery(null, {
    skip: !tracerInstanceUuid || tracerInstanceUuidLoading,
  });
  const [triggerLoadRenderImage, { error: loadRenderImageError }] =
    useLazyLoadRenderImageQuery();
  const [triggerRender] = useRenderMutation();
  const [triggerGetIntersectedUuid] =
    useLazyGetIntersectedUuidByPixelPosQuery();

  const [imageData, setImageData] = useState<Uint8Array<ArrayBuffer> | null>(
    null,
  );
  const [dialogMessage, setDialogMessage] = useState<string | null>(null);

  useEffect(() => {
    const execute = async () => {
      if (renderStatus && tracerInstanceUuid) {
        if (renderStatus.renderInProgress) {
          setPollRenderProgress(true);
        } else if (renderStatus.renderImageAvailable) {
          try {
            setPollRenderProgress(false);
            await triggerLoadRenderImage(tracerInstanceUuid);
          } catch (error) {
            console.error('Failed to load render image:', error);
          }
        }

        const cachedImage = await loadLatestRender(tracerInstanceUuid);
        if (cachedImage) {
          setImageData(cachedImage);
        } else {
          setPollRenderProgress(true);
          await triggerRender(null);
        }
      }
    };

    execute().catch(console.error);
  }, [renderStatus, tracerInstanceUuid, triggerLoadRenderImage, triggerRender]);

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
            setDialogMessage(JSON.stringify(sphere, null, 2));
          } else {
            setDialogMessage('The object information could not be retrieved.');
          }
        } catch {
          // on IPC error, show retrieval message
          setDialogMessage('The object information could not be retrieved.');
        }
      };

      execute().catch(console.error);
    },
    [triggerGetIntersectedUuid, spheresMap],
  );

  return (
    <Card className="flex-1 min-h-0">
      <Card.Content className="p-2 w-full h-full items-center justify-center">
        {!renderStatus || renderStatus.renderInProgress || !imageData ? (
          <div className="flex flex-col h-full w-full items-center justify-center bg-muted">
            <Alert className="flex flex-col lg:max-w-2/3 max-w-1/2">
              <Alert.Title className="text-center">
                Render in progress...
              </Alert.Title>
              <div className="flex items-center justify-center pt-2">
                <Loader count={10} delayStep={60} />
              </div>
            </Alert>
          </div>
        ) : (
          <>
            {renderStatus.renderErrorMsg || loadRenderImageError ? (
              <Alert>{`An error occurred: ${renderStatus.renderErrorMsg || 'Failed to load render image'}`}</Alert>
            ) : (
              <>
                <RenderedImageCanvasWidget
                  imageData={imageData}
                  onClickImagePixel={handlePixelClick}
                />
                {dialogMessage && (
                  <Dialog
                    open
                    onOpenChange={() => {
                      setDialogMessage(null);
                    }}
                  >
                    <Dialog.Content>
                      <Dialog.Header>Sphere info</Dialog.Header>
                      <Dialog.Description
                        style={{
                          whiteSpace: 'pre-wrap',
                          fontSize: 12,
                        }}
                      >
                        {dialogMessage || null}
                      </Dialog.Description>
                    </Dialog.Content>
                  </Dialog>
                )}
              </>
            )}
          </>
        )}
      </Card.Content>
    </Card>
  );
};

export { RenderedImageLayout };
