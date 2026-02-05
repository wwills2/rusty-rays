## RetroUI Components and utils

This directory contains the installed retro-ui component and utility code.

https://www.retroui.dev/docs

### RetroUI is built on ShadCN
- https://ui.shadcn.com/docs/installation/manual
- this repo is set up with the intent to only use RetroUI components. ShadCN base components will be installed 
to ./shadecn-components
- See components.json in the root for the configuration
- https://ui.shadcn.com/docs/components-json

### Adding (Installing) addtional retroUI components (THERE WILL BE ERRORS)
- By default retro UI tries to install components in the root of the repo - dont do that
- There is a custom install script `ui-add` in package.json to place the components in this directory
- Find the NPM install command for the component and use its name 
  - Ex. https://www.retroui.dev/docs/components/button
  - Docs install command from the above: `npx shadcn@latest add @retroui/button`
  - From the above command, the `<component>` is `button`
- run ```sh npm run ui-add <component>``` to install a new retro ui component
  - ```sh npm run ui-add button``` to install the button
- About the aforementioned errors:
  - The imports in the installed files do not explicitly import types, this will need to be addressed by importing
  types as types with the `type` keyword
  - Components that the installed components depend on may have not been added yet. These components will also need to be
  installed, and care taken to ensure the import in the requiring component is correct