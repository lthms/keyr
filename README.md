# keyr

## Overview

A collection of tools to keep track of your keystrokes (keyr stands
for **key**strokes **r**eporting).

  - `keyrd` (keyr daemon) counts your keystrokes
  - `keyr-sync` maintains a detailed log of your keystrokes statistics
  - `keyr-fmt` outputs said log to the standard output (json)

## Getting Started

### Building From Source

In order to build keyr’s tools, you will need the following dependencies:

  - udev
  - libinput
  - rustc and cargo

You can build the project using `make`.

```bash
make
sudo make install
```

### Setting-up

`keyrd` is a daemon: it is expected to be run in the background. It is
installed with the setuid bit, which means you do not need to start it
as root.

You can manage it as a user systemd service.

```
[Unit]
Description=keyrd - the keyr daemon
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/keyrd

[Install]
WantedBy=sway-session.target
```

`keyrd` does only one thing: it counts. It does not deal with
persistence. This part is achieved by `keyr-sync`.  You need to
execute `keyr-sync` regularly. If you use
[waybar](https://github.com/Alexays/Waybar), you can use the `custom`
module to that end.

```json
{
    ...
    "custom/muf": {
        "exec": "keyr-sync",
        "format" : "{}",
        "interval" : 5
    },
    ...
}
```
