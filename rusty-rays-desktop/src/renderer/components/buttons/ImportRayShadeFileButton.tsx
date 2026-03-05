import { Button } from '@/retro-ui-lib';
import React, { useCallback, useEffect, useRef } from 'react';
import { useLoadModelFromFileMutation } from '@/redux/ipc/model.ipc.ts';
import { useGetFileText } from '@/hooks';
import _ from 'lodash';
import { useNavigate } from 'react-router';
import ROUTES from '@/routes/route-constants.ts';

interface Props {
  label: string;
}

const ImportRayShadeFileButton: React.FC<Props> = ({ label }) => {
  const navigate = useNavigate();
  const [
    triggerLoadModel,
    { isLoading: modelIsLoading, error: loadModelError },
  ] = useLoadModelFromFileMutation();

  const inputRef = useRef<HTMLInputElement>(null);
  const [processFile, fileText, fileTextLoading, fileTextError] =
    useGetFileText();

  useEffect(() => {
    const load = async () => {
      if (fileText && !fileTextLoading && !fileTextError) {
        const loadResult = await triggerLoadModel(fileText);
        if (loadResult.data) {
          void navigate(ROUTES.EDITOR);
        }
      }
    };

    load().catch((error: unknown) => {
      console.error(
        'An error occurred loading model from file',
        JSON.stringify(error),
      );
    });
  }, [fileText, fileTextError, fileTextLoading, navigate, triggerLoadModel]);

  const handleFileChange = useCallback<
    React.ChangeEventHandler<HTMLInputElement>
  >(
    (event) => {
      const file = event.target.files?.[0];
      if (file !== undefined) {
        processFile(file);
      }
    },
    [processFile],
  );

  return (
    <>
      <Button
        disabled={modelIsLoading || !_.isNil(loadModelError)}
        onClick={() => inputRef.current?.click()}
      >
        {label}
      </Button>
      <input
        ref={inputRef}
        type="file"
        className="hidden"
        onChange={handleFileChange}
      />
    </>
  );
};

export { ImportRayShadeFileButton };
