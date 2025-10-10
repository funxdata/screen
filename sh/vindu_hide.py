import ctypes
from ctypes import wintypes

SW_HIDE = 0

user32 = ctypes.WinDLL("user32", use_last_error=True)

FindWindowA = user32.FindWindowA
FindWindowA.argtypes = [wintypes.LPCSTR, wintypes.LPCSTR]
FindWindowA.restype = wintypes.HWND

FindWindowExA = user32.FindWindowExA
FindWindowExA.argtypes = [wintypes.HWND, wintypes.HWND, wintypes.LPCSTR, wintypes.LPCSTR]
FindWindowExA.restype = wintypes.HWND

ShowWindow = user32.ShowWindow
ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
ShowWindow.restype = wintypes.BOOL

def hide_window(hwnd):
    if hwnd:
        ShowWindow(hwnd, SW_HIDE)

def main():
    # 隐藏任务栏
    taskbar_hwnd = FindWindowA(b"Shell_TrayWnd", None)
    hide_window(taskbar_hwnd)

    # 隐藏桌面图标
    progman_hwnd = FindWindowA(b"Progman", None)
    desktop_hwnd = FindWindowExA(progman_hwnd, None, b"SHELLDLL_DefView", None)
    hide_window(desktop_hwnd)

    print("任务栏和桌面图标已隐藏。")

if __name__ == "__main__":
    main()
