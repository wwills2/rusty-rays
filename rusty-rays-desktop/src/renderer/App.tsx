import { useCallback } from 'react';
import reactLogo from './assets/react.svg';
import viteLogo from './assets/vite.svg';
import {
  useGetAllSpheresQuery,
  useLoadModelFromFileMutation,
} from '@/redux/ipc/model.ipc.ts';
import { Button } from '@/retro-ui-lib/Button.tsx';

function App() {
  const { data: spheres, refetch: refetchSpheres } =
    useGetAllSpheresQuery(null);
  const [
    triggerLoadModel,
    { isLoading: modelIsLoading, error: loadModelError },
  ] = useLoadModelFromFileMutation();

  const loadModel = useCallback(() => {
    const trigger = async () => {
      const result = await triggerLoadModel(
        '/home/zan/RustroverProjects/rusty-rays/sample-files/sphereflake-reduced-color-mirror.ray',
      );

      console.log('#####', result);

      if (!result.error) {
        await refetchSpheres();
      }
    };

    trigger().catch((error) => console.error(error));
  }, [refetchSpheres, triggerLoadModel]);

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <div className="card card-border">
        <div className="card-body">
          <h2 className="card-title">Spheres</h2>
          <ul>
            {spheres
              ? `loaded ${spheres.length} spheres from model`
              : 'model not loaded'}
          </ul>
        </div>
      </div>
      <h1>Vite + React</h1>
      <div className="card-body">
        <Button onClick={loadModel}>
          {modelIsLoading && <span className="loading loading-spinner" />}
          load model
        </Button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
      {loadModelError && (
        <div role="alert" className="alert alert-error">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-6 w-6 shrink-0 stroke-current"
            fill="none"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span>Error! Task failed successfully.</span>
        </div>
      )}
    </>
  );
}

export default App;
