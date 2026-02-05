const TracerChannels = {
  'tracer:Render': {
    args: [],
    dataType: {} as unknown as ArrayBuffer,
  },
  'tracer:GetIntersectedUuidByPixelPos': {
    args: [] as unknown as [i: number, j: number],
    dataType: {} as string | null,
  },
};

export { TracerChannels };
