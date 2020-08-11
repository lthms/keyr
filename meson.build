project(
  'keyr',
  'c',
  version: '0.0.0-dev',
  license: 'GPL-3.0-or-later',
  meson_version: '>=0.48.0',
  default_options: [
    'c_std=c11',
    'warning_level=3',
    'werror=true',
  ],
)

cc = meson.get_compiler('c')

add_project_arguments(cc.get_supported_arguments([
  '-Wundef',
  '-Wlogical-op',
  '-Wmissing-include-dirs',
  '-Wold-style-definition',
  '-Wpointer-arith',
  '-Winit-self',
  '-Wstrict-prototypes',
  '-Wimplicit-fallthrough=2',
  '-Wendif-labels',
  '-Wstrict-aliasing=2',
  '-Woverflow',

  '-Wno-unused-parameter',
]), language: 'c')

executable(
  'keyrd',
  files('keyrd/main.c'),
  dependencies: [
    dependency('libinput'),
    dependency('libudev'),
  ],
)