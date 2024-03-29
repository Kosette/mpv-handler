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
    set /p operation="����'r'��װע������߰���'r'ж��ע��� (i/r)? �����룺"
) else (
    set operation=i
)

if [%operation%] == [] goto :choose_operation

if "%operation%"=="i" (
    :: Get mpv-handler.exe location
    call :check_binary

    :: Add registry
    call :add_verbs

    echo �ɹ���װע����������ðɣ�
) else (
    :: Delete registry
    call :del_verbs

    echo �ɹ�ж��ע����ݰݣ�
)

:die
    if not [%1] == [] echo %~1
    if [%unattended%] == [yes] exit 1
    pause
    exit 1

:ensure_admin
    openfiles >nul 2>&1
    if errorlevel 1 (
        echo �ű���Ҫ����ԱȨ�����У��Ҽ�ѡ���Թ���ԱȨ�����С�.
        call :die
    )
    goto :EOF

:ensure_vista
    ver | find "XP" >nul
    if not errorlevel 1 (
        echo �ű�֧��Windows Vista��֮���ϵͳ.
        call :die
    )
    goto :EOF

:check_binary
    cd /D %~dp0
    set mpv_handler_path=%cd%\mpv-handler.exe
    set mpv_handler_conf=%cd%\config.toml
    if not exist "%mpv_handler_path%" call :die "��ǰĿ¼û�з���mpv-handler.exe"
    if not exist "%mpv_handler_conf%" call :die "��ǰĿ¼û�з���config.toml"
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