#!/bin/bash
# ============================================
# Kubuntu KDE - 自动添加开机启动项脚本
# 功能类似 Windows 注册表 Run
# ============================================

APP_NAME="funxscreen"
REL_PATH="./screen"   # 如果是 Wine 程序用 ./screen.exe

# 解析绝对路径和目录
EXE_PATH="$(readlink -f "$REL_PATH")"
EXE_DIR="$(dirname "$EXE_PATH")"

# 检查文件是否存在
if [[ ! -f "$EXE_PATH" ]]; then
    echo "❌ 错误：未找到程序文件：$EXE_PATH"
    exit 1
fi

echo "ℹ️ 程序信息："
echo "  程序路径：$EXE_PATH"
echo "  工作目录：$EXE_DIR"
echo

# KDE autostart 目录
AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$AUTOSTART_DIR"

# Desktop 文件路径
DESKTOP_FILE="$AUTOSTART_DIR/$APP_NAME.desktop"

echo "ℹ️ 正在创建开机启动项：$DESKTOP_FILE"

# 自动生成 KDE 自启动项
cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Exec=bash -c "cd '$EXE_DIR' && '$EXE_PATH'"
X-GNOME-Autostart-enabled=true
EOF

echo "ℹ️ 验证 Desktop 文件..."

if [[ -f "$DESKTOP_FILE" ]]; then
    echo "✅ 自启动创建成功！"
    echo "文件位置：$DESKTOP_FILE"
    echo
    cat "$DESKTOP_FILE"
else
    echo "❌ 自启动创建失败！"
fi

echo
echo "完成!"