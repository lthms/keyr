# `muu`

- `mud` (`muu` daemon) counts the keypress events; the counter can be
  retrieved by connecting to `/tmp/muu.socket` UNIX socket
- `muf` (`muu` fetcher) connects to the socket of `mud`, updates a
  persistent counter (`${XDG_CONFIG_HOME}/muu/counter`), and prints
  the total number of keypress events intercepted by `mud`
