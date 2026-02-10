import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

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

interface RenderedImageCanvasWigetProps {
  imageData: Uint8Array<ArrayBuffer>;
  onClickImagePixel: (x: number, y: number) => void;
}

const RenderedImageCanvasWidget: React.FC<RenderedImageCanvasWigetProps> = ({
  imageData,
  onClickImagePixel,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);
  const drawRectRef = useRef<DrawRect | null>(null);

  const draw = () => {
    const canvas = canvasRef.current;
    const image = imgRef.current;
    if (!canvas) return;

    const canvasContext = canvas.getContext('2d');
    if (!canvasContext) return;

    resizeCanvasToDisplaySize(canvas);

    canvasContext.setTransform(1, 0, 0, 1, 0, 0);
    canvasContext.clearRect(0, 0, canvas.width, canvas.height);

    if (!image) return;

    const containingRect = computeContainRect(
      canvas.width,
      canvas.height,
      image.width,
      image.height,
    );
    drawRectRef.current = containingRect;

    canvasContext.imageSmoothingEnabled = true;
    canvasContext.imageSmoothingQuality = 'high';
    canvasContext.drawImage(
      image,
      containingRect.dx,
      containingRect.dy,
      containingRect.dw,
      containingRect.dh,
    );
  };

  // Redraw on resize (parent size changes incl. padding changes)
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resizeObserver = new ResizeObserver(() => {
      draw();
    });
    resizeObserver.observe(canvas);

    // Also redraw if devicePixelRatio changes (zoom / moving window between monitors)
    const mq = window.matchMedia(
      `(resolution: ${window.devicePixelRatio}dppx)`,
    );
    const onDprChange = () => {
      draw();
    };
    mq.addEventListener('change', onDprChange);

    return () => {
      resizeObserver.disconnect();
      mq.removeEventListener('change', onDprChange);
    };
  }, []);

  const [hover, setHover] = useState<{ x: number; y: number } | null>(null);
  const imageUrl = useMemo(() => {
    const png_data_blob = new Blob([imageData], { type: 'image/png' });
    return URL.createObjectURL(png_data_blob);
  }, [imageData]);

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

  const handleCanvasClick = useCallback(() => {
    if (hover) {
      onClickImagePixel(hover.x, hover.y);
    }
  }, [hover, onClickImagePixel]);

  const onMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    const img = imgRef.current;
    const rect = drawRectRef.current;
    if (!canvas || !img || !rect) return;

    const pixelPosition = hoveredPixelFromMouse(e, canvas, img, rect);
    setHover(pixelPosition);
  }, []);

  const onMouseLeave = useCallback(() => {
    setHover(null);
  }, []);

  return (
    <div className="w-full h-full">
      <canvas
        ref={canvasRef}
        onMouseMove={onMouseMove}
        onMouseLeave={onMouseLeave}
        onClick={handleCanvasClick}
        style={{
          width: '100%',
          height: '100%',
          display: 'block',
          position: 'relative',
          background: 'black',
        }}
      />
      <div
        style={{
          position: 'absolute',
          left: 8,
          top: 8,
          padding: '4px 8px',
          background: 'rgba(100, 100 , 100)',
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
  );
};

export { RenderedImageCanvasWidget };
