# Rusty Rays

A simple primary-bounce raytracer

#### Logging

* The application will create a log folder in the users cache directory
    * win: `C:\Users\<user>\AppData\Local\rusty-rays\logs\`
    * linux: `$HOME/.cache/rusty-rays/logs/`
    * mac: `$HOME/Libary/Caches/rusty-rays/logs/`
* The logger will default to console only logging if unable to create the log folder or file

#### Config

* The application will create a `config.json5` file in the users config directory
    * win: `C:\Users\<user>\AppData\Roaming\rusty-rays\`
    * linux: `$HOME/.config/rusty-rays/`
    * mac: `$HOME/Library/Application Support/rusty-rays/`
* If `config.json5` is inaccessible, the application will use its internal default config
