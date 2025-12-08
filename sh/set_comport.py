import serial

def write_to_com(port="COM3", baudrate=9600, message="Hello Device"):
    try:
        # 打开串口
        with serial.Serial(port, baudrate, timeout=1) as ser:
            # 写入数据（需要 bytes 类型）
            ser.write(message.encode("utf-8"))
            print(f"已发送: {message} -> {port}")
    except serial.SerialException as e:
        print(f"串口错误: {e}")

if __name__ == "__main__":
    # 修改端口、波特率和要发送的信息
    write_to_com(port="COM3", baudrate=9600, message="Test Message\r\n")