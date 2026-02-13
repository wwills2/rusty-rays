import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Alert, Dialog, Loader } from '@/retro-ui-lib';
import {
  useGetRenderStatusQuery,
  useGetTracerInstanceUuidQuery,
  useLazyGetIntersectedUuidByPixelPosQuery,
  useLazyLoadRenderImageQuery,
  useRenderMutation,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';
import { CloseModelButton, RenderedImageCanvasWidget } from '@/components';
import { loadLatestRender } from '@/indexed-db-image-cache.ts';
import { useNavigate } from 'react-router';
import ROUTES from '@/routes/route-constants.ts';

const EditorPage: React.FC = () => {
  const navigate = useNavigate();
  // tracer needs to be loaded for this page to work
  const { data: tracerInstanceUuid, isLoading: tracerInstanceUuidLoading } =
    useGetTracerInstanceUuidQuery(null);
  const tracerLoaded = useMemo(
    () =>
      !tracerInstanceUuid ||
      !(tracerInstanceUuid === 'TRACER_INSTANCE_NOT_LOADED'),
    [tracerInstanceUuid],
  );

  // navigate off the page if tracer is not loaded
  useEffect(() => {
    if (!tracerLoaded) {
      navigate(ROUTES.LANDING)?.catch((error: unknown) => {
        console.log('failed to navigate:', error);
      });
    }
  }, [navigate, tracerLoaded]);

  const [pollRenderProgress, setPollRenderProgress] = useState(false);
  const { data: renderStatus } = useGetRenderStatusQuery(null, {
    pollingInterval: pollRenderProgress ? 100 : 0,
    skip: !tracerLoaded || tracerInstanceUuidLoading,
  });
  const { data: spheresMap } = useGetAllSpheresQuery(null, {
    skip: !tracerLoaded || tracerInstanceUuidLoading,
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
    <div className="flex flex-col w-full h-full items-center justify-center">
      {!renderStatus || renderStatus.renderInProgress || !imageData ? (
        <div>
          <Loader />
        </div>
      ) : (
        <div className="w-full h-full">
          {renderStatus.renderErrorMsg || loadRenderImageError ? (
            <Alert>{`An error occurred: ${renderStatus.renderErrorMsg || 'Failed to load render image'}`}</Alert>
          ) : (
            <div className="flex flex-col h-full w-full">
              <div>
                <CloseModelButton />
              </div>
              <div className="flex-1 min-h-0">
                <RenderedImageCanvasWidget
                  imageData={imageData}
                  onClickImagePixel={handlePixelClick}
                />
              </div>
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
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export { EditorPage };
