@echo off
:: ==============================
:: 添加程序到开机启动
:: ==============================
:: 相对路径写法
set REL_PATH=.\screen.exe
set APP_NAME=MyApp

:: 转换为绝对路径
for %%I in (%REL_PATH%) do set EXE_PATH=%%~fI

:: 写入注册表，添加到开机启动
reg add HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v %APP_NAME% /t REG_SZ /d "%EXE_PATH%" /f

echo 已添加到开机启动：%EXE_PATH%
pause