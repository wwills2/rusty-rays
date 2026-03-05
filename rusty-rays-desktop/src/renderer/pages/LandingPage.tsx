import React from 'react';
import { Card, Text } from '@/retro-ui-lib';
import { ImportRayShadeFileButton } from '@/components';

const LandingPage: React.FC = () => {
  return (
    <div className="h-full w-full absolute inset-0 bg-gradient-to-br from-muted via-accent to-background animate-gradient bg-[length:300%_300%]">
      <div className="flex h-full w-full items-center justify-center">
        <Card className="max-w-1/2">
          <Card.Header>
            <Card.Title className="text-center">
              Welcome to Rusty Rays!
            </Card.Title>
            <Card.Description>
              Rusty Rays is a still-image CPU ray tracer implemented from
              scratch in rust, capable of rendering scenes defined by a subset
              of the RayShade4 file format.
            </Card.Description>
            <Card.Description className="mt-4">
              This application is a renderer and simple editor for rendering
              using the Rusty Rays ray shader.
            </Card.Description>
            <div className="flex mt-4 space-x-1">
              <Card.Description>Sample input files</Card.Description>
              <a
                href="https://github.com/wwills2/rusty-rays/tree/develop/sample-files"
                target="_blank"
              >
                <Text as="a" className="font-medium">
                  can be downloaded here.
                </Text>
              </a>
            </div>

            <Card.Content className="flex justify-center items-center">
              <ImportRayShadeFileButton label="Open a RayShade4 file (.ray)" />
            </Card.Content>
          </Card.Header>
        </Card>
      </div>
    </div>
  );
};

export { LandingPage };
