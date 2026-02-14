import {
  Card,
  Tabs,
  TabsContent,
  TabsPanels,
  TabsTrigger,
  TabsTriggerList,
} from '@/retro-ui-lib';

import React from 'react';

const ModelObjectExplorerLayout: React.FC = () => {
  return (
    <Card className="flex h-full w-full">
      <Card.Content className="flex h-full w-full">
        <Tabs className="w-full">
          <TabsTriggerList className="flex-wrap">
            <TabsTrigger>General</TabsTrigger>
            <TabsTrigger>Spheres</TabsTrigger>
            <TabsTrigger>Polygons</TabsTrigger>
          </TabsTriggerList>
          <TabsPanels>
            <TabsContent>
              This is where general information about the model is displayed.
            </TabsContent>
            <TabsContent>This is where Speres are displayed</TabsContent>
            <TabsContent>This is where polygons are displayed</TabsContent>
          </TabsPanels>
        </Tabs>
      </Card.Content>
    </Card>
  );
};

export { ModelObjectExplorerLayout };
