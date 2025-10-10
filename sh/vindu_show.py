import ctypes
from ctypes import wintypes

SW_SHOW = 5

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

def show_window(hwnd):
    if hwnd:
        ShowWindow(hwnd, SW_SHOW)

def main():
    # 恢复任务栏
    taskbar_hwnd = FindWindowA(b"Shell_TrayWnd", None)
    show_window(taskbar_hwnd)

    # 恢复桌面图标
    progman_hwnd = FindWindowA(b"Progman", None)
    desktop_hwnd = FindWindowExA(progman_hwnd, None, b"SHELLDLL_DefView", None)
    show_window(desktop_hwnd)

    print("任务栏和桌面图标已恢复。")

if __name__ == "__main__":
    main()
