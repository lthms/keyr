# keyr

## Overview

A collection of tools to keep track of your keystrokes (keyr stands
for **key**strokes **r**eporting).

  - `keyr-daemon` counts your keystrokes
  - `keyr-hub` allows for synchronizing your keystrokes count among
    several computers
  - `keyr-agent` maintains a detailed log of your keystrokes statistics locally,
    and can communicate `keyr-hub`

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
Description=keyr-daemon
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/keyr-daemon

[Install]
WantedBy=sway-session.target
```

Prior to starting this service, `systemctl --user import-environment`
shall be run.

`keyrd` does only one thing: it counts. It does not deal with
persistence. This part is achieved by `keyr-agent`.

You need to execute `keyr-agent` regularly. If you use
[waybar](https://github.com/Alexays/Waybar), you can use the `custom`
module to that end. In this case, `keyr-agent` can be used to print a
text in the bar.

```json
{
    ...
    "custom/keyr": {
        "exec": "keyr-agent stage; keyr-agent format --template '{today_count | num_format} today ({global_count | num_format} total)'",
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
