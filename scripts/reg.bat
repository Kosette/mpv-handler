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
    set /p operation="Enter 'i' to install or 'r' to uninstall (i/r)? Your input: "
    if [%operation%] == [""] goto :choose_operation
) else (
    set operation=i
)

if "%operation%"=="i" (
    :: Get mpv-handler.exe location
    call :check_binary

    :: Add registry
    call :add_verbs

    echo Successfully install registry
    echo Enjoy!
) else (
    :: Delete registry
    call :del_verbs

    echo Successfully uninstall registry
)

:die
    if not [%1] == [] echo %~1
    if [%unattended%] == [yes] exit 1
    pause
    exit 1

:ensure_admin
    openfiles >nul 2>&1
    if errorlevel 1 (
        echo This batch script requires administrator privileges.
        echo Right-click on reg.bat and select "Run as administrator".
        call :die
    )
    goto :EOF

:ensure_vista
    ver | find "XP" >nul
    if not errorlevel 1 (
        echo This batch script only works on Windows Vista and later.
        call :die
    )
    goto :EOF

:check_binary
    cd /D %~dp0
    set mpv_handler_path=%cd%\mpv-handler.exe
    set mpv_handler_conf=%cd%\config.toml
    if not exist "%mpv_handler_path%" call :die "mpv-handler.exe not found."
    if not exist "%mpv_handler_conf%" call :die "config.toml not found."
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
