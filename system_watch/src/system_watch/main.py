import platform
import shutil

import psutil


CPU_WARNING = 80
MEMORY_WARNING = 85
DISK_WARNING = 90


def get_system_info():
    return {
        "OS": f"{platform.system()} {platform.release()}",
        "Hostname": platform.node(),
        "Architecture": platform.machine(),
        "CPU Cores": psutil.cpu_count(),
    }


def get_health_info():
    cpu_usage = psutil.cpu_percent(interval=1)
    memory = psutil.virtual_memory()
    disk = shutil.disk_usage("/")

    return {
        "CPU Usage": cpu_usage,
        "Memory Usage": memory.percent,
        "Disk Usage": round(disk.used / disk.total * 100, 1),
    }


def show_report():
    system = get_system_info()
    health = get_health_info()

    print("\nSYSTEM WATCH")
    print("=" * 30)

    print("\nSYSTEM INFORMATION")
    for name, value in system.items():
        print(f"{name:<15}: {value}")

    print("\nSYSTEM HEALTH")
    for name, value in health.items():
        print(f"{name:<15}: {value}%")

    warnings = []

    if health["CPU Usage"] >= CPU_WARNING:
        warnings.append("High CPU usage")

    if health["Memory Usage"] >= MEMORY_WARNING:
        warnings.append("High memory usage")

    if health["Disk Usage"] >= DISK_WARNING:
        warnings.append("Low disk space")

    print("\nSTATUS")
    if warnings:
        for warning in warnings:
            print(f"WARNING: {warning}")
    else:
        print("HEALTHY")


def main():
    try:
        show_report()
    except KeyboardInterrupt:
        print("\nMonitoring cancelled.")
    except OSError as error:
        print(f"System monitoring failed: {error}")


if __name__ == "__main__":
    main()

