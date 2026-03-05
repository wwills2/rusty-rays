import { CloseModelButton } from '@/components';
import { Card } from '@/retro-ui-lib';
import React from 'react';

const RenderImageActionHeaderLayout: React.FC = () => {
  return (
    <Card className="flex w-full">
      <Card.Content className="flex w-full p-2">
        <CloseModelButton />
      </Card.Content>
    </Card>
  );
};

export { RenderImageActionHeaderLayout };
