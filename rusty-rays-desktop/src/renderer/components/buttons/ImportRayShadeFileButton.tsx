import { Button } from '@/retro-ui-lib';
import React, { useCallback, useEffect, useRef } from 'react';
import { useLoadModelFromFileMutation } from '@/redux/ipc/model.ipc.ts';
import { useGetFileText } from '@/hooks';
import _ from 'lodash';

interface Props {
  label: string;
}

const ImportRayShadeFileButton: React.FC<Props> = ({ label }) => {
  const [
    triggerLoadModel,
    { isLoading: modelIsLoading, error: loadModelError },
  ] = useLoadModelFromFileMutation();

  const inputRef = useRef<HTMLInputElement>(null);
  const [processFile, fileText, fileTextLoading, fileTextError] =
    useGetFileText();

  useEffect(() => {
    if (fileText && !fileTextLoading && !fileTextError) {
      triggerLoadModel(fileText).catch((error: unknown) => {
        console.error('failed to load model', error);
      });
    }
  }, [fileText, fileTextError, fileTextLoading, triggerLoadModel]);

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
