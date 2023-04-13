# Adjust for new machines


def detail_get_cpu_info(name):
    name_lower = name.lower()

    if "zen3" in name_lower:
        cpu_boost_ghz = 4.9
        cpu_arch = "Zen3"
        os_name = "Linux"
    elif "firestorm" in name_lower:
        cpu_boost_ghz = 3.2
        cpu_arch = "Firestorm"
        os_name = "MacOS"
    elif "haswell" in name_lower:
        cpu_boost_ghz = 3.0
        cpu_arch = "Haswell"
        os_name = "Linux"
    elif "skylake" in name_lower:
        cpu_boost_ghz = 4.8
        cpu_arch = "Skylake"
        os_name = "Windows"

    return cpu_arch, cpu_boost_ghz, os_name


def get_cpu_info(name):
    cpu_arch, cpu_boost_ghz, os_name = detail_get_cpu_info(name)

    return f"{cpu_arch} max {cpu_boost_ghz}GHz | {os_name}"
