@echo OFF

:: Unattended mode flag. When set, the script will not require user input.
set unattended=no
if "%1"=="/u" set unattended=yes

:: Make sure this is Windows Vista or later
call :ensure_vista

:: Make sure the script is running as admin
call :ensure_admin

:choose_operation
if [%unattended%] == [no] (
    set /p operation="按下'r'安装注册表，或者按下'r'卸载注册表 (i/r)? 请输入："
) else (
    set operation=i
)

if [%operation%] == [] goto :choose_operation

if "%operation%"=="i" (
    :: Get mpv-handler.exe location
    call :check_binary

    :: Add registry
    call :add_verbs

    echo 成功安装注册表，尽情享用吧！
) else (
    :: Delete registry
    call :del_verbs

    echo 成功卸载注册表，拜拜！
)

:die
    if not [%1] == [] echo %~1
    if [%unattended%] == [yes] exit 1
    pause
    exit 1

:ensure_admin
    openfiles >nul 2>&1
    if errorlevel 1 (
        echo 脚本需要管理员权限运行，右键选择“以管理员权限运行”.
        call :die
    )
    goto :EOF

:ensure_vista
    ver | find "XP" >nul
    if not errorlevel 1 (
        echo 脚本支持Windows Vista及之后的系统.
        call :die
    )
    goto :EOF

:check_binary
    cd /D %~dp0
    set mpv_handler_path=%cd%\mpv-handler.exe
    set mpv_handler_conf=%cd%\config.toml
    if not exist "%mpv_handler_path%" call :die "当前目录没有发现mpv-handler.exe"
    if not exist "%mpv_handler_conf%" call :die "当前目录没有发现config.toml"
    goto :EOF

:reg
    >nul reg %*
    if errorlevel 1 set error=yes
    if [%error%] == [yes] echo Error in command: reg %*
    if [%error%] == [yes] call :die
    goto :EOF

:add_verbs
    call :reg add "HKCR\mpv" /d "URL:mpv" /f
    call :reg add "HKCR\mpv" /v "URL Protocol" /f
    call :reg add "HKCR\mpv\shell\open\command" /d "\"%mpv_handler_path%\" \"%%%%1\"" /f
    goto :EOF

:del_verbs
    call :reg delete "HKCR\mpv" /f
    goto :EOF