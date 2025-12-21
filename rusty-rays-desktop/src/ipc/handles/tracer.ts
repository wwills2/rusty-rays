import { handle } from '#/ipc/handles/index';
import { getModel } from '#/model-manager';
import { Tracer } from 'rusty-rays-napi-node/index';
import { toIpcError } from '#/ipc/shared';

function initTracerChannels() {
  handle('tracer:Render', async () => {
    const model = getModel();
    if (model === undefined) {
      return {
        error: new Error('No model loaded. A model must be loaded to render'),
      };
    }

    try {
      const tracer = new Tracer(model);
      const png_buffer = await tracer.renderToImageBuffer('png');
      const png_array_buffer = new Uint8Array(png_buffer).slice().buffer;
      return { data: png_array_buffer };
    } catch (error: unknown) {
      return toIpcError(error, 'Render failed');
    }
  });
}

export { initTracerChannels };
