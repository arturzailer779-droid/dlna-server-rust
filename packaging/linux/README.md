# Linux packaging

## Debian package and KDE Discover

The AppStream metadata for DLNA SERVER 2.6.0 is maintained in:

`io.github.dlna_server.DlnaVideoServer2.metainfo.xml`

When building the Debian package, install this file as:

`/usr/share/metainfo/io.github.dlna_server.DlnaVideoServer2.metainfo.xml`

The metadata refers to the desktop entry already shipped by the application package:

- Desktop file: `/usr/share/applications/dlna-video-server.desktop`
- Executable: `/usr/bin/dlna-video-server`
- Icon name: `dlna-video-server`

The AppStream component ID is `io.github.dlna_server.DlnaVideoServer2`.
The metadata describes the existing application; it does not change its
source code, executable, interface, or runtime behavior.

Validate the metadata before publishing with:

```bash
appstreamcli validate packaging/linux/io.github.dlna_server.DlnaVideoServer2.metainfo.xml
```

The Debian package intended for publication must include the metadata at the
installed path above. A standalone XML file in the source repository is not
sufficient for KDE Discover to receive metadata from a Debian repository.
