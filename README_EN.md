# mpv-handler

> Source repo: [mpv-handler](https://github.com/Kosette/mpv-handler/)

> [!TIP]
> Report playback status to emby server every 10s

> [!WARNING]
> Not tested on Linux

#### Usage

> Supports parameters like `mpv://play/<url_base64>`.
>
> You may need to use the Greasemonkey script [EmbytoLocalPlayer](https://github.com/bpking1/embyExternalUrl) together to enjoy it

> [!IMPORTANT]  
> Use handler-config.exe to generate reg file and toml config file

`mpv-handler` needs to be used with the `mpv` player. If the `mpv` program is not added to the system `PATH`, you can also use the `mpv-handler.toml` file to customize the path, and put it under same folder where `mpv-handler` is. The format of `mpv-handler.toml` is as follows:

```toml
# Required
mpv = "/usr/local/bin/mpv"
# There are two ways to write in Windows
# mpv = "c:\\programs\\mpv.exe" or mpv = "c:/programs/mpv.exe"

# Optional, set to use a proxy to report progress, supports http proxy, leave it blank if not used
proxy = ""
```

> [!IMPORTANT]
> If you don't know how to mannually write registry, use handler-config.exe

~~`mpv-handler` needs to add registry to work, the format as follows:~~

```
Windows Registry Editor Version 5.00
[HKEY_CLASSES_ROOT\mpv]
"URL Protocol"=""
@="mpv"
[HKEY_CLASSES_ROOT\mpv\shell]
[HKEY_CLASSES_ROOT\mpv\shell\open]
[HKEY_CLASSES_ROOT\mpv\shell\open\command]
@="\"C:\\Programs\\mpv-handler.exe\" \"%1\""
```

~~**In which, the path on the last line should be rewritten to the path where `mpv-handler.exe` is actually stored. Note the format: `\` and `"` should be preceded by `\`.**~~

#### Description

|                                                 | URL_SAFE_NO_PAD | URL_SAFE |
| ----------------------------------------------- | --------------- | -------- |
| `mpv://play/<url_base64>/?subfile=<url_base64>` | ✅              | ❌       |
| `mpv://play/<url_base64>`                       | ✅              | ✅       |

#### Acknowledgements

Inspired by [mpv-handler@akiirui](https://github.com/akiirui/mpv-handler).

### License

MIT
