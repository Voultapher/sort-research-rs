# Adjust for new machines


def get_cpu_info(name):
    name_lower = name.lower()

    if "zen3" in name_lower:
        cpu_boost_ghz = 4.9
        cpu_arch = "Zen3"
    elif "firestorm" in name_lower:
        cpu_boost_ghz = 3.2
        cpu_arch = "Firestorm"
    elif "haswell" in name_lower:
        cpu_boost_ghz = 3.0
        cpu_arch = "Haswell"
    elif "skylake" in name_lower:
        cpu_boost_ghz = 4.8
        cpu_arch = "skylake"

    return cpu_boost_ghz, cpu_arch
