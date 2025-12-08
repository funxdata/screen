@echo off
chcp 65001 >nul
:: ==============================
:: 确保程序添加到开机启动项并在正确目录运行
:: ==============================

setlocal enabledelayedexpansion

:: 配置
set "APP_NAME=funxscreen"
set "REL_PATH=.\screen.exe"

:: 获取程序绝对路径和目录
for %%I in ("%REL_PATH%") do (
    set "EXE_PATH=%%~fI"
    set "EXE_DIR=%%~dpI"
)

:: 检查程序是否存在
if not exist "!EXE_PATH!" (
    echo ❌ 错误：未找到程序文件 "!EXE_PATH!"
    pause
    exit /b 1
)

echo ℹ️ 检测到程序信息：
echo   程序路径：!EXE_PATH!
echo   工作目录：!EXE_DIR!
echo.

:: 构造启动命令 - 确保在正确目录运行
set "START_CMD=cmd /c "cd /d "!EXE_DIR!" && start "" "!EXE_PATH!""

:: 先删除可能存在的旧启动项
echo ℹ️ 正在清理旧启动项...
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "!APP_NAME!" /f >nul 2>&1

:: 添加新的启动项
echo ℹ️ 正在添加新启动项...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "!APP_NAME!" /t REG_SZ /d "!START_CMD!" /f >nul 2>&1

:: 验证添加结果
echo ℹ️ 正在验证启动项...
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "!APP_NAME!" >nul 2>&1

if !errorlevel! equ 0 (
    echo.
    echo ✅ 启动项添加成功！
    echo   在注册表中的值：
    reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "!APP_NAME!"
) else (
    echo.
    echo ❌ 启动项添加失败！
    echo   请尝试以下解决方法：
    echo   1. 右键点击本脚本，选择"以管理员身份运行"
    echo   2. 检查用户账户控制(UAC)设置是否过高
    echo   3. 手动添加：运行regedit，导航到
    echo      HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
    echo      新建字符串值，名称：!APP_NAME!
    echo      数值数据：!START_CMD!
)

echo.
pause
endlocal