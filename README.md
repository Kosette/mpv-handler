### mpv-handler

#### 说明

在不支持`mpv://`链接的某些应用里（比如Alist），劫持VLC的链接，通过`vlc://http(s)....`链接调用mpv播放器

#### 注意⚠️

在firefox中链接形式为`vlc://https//example.com/a.mp4`，而其他浏览器可能为`vlc://https://example.com/a.mp4`，脚本同时兼容了两种情况。为避免奇怪情况，链接中务必包含网络协议`https`或者`http`。
