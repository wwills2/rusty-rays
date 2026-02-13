import { Button, Dialog } from '@/retro-ui-lib';
import React, { useCallback, useState } from 'react';
import { useSetModelMutation } from '@/redux/ipc/model.ipc.ts';
import ROUTES from '@/routes/route-constants.ts';
import { useNavigate } from 'react-router';

const CloseModelButton: React.FC = () => {
  const navigate = useNavigate();
  const [triggerSetModel, { isLoading }] = useSetModelMutation();
  const [errorMessage, setErrorMessage] = useState<string>('');
  const onClickClose = useCallback(() => {
    navigate(ROUTES.LANDING)?.catch((error: unknown) => {
      console.log('failed to navigate:', error);
    });
    triggerSetModel(undefined).catch((error: unknown) => {
      console.error('Failed to close model:', error);
      if (error instanceof Error) {
        setErrorMessage(error.message);
      } else if (error) {
        setErrorMessage('An unknown error occurred while closing the model.');
      } else {
        setErrorMessage('');
      }
    });
  }, [navigate, triggerSetModel]);

  return (
    <>
      <Button disabled={isLoading} onClick={onClickClose}>
        Close Model
      </Button>
      {errorMessage && (
        <Dialog
          open
          onOpenChange={() => {
            setErrorMessage('');
          }}
        >
          <Dialog.Header>Unable to close current model</Dialog.Header>
          <Dialog.Content>{errorMessage}</Dialog.Content>
        </Dialog>
      )}
    </>
  );
};

export { CloseModelButton };
