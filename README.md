# `muu`

## `mud`

`mud` (`muu` daemon) counts the keypress events; the counter can be
retrieved by connecting to `/tmp/muu.socket` UNIX socket

It can be used with `systemd`.

```
[Unit]
Description=mud - count your pressed keys
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/mud

[Install]
WantedBy=sway-session.target
```

## `muf`

`muf` (`muu` fetcher) connects to the socket of `mud`, updates a
persistent counter (`${XDG_CONFIG_HOME}/muu/counter`), and prints the
total number of keypress events intercepted by `mud`

It also keeps track of the keypress events per minutes per days.

It can be used with status bar such as `waybar`.

```json
{
    ...
    "custom/muf": {
        "exec": "muf",
        "format" : "{}",
        "interval" : 5
    },
    ...
}
```

## Getting Started

```bash
make
sudo make install
```
