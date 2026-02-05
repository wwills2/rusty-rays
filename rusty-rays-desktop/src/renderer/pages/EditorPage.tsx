import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Alert, Dialog, Loader } from '@/retro-ui-lib';
import {
  useGetIntersectedUuidByPixelPosMutation,
  useRenderQuery,
} from '@/redux/ipc/tracer.ipc.ts';
import { useGetAllSpheresQuery } from '@/redux/ipc/model.ipc.ts';

type DrawRect = { dx: number; dy: number; dw: number; dh: number };

function computeContainRect(
  canvasW: number,
  canvasH: number,
  imgW: number,
  imgH: number,
): DrawRect {
  const scale = Math.min(canvasW / imgW, canvasH / imgH);
  const dw = imgW * scale;
  const dh = imgH * scale;
  const dx = (canvasW - dw) / 2;
  const dy = (canvasH - dh) / 2;
  return { dx, dy, dw, dh };
}

function resizeCanvasToDisplaySize(canvas: HTMLCanvasElement) {
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect(); // CSS px
  const w = Math.max(1, Math.round(rect.width * dpr));
  const h = Math.max(1, Math.round(rect.height * dpr));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  return { rect };
}

function hoveredPixelFromMouse(
  e: React.MouseEvent<HTMLCanvasElement>,
  canvas: HTMLCanvasElement,
  img: HTMLImageElement,
  drawRect: DrawRect,
): { x: number; y: number } | null {
  const rect = canvas.getBoundingClientRect();

  // Mouse in CSS px relative to canvas
  const cx = e.clientX - rect.left;
  const cy = e.clientY - rect.top;

  // CSS px -> canvas device px
  const sx = canvas.width / rect.width;
  const sy = canvas.height / rect.height;

  const px = cx * sx;
  const py = cy * sy;

  // Outside the area where the image was drawn (letterbox/padding)
  if (
    px < drawRect.dx ||
    py < drawRect.dy ||
    px >= drawRect.dx + drawRect.dw ||
    py >= drawRect.dy + drawRect.dh
  ) {
    return null;
  }

  // Normalized within drawn image rect
  const u = (px - drawRect.dx) / drawRect.dw;
  const v = (py - drawRect.dy) / drawRect.dh;

  // Map to source image pixels
  const x = Math.floor(u * img.width);
  const y = Math.floor(v * img.height);

  return {
    x: Math.min(img.width - 1, Math.max(0, x)),
    y: Math.min(img.height - 1, Math.max(0, y)),
  };
}

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

  const [hover, setHover] = useState<{ x: number; y: number } | null>(null);
  const imageUrl = useMemo(() => {
    if (imageData) {
      const png_data_blob = new Blob([imageData], { type: 'image/png' });
      return URL.createObjectURL(png_data_blob);
    }
  }, [imageData]);

  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);
  const drawRectRef = useRef<DrawRect | null>(null);
  const [dialogBody, setDialogBody] = useState<string | null>(null);

  const draw = () => {
    const canvas = canvasRef.current;
    const img = imgRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    resizeCanvasToDisplaySize(canvas);

    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (!img) return;

    const rect = computeContainRect(
      canvas.width,
      canvas.height,
      img.width,
      img.height,
    );
    drawRectRef.current = rect;

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(img, rect.dx, rect.dy, rect.dw, rect.dh);
  };

  // Load image
  useEffect(() => {
    if (imageUrl) {
      let cancelled = false;
      const img = new Image();
      img.decoding = 'async';
      img.onload = () => {
        if (cancelled) return;
        imgRef.current = img;
        draw(); // initial
      };
      img.onerror = () => {
        if (cancelled) return;
        imgRef.current = null;
        drawRectRef.current = null;
      };
      img.src = imageUrl;

      return () => {
        cancelled = true;
      };
    }
  }, [imageUrl]);

  // Redraw on resize (parent size changes incl. padding changes)
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ro = new ResizeObserver(() => {
      draw();
    });
    ro.observe(canvas);

    // Also redraw if devicePixelRatio changes (zoom / moving window between monitors)
    const mq = window.matchMedia(
      `(resolution: ${window.devicePixelRatio}dppx)`,
    );
    const onDprChange = () => {
      draw();
    };
    mq.addEventListener('change', onDprChange);

    return () => {
      ro.disconnect();
      mq.removeEventListener('change', onDprChange);
    };
  }, []);

  const onMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    const img = imgRef.current;
    const rect = drawRectRef.current;
    if (!canvas || !img || !rect) return;

    const p = hoveredPixelFromMouse(e, canvas, img, rect);
    setHover(p);
  };

  const onMouseLeave = () => {
    setHover(null);
  };

  const onClickPixel = useCallback(() => {
    const execute = async () => {
      if (!hover) return; // not over the canvas / image area
      try {
        const uuid = await getIntersectedUuidByPixelPos({
          x: hover.x,
          y: hover.y,
        }).unwrap();

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
  }, [getIntersectedUuidByPixelPos, hover, spheresMap]);

  return (
    <div className="w-full h-full items-center justify-center">
      {isRendering ? (
        <Loader />
      ) : (
        <div className="w-full h-full">
          {renderError ? (
            <Alert>An error occurred</Alert>
          ) : (
            <div className="h-full w-full">
              <div className="w-full h-full">
                <canvas
                  ref={canvasRef}
                  onMouseMove={onMouseMove}
                  onMouseLeave={onMouseLeave}
                  onClick={onClickPixel}
                  style={{
                    width: '100%',
                    height: '100%',
                    display: 'block',
                  }}
                />
                <div
                  style={{
                    position: 'absolute',
                    left: 8,
                    top: 8,
                    padding: '4px 8px',
                    background: 'rgba(0,0,0,0.6)',
                    color: 'white',
                    fontFamily: 'monospace',
                    fontSize: 12,
                    borderRadius: 6,
                    pointerEvents: 'none',
                  }}
                >
                  {hover ? `x=${hover.x}, y=${hover.y}` : '—'}
                </div>
              </div>
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
