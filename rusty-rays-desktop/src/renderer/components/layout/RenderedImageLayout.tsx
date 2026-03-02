import { Alert, Card, Dialog, Loader, Progress } from '@/retro-ui-lib';
import { RenderedImageCanvasWidget } from '@/components';
import React, { useCallback, useEffect, useState } from 'react';

import {
  useGetRenderStatusQuery,
  useGetTracerInstanceUuidQuery,
  useLazyGetIntersectedObjectByPixelPosQuery,
  useLazyLoadRenderImageQuery,
  useRenderMutation,
} from '@/redux/ipc/tracer.ipc.ts';
import {
  useGetAllConesQuery,
  useGetAllPolygonsQuery,
  useGetAllSpheresQuery,
  useGetAllTrianglesQuery,
} from '@/redux/ipc/model.ipc.ts';
import { loadLatestRender } from '@/indexed-db-image-cache.ts';
import _ from 'lodash';

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
  const { data: conesMap } = useGetAllConesQuery(null, {
    skip: !tracerInstanceUuid || tracerInstanceUuidLoading,
  });
  const { data: trianglesMap } = useGetAllTrianglesQuery(null, {
    skip: !tracerInstanceUuid || tracerInstanceUuidLoading,
  });
  const { data: polygonsMap } = useGetAllPolygonsQuery(null, {
    skip: !tracerInstanceUuid || tracerInstanceUuidLoading,
  });
  const [
    triggerLoadRenderImage,
    { isLoading: loadingRenderImage, error: loadRenderImageError },
  ] = useLazyLoadRenderImageQuery();
  const [triggerRender] = useRenderMutation();
  const [triggerGetIntersectedUuid] =
    useLazyGetIntersectedObjectByPixelPosQuery();

  const [imageData, setImageData] = useState<Uint8Array<ArrayBuffer> | null>(
    null,
  );
  const [dialogMessage, setDialogMessage] = useState<string | null>(null);

  useEffect(() => {
    const execute = async () => {
      if (renderStatus && tracerInstanceUuid) {
        if (renderStatus.renderProgressPercentage) {
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
        } else if (
          _.isNil(renderStatus.renderProgressPercentage) &&
          !renderStatus.writingImage
        ) {
          setPollRenderProgress(true);
          console.debug('triggering render. status:', renderStatus);
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
          const intersectedInfo = await triggerGetIntersectedUuid({
            x,
            y,
          }).unwrap();

          if (!intersectedInfo) {
            // no intersection — do not open dialog
            return;
          }

          const { uuid, objectType } = intersectedInfo;
          switch (objectType) {
            case 'sphere': {
              const sphere = spheresMap ? spheresMap[uuid] : undefined;
              if (sphere) {
                setDialogMessage(JSON.stringify(sphere, null, 2));
              }
              break;
            }
            case 'cone': {
              const cone = conesMap ? conesMap[uuid] : undefined;
              if (cone) {
                setDialogMessage(JSON.stringify(cone, null, 2));
              }
              break;
            }
            case 'triangle': {
              const triangle = trianglesMap ? trianglesMap[uuid] : undefined;
              if (triangle) {
                const { derived: _derived, ...triangleRest } = triangle;
                setDialogMessage(JSON.stringify(triangleRest, null, 2));
              }
              break;
            }
            case 'polygon': {
              const polygon = polygonsMap ? polygonsMap[uuid] : undefined;
              if (polygon) {
                const { derived: _derived, ...polygonRest } = polygon;
                setDialogMessage(JSON.stringify(polygonRest, null, 2));
              }
              break;
            }
            default: {
              setDialogMessage(
                'The object information could not be retrieved.',
              );
            }
          }
        } catch {
          // on IPC error, show retrieval message
          setDialogMessage('The object information could not be retrieved.');
        }
      };

      execute().catch(console.error);
    },
    [
      triggerGetIntersectedUuid,
      spheresMap,
      conesMap,
      trianglesMap,
      polygonsMap,
    ],
  );

  return (
    <Card className="flex-1 min-h-0">
      <Card.Content className="p-2 w-full h-full items-center justify-center">
        {!renderStatus ||
        renderStatus.renderProgressPercentage ||
        !imageData ? (
          <div className="flex flex-col h-full w-full items-center justify-center bg-muted">
            <Alert className="flex flex-col lg:max-w-2/3 max-w-1/2">
              {renderStatus?.writingImage ? (
                <>
                  <Alert.Title className="text-center">
                    Encoding image...
                  </Alert.Title>
                  <div className="flex items-center justify-center pt-2">
                    <Loader count={10} delayStep={60} />
                  </div>
                </>
              ) : (
                <>
                  {loadingRenderImage ||
                  !renderStatus?.renderProgressPercentage ? (
                    <>
                      <Alert.Title className="text-center">
                        Loading...
                      </Alert.Title>
                      <div className="flex items-center justify-center pt-2">
                        <Loader count={10} delayStep={60} />
                      </div>
                    </>
                  ) : (
                    <>
                      <Alert.Title className="text-center">
                        Render in progress...
                      </Alert.Title>
                      <div className="flex items-center justify-center pt-2">
                        <Progress
                          value={renderStatus.renderProgressPercentage || 0}
                        />
                      </div>
                    </>
                  )}
                </>
              )}
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
