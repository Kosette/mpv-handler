# mpv-handler

> 仓库地址： [mpv-handler](https://github.com/Kosette/mpv-handler/)

## English Ver. [README](README_EN.md)

> [!TIP]
> 本项目支持 emby 调用 mpv 后回传进度，回传频率为 1 次/10s

> [!WARNING]
> 没有在 Linux 平台测试回传功能

#### 使用方法

> 支持形如`mpv://play/<url_base64>`的参数。
>
> 你可能需要配合油猴脚本[EmbytoLocalPlayer](https://github.com/bpking1/embyExternalUrl)一起享用

> [!IMPORTANT]  
> 使用 GUI 工具`handler-config.exe`可以较为方便的配置 mpv-handler.toml 和生成所需要的注册表。

`mpv-handler`需配合`mpv`播放器使用，如果`mpv`程序没有加入系统环境变量`PATH`，可以使用`mpv-handler.toml`文件自定义路径，把写好的`mpv-handler.toml`文件放在和`mpv-handler`相同文件夹下面。`mpv-handler.toml`格式如下：

```toml
# 必填项
mpv = "/usr/local/bin/mpv"
# Windows有两种写法
# mpv = "c:\\programs\\mpv.exe" 或者 mpv = "c:/programs/mpv.exe"

# 可选项，设置使用代理回传进度，支持http代理，不使用可以留空
proxy = ""
```

> [!IMPORTANT]
> 如果您不知道怎么手动处理注册表，请使用 handler-config.exe

~~`mpv-handler`需要写入相关注册表项后才能成功调用，可以将以下格式的内容写入空白 txt 文本文件，将其后缀修改为 reg，双击导入注册表。~~

~~**注意 ⚠️，最后一行的路径修改为本机实际存放 mpv-handler.exe 的路径，注意格式：`\`和`"`前面要加上`\`。**~~

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

#### 说明

|                                                 | URL_SAFE_NO_PAD | URL_SAFE |
| ----------------------------------------------- | --------------- | -------- |
| `mpv://play/<url_base64>/?subfile=<url_base64>` | ✅              | ❌       |
| `mpv://play/<url_base64>`                       | ✅              | ✅       |

#### 致谢

由[mpv-handler@akiirui](https://github.com/akiirui/mpv-handler)启发。

### License

MIT
