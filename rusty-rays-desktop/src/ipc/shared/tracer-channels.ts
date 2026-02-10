const TracerChannels = {
  'tracer:TriggerRender': {
    args: [],
    dataType: {} as true,
  },
  'tracer:GetRenderProgress': {
    args: [],
    dataType: {} as boolean,
  },
  'tracer:GetInstanceUuid': {
    args: [],
    dataType: {} as string,
  },
  'tracer:GetRenderImageData': {
    args: [],
    dataType: {} as ArrayBuffer,
  },
  'tracer:GetIsRenderAvailable': {
    args: [],
    dataType: {} as boolean,
  },
  'tracer:GetIntersectedUuidByPixelPos': {
    args: [] as unknown as [i: number, j: number],
    dataType: {} as string | null,
  },
};

export { TracerChannels };
