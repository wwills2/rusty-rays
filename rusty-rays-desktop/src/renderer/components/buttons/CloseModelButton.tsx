import { Button, Dialog } from '@/retro-ui-lib';
import React, { useCallback, useMemo, useState } from 'react';
import { useSetModelMutation } from '@/redux/ipc/model.ipc.ts';
import ROUTES from '@/routes/route-constants.ts';
import { useNavigate } from 'react-router';
import { useGetRenderStatusQuery } from '@/redux/ipc/tracer.ipc.ts';

const CloseModelButton: React.FC = () => {
  const navigate = useNavigate();
  const [triggerSetModel, { isLoading }] = useSetModelMutation();
  const { data: renderStatus, isFetching: fetchingRenderStatus } =
    useGetRenderStatusQuery(null);
  const [errorMessage, setErrorMessage] = useState<string>('');

  const maybeRenderInProgress = useMemo(
    () => renderStatus?.renderInProgress || fetchingRenderStatus,
    [fetchingRenderStatus, renderStatus?.renderInProgress],
  );

  const onClickClose = useCallback(() => {
    navigate(ROUTES.LANDING)?.catch((error: unknown) => {
      console.log('failed to navigate:', error);
    });

    // timeout prevents rtkquery tag invalidation race
    setTimeout(() => {
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
    }, 50);
  }, [navigate, triggerSetModel]);

  return (
    <>
      <Button
        size="sm"
        disabled={isLoading || maybeRenderInProgress}
        onClick={onClickClose}
      >
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
