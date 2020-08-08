#include <errno.h>
#include <fcntl.h>
#include <libinput.h>
#include <libudev.h>
#include <poll.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/stat.h>
#include <unistd.h>

#define UNIX_PATH_MAX 108

typedef uint32_t muu_count;

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
                                muu_count *state
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
      break;
    default:
      break;
    }
  }
}

int muu_poll_libinput_events (struct libinput *li, muu_count *state) {
  if (libinput_dispatch (li) != 0) {
    goto exit;
  }

  struct libinput_event *event = NULL;

  while ((event = libinput_get_event (li))) {
    muu_libinput_event_handle (event, state);
    libinput_event_destroy (event);
  }

  return 0;

 exit:
  return -1;
}

#define MUU_SOCKET_PATH "/tmp/mud.socket"

int main (void) {
  muu_count state = 0;
  int ret = 0;
  struct libinput *li = NULL;
  int server_socket = -1;

  // file creation rights, we make sure that anyone can connect to the socket
  umask (0111);

  // unix socket
  unlink (MUU_SOCKET_PATH);

  server_socket = socket (AF_UNIX, SOCK_STREAM, 0);

  if (server_socket == -1) {
    goto exit;
  }

  struct sockaddr_un server_sockaddr =
    { .sun_family = AF_UNIX, .sun_path = MUU_SOCKET_PATH };

  int len = sizeof(server_sockaddr);

  if (bind (server_socket, (struct sockaddr *)&server_sockaddr, len) == -1) {
    ret = 1;
    goto exit;
  }

  if (listen (server_socket, 10) == -1) {
    ret = 2;
    goto exit;
  }

  // libinput struct
  li = muu_libinput_create ();

  if (!li) {
    ret = 3;
    goto exit;
  }

  // pollfd structures
  struct pollfd pollfds[] = {
    { .fd = libinput_get_fd (li), .events = POLLIN },
    { .fd = server_socket, .events = POLLIN },
  };

  // event loop
  while (1) {
    if (poll(pollfds, sizeof (pollfds) / sizeof (pollfds[0]), -1) < 0) {
      ret = 4;
      goto exit;
    }

    // libinput case
    if ((pollfds[0].revents & POLLIN)) {
      if (muu_poll_libinput_events (li, &state) != 0) {
        ret = 5;
        goto exit;
      }
    }

    // socket case
    if ((pollfds[1].revents & POLLIN)) {
      struct sockaddr_un client_sockaddr;
      int client_len = sizeof (client_sockaddr);

      int client_socket =
        accept(server_socket,
               (struct sockaddr *)&client_sockaddr,
               (socklen_t *)&client_len);

      if (client_socket == -1) {
        break;
      }

      if (send (client_socket, &state, sizeof (state), 0) == sizeof (state)) {
        // keypressed delta has been sent, so we reset the counter
        state = 0;
      }

      close (client_socket);
    }
  }

 exit:
  unlink (MUU_SOCKET_PATH);
  close (server_socket);
  libinput_unref (li);
  return ret;
}
