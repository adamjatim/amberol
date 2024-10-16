# SPDX-FileCopyrightText: 2022  Emmanuele Bassi
# SPDX-License-Identifier: GPL-3.0-or-later

desktop_data = configuration_data()
desktop_data.set('APPLICATION_ID', application_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'io.bassi.Amberol.desktop.in.in',
    output: 'io.bassi.Amberol.desktop.in',
    configuration: desktop_data,
  ),
  output: '@0@.desktop'.format(application_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file',
    desktop_utils,
    args: [desktop_file],
    suite: ['lint'],
  )
endif

appstream_data = configuration_data()
appstream_data.set('APPLICATION_ID', application_id)
appstream_file = i18n.merge_file(
  input: configure_file(
    input: 'io.bassi.Amberol.metainfo.xml.in.in',
    output: 'io.bassi.Amberol.metainfo.xml.in',
    configuration: appstream_data,
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
 test('Validate appstream file',
   appstreamcli,
   args: ['validate', '--no-net', '--explain', appstream_file],
   suite: ['lint'],
 )
endif

install_data('io.bassi.Amberol.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0/schemas',
)

# Compile schemas locally, so we can run uninstalled under a devenv
gnome.compile_schemas()

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()],
    suite: ['lint'],
  )
endif

service_conf = configuration_data()
service_conf.set('application_id', application_id)
service_conf.set('bindir', get_option('prefix') / get_option('bindir'))
configure_file(
  input: 'io.bassi.Amberol.service.in',
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install: true,
  install_dir: get_option('datadir') / 'dbus-1/services',
)

subdir('icons')
