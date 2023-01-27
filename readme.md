# Transmission Torrents Recovery

If you lost your transmission's configurations, you can use this program to re-add your torrents to transmission. This program will parse torrent names and look for them in given directories, and if found, submit the download job to transmission to verify.

Use cargo to build this program.

Usage:
```
Usage: torrent_recovery [OPTIONS] <TORRENT>...

Arguments:
  <TORRENT>...  Torrent files to process

Options:
  -s, --search <DIR>...      Directories to search for downloaded files
  -k <SKIP>...               Directories to skip during search, e.g. .DS_Store
  -u <URL>                   Transmission URL, e.g.: http://example.com:9091/transmission/rpc
      --user <USER>          Username for Transmission
      --password <PASSWORD>  Password for Transmission
  -h, --help                 Print help
  -V, --version              Print version
```
