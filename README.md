# keyr

## Overview

A collection of tools to keep track of your keystrokes (keyr stands
for **key**strokes **r**eporting).

  - `keyrd` (keyr daemon) counts your keystrokes
  - `keyr-sync` maintains a detailed log of your keystrokes statistics
  - `keyr-fmt` outputs said log to the standard output (json)

## Getting Started

### Building From Source

You will need the following programs to build keyr binaries.

  - make
  - meson and ninja
  - rustc and cargo

Besides, keyrd requires the following runtime dependencies:

  - udev
  - libinput

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

Prior to starting this service, `systemctl --user import-environment`
shall be run.

`keyrd` does only one thing: it counts. It does not deal with
persistence. This part is achieved by `keyr-sync`.

You need to execute `keyr-sync` regularly. If you use
[waybar](https://github.com/Alexays/Waybar), you can use the `custom`
module to that end. In this case, `keyr-fmt` can be used to print a
text in the bar.

```json
{
    ...
    "custom/keyr": {
        "exec": "keyr-sync; keyr-fmt --minimal --template '{today_count | num_format} today ({global_count | num_format} total)'",
        "format" : "{} ⌨",
        "interval" : 5
    },
    ...
}
```

## Credits

keyrd could not have been written without [the source code of
`wshowkeys`](https://git.sr.ht/~sircmpwn/wshowkeys), released under
the terms of the GPLv3 by [Drew DeVault](https://drewdevault.com/).
