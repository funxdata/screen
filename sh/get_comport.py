import serial
import serial.tools.list_ports

#  com 端口测试

# 列出所有的com接口
ports = serial.tools.list_ports.comports()
for port in ports:
    print(port.device, port.description)


# 打开 COM3，波特率 9600
# pip install pyserial

ser = serial.Serial("COM3", 9600, timeout=1)

print("正在监听 COM3...")
while True:
    if ser.in_waiting:  # 检查缓冲区是否有数据
        data = ser.readline().decode(errors="ignore").strip()
        print("收到:", data)