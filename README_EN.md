# mpv-handler
> Source repo: [mpv-handler](https://github.com/Kosette/mpv-handler/)

#### Usage

> Supports parameters like `mpv://play/<url_base64>`.
>
> You may need to use the Greasemonkey script [EmbytoLocalPlayer](https://github.com/bpking1/embyExternalUrl) together to enjoy it

`mpv-handler` needs to be used with the `mpv` player. If the `mpv` program is not added to the system `PATH`, you can also use the `config.toml` file to customize the path, and put it under same folder where `mpv-handler` is. The format of `config.toml` is as follows:
```toml
# Required
mpv = "/usr/local/bin/mpv"
# There are two ways to write in Windows
# mpv = "c:\\programs\\mpv.exe" or mpv = "c:/programs/mpv.exe"

# Optional, set to use a proxy to report progress, supports http\socks proxy, leave it blank if not used
proxy = ""
```

> [!IMPORTANT]  
> Now we have a `config.exe` to manage registry!

### `config` Usage

Since inserting or deleting registry requires administrator privileges, **you must run it as administrator**. Either **right-click and "Run as administrator"** or type `sudo .\config.exe [/r|/i]` in command line, provided that you have sudo utility installed.

If you run it by clicking, just follow the prompts. If you run it in a terminal, it supports two options: `/r` to uninstall the registry, and `/i` to install the registry.

Moreover, `config.exe` also auto-generate `config.toml` and insert `mpv` path, if you put `mpv.exe` in same or parent directory.

> [!CAUTION]  
> The following solutions are outdated.

~~In addition, in order to successfully call `mpv-handler`, it needs to be written to the registry. Create a new text file, save it after writing the following content:~~
```
Windows Registry Editor Version 5.00
[HKEY_CLASSES_ROOT\mpv]
"URL Protocol"=""
@="mpv"
[HKEY_CLASSES_ROOT\mpv\shell]
[HKEY_CLASSES_ROOT\mpv\shell\open]
[HKEY_CLASSES_ROOT\mpv\shell\open\command]
@="\"D:\\Programs\\mpv-handler.exe\" \"%1\""
```
~~**In which, the path on the last line should be rewritten to the path where `mpv-handler.exe` is actually stored. Note the format: `\` and `"` should be preceded by `\`.**~~

~~Change the file extension to `.reg`, double-click to install the registry, so that you can use the above `mpv://play/<url_base64>` link format in the browser to call mpv.~~

#### Description

||URL_SAFE_NO_PAD|URL_SAFE|
|---|---|---|
|`mpv://play/<url_base64>/?subfile=<url_base64>`|✅|❌|
|`mpv://play/<url_base64>`|✅|✅|

#### Acknowledgements

Inspired by [mpv-handler@akiirui](https://github.com/akiirui/mpv-handler).

### License

MIT
