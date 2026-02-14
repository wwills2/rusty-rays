import React, { useEffect } from 'react';
import { useGetTracerInstanceUuidQuery } from '@/redux/ipc/tracer.ipc.ts';
import { useNavigate } from 'react-router';
import ROUTES from '@/routes/route-constants.ts';
import {
  RenderedImageLayout,
  RenderImageActionHeaderLayout,
} from '@/components/layout';
import { ModelObjectExplorerLayout } from '@/components/layout/ModelObjectExplorerLayout.tsx';

const EditorPage: React.FC = () => {
  const navigate = useNavigate();
  // tracer needs to be loaded for this page to work
  const { data: tracerInstanceUuid, isLoading: tracerInstanceUuidLoading } =
    useGetTracerInstanceUuidQuery(null);

  // navigate off the page if tracer is not loaded
  useEffect(() => {
    if (!tracerInstanceUuid && !tracerInstanceUuidLoading) {
      navigate(ROUTES.LANDING)?.catch((error: unknown) => {
        console.log('failed to navigate:', error);
      });
    }
  }, [navigate, tracerInstanceUuid, tracerInstanceUuidLoading]);

  return (
    <div className="grid grid-rows-1 grid-cols-6 gap-4 p-4 w-full h-full bg-accent dark:bg-background">
      <div className="flex flex-col col-span-4 col-start-1 row-start-1 row-span-1 space-y-4">
        <RenderImageActionHeaderLayout />
        <RenderedImageLayout />
      </div>
      <div className="col-span-2 col-start-5 row-start-1 row-span-1">
        <ModelObjectExplorerLayout />
      </div>
    </div>
  );
};

export { EditorPage };
