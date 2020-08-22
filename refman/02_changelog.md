# ChangeLog

## Unreleased Changes

This will be the first *alpha* release of the `keyr` project.

### `keyr-daemon`

- Use `libinput` to count keystrokes
- Create a UNIX socket (`/tmp/keyrd.socket`) to share this counter

### `keyr-agent`

- Add the `stage` command to fetch the current counter of `keyr-daemon`,
  and saves it locally in a “staging area”
- Add the `commit` command to push the staging area to a `keyr-hub` instance
- Add the `format` command to output the current keystroke counters
- Add the `revert` command to get back keystrokes statistics from a
  `keyr-hub` instance
- Use a Sqlite database as the persistent storage
- Configure the tool using a TOML configuration file

### `keyr-hub`

- Add a route to commit keystrokes statistics
- Add three routes to revert keystrokes statistics (i.e., sending it
  back to an agent)
- Add a route to fetch the keystrokes statistics of a given *visible*
  user
