# PRELOAD CODE

### The code in this directory is to be run in the preload context

Its important to ensure that this code remains environment agnostic, meaning do not import node utilities or electron
utilities that are mean for the main process.

There are resources that a imported from this directory into the main process, but they are stand-alone declarations so
they do not cause issues.

### Read the docs

https://www.electronjs.org/docs/latest/tutorial/ipc
