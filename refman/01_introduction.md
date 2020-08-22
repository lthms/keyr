# Introduction

**keyr** (**key** **r**eporting) gathers a collection of tools to keep
track of your keystrokes. It is made of three software components:

  - `keyr-daemon` counts your keystrokes
  - `keyr-agent` keeps a detailed log of your keystrokes locally, and
    can be used to share this log remotely with an instance of
    `keyr-hub`
  - `keyr-hub` keeps a detailed log of your keystrokes count, hour
    by hour, and can be used to synchronize a shared counter among
    several computers; it is completely optional.
