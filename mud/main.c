#include <errno.h>
#include <fcntl.h>
#include <libinput.h>
#include <libudev.h>
#include <stdio.h>
#include <unistd.h>

static int open_restricted (const char *path, int flags, void *data) {
	int fd = open (path, flags);
	return fd < 0 ? -errno : fd;
}

static void close_restricted (int fd, void *data) {
	close (fd);
}

const struct libinput_interface INTERFACE = {
  .open_restricted = open_restricted,
  .close_restricted = close_restricted,
};

// Creates a new `struct libinput', using the udev backend.
struct libinput *muu_libinput_create () {
  struct udev *udev = NULL;
  struct libinput *li = NULL;

  udev = udev_new ();

  if (!udev) {
    goto exit;
  }

  li = libinput_udev_create_context (&INTERFACE, NULL, udev);

  if (!li) {
    goto exit;
  }

  // We will not have to manipulate `udev' manually, and therefore we decrement
  // its reference counter.
  udev_unref (udev);

  if (libinput_udev_assign_seat (li, "seat0") != 0) {
    goto exit;
  }

  return li;

 exit:
  libinput_unref (li);
  udev_unref (udev);
  return NULL;
}

void muu_libinput_event_handle (struct libinput_event *lev,
                                int *state
                                ) {
  enum libinput_event_type event_type = libinput_event_get_type (lev);

  if (event_type == LIBINPUT_EVENT_KEYBOARD_KEY) {
    struct libinput_event_keyboard *kb =
      libinput_event_get_keyboard_event (lev);

    enum libinput_key_state key_state =
      libinput_event_keyboard_get_key_state (kb);

    switch (key_state) {
    case LIBINPUT_KEY_STATE_PRESSED:
      *state += 1;
      printf ("-");
      fflush (stdout);
      break;
    default:
      break;
    }
  }
}

int main () {
  int state = 0;
  int ret = 0;
  struct libinput *li = muu_libinput_create ();

  if (!li) {
    ret = 1;
    goto exit;
  }

  while (1) {
    struct libinput_event *event = NULL;

    if (libinput_dispatch (li) != 0) {
      ret = 2;
      goto exit;
    }

    while ((event = libinput_get_event (li))) {
      muu_libinput_event_handle (event, &state);
      libinput_event_destroy (event);
    }
  }

 exit:
  libinput_unref (li);
  return ret;
}
