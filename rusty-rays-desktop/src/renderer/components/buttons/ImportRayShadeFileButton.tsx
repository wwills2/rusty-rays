import { Button, Dialog } from '@/retro-ui-lib';
import React, { useCallback, useEffect, useMemo, useRef } from 'react';
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
    {
      isLoading: modelIsLoading,
      error: loadModelError,
      reset: resetLoadModelMutation,
    },
  ] = useLoadModelFromFileMutation();

  const isError = useMemo(() => !_.isNil(loadModelError), [loadModelError]);
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

  const onClickOpenInputFile = useCallback(() => {
    if (inputRef.current) {
      inputRef.current.value = '';
      inputRef.current.click();
    }
  }, []);

  const processedErrorMessage = useMemo(() => {
    let rawErrorMessage = loadModelError?.message;
    if (rawErrorMessage) {
      return rawErrorMessage;
    }

    rawErrorMessage = fileTextError?.message;

    if (rawErrorMessage) {
      const errorMessageParts = rawErrorMessage.split(' Got:');
      return errorMessageParts[0];
    }

    return undefined;
  }, [fileTextError?.message, loadModelError?.message]);

  return (
    <>
      <Button
        disabled={modelIsLoading || isError}
        onClick={onClickOpenInputFile}
      >
        {label}
      </Button>
      <input
        ref={inputRef}
        type="file"
        className="hidden"
        onChange={handleFileChange}
      />
      {isError && (
        <Dialog
          open
          onOpenChange={() => {
            resetLoadModelMutation();
          }}
        >
          <Dialog.Content>
            <Dialog.Header>Error loading model from input file</Dialog.Header>
            <div className="normal-case p-4">{processedErrorMessage}</div>
          </Dialog.Content>
        </Dialog>
      )}
    </>
  );
};

export { ImportRayShadeFileButton };
