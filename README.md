# mpv-handler

## English Ver. [README](README_EN.md)

#### 使用方法

> 支持形如`mpv://play/<url_base64>`的参数。
> 
> 你可能需要配合油猴脚本[EmbytoLocalPlayer](https://github.com/bpking1/embyExternalUrl)一起享用

`mpv-handler`需配合`mpv`播放器使用，如果`mpv`程序没有加入系统`PATH`，也可以使用`config.toml`文件自定义路径，把写好的`config.toml`文件放在和`mpv-handler`相同文件夹下面。`config.toml`格式如下：
```toml
# 必填项
mpv = "/usr/local/bin/mpv"
# Windows有两种写法
# mpv = "c:\\programs\\mpv.exe" 或者 mpv = "c:/programs/mpv.exe"

# 可选项，设置使用代理回传进度，支持http\socks代理，不使用可以留空
proxy = ""
```
> [!IMPORTANT]  
> 现在有了可以导入注册表的工具`cfg_tool.exe`，不再需要手动编写。

### `cfg_tool`使用方法如下

因为写、删注册表需要管理员权限，所以必须以管理员权限运行。直接右键点击“以管理员身份运行”或者在命令行中输入`sudo .\cfg_tool.exe [/r|/i]`，前提是你安装了sudo工具。

如果是点击运行，按照提示操作即可。如果是终端运行，支持两个选项：`/r`卸载注册表，`/i`安装注册表。

> [!CAUTION]  
> 下面的方案已经过时。

~~此外，为了成功调用`mpv-handler`，需要将其写入注册表，新建文本文件，在其中写入以下内容后保存：~~
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
~~**其中，最后一行的路径改写成实际存放mpv-handler.exe的路径，注意格式：`\`和`"`前面要加上`\`。**~~

~~将文件后缀改成`.reg`，双击安装注册表，这样就可以在浏览器中使用上述的`mpv://play/<url_base64>`链接形式调用mpv了。~~

#### 说明

||URL_SAFE_NO_PAD|URL_SAFE|
|---|---|---|
|`mpv://play/<url_base64>/?subfile=<url_base64>`|✅|❌|
|`mpv://play/<url_base64>`|✅|✅|

#### 致谢

由[mpv-handler@akiirui](https://github.com/akiirui/mpv-handler)启发。

### License

MIT
