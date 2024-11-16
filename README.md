# Rusty Rays

#### Exit Code Meanings (relevant when the program fails to initalize its logger):

* 10 - failed to open the caching directory in the users home directory
    * win: `C:\Users\<user>\AppData\Local`
    * linux: `home/<user>/.cache`
    * mac: `Users/<user>/Libary/Caches`
  
* 11 - failed to create logging directory
    * cli: `<above caching dir>/<cargo.toml 'name'>/cli/logs`
    * application: `<above caching dir>/<cargo.toml 'name'>/application/logs` 
