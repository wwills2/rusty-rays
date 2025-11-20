# PRELOAD ONLY CODE
### The code in this directory should be run from the preload.ts file ONLY

The code contained in this directory is for exposing the electron interprocess communication handles to the renderer
process via the electron preload script.

### This directory should be ENTIRELY self-contained
- Only import types 
- Do not import code from outside this module into it
- Do not import this code into main process modules
- Do not import this module into renderer process modules

### Read the docs
https://www.electronjs.org/docs/latest/tutorial/ipc
