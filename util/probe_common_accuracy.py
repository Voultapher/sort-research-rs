import sys
import re
import statistics


if __name__ == "__main__":
    with open(sys.argv[1], "r") as file:
        input = file.read()

    yes_count = input.count("yes")
    no_count = input.count("no")
    total = yes_count + no_count
    found_yes = round((yes_count / total) * 100.0, 2)

    # average_count = round(
    #     statistics.mean(
    #         [int(val) for val in re.findall(r"count: (\d+)", input)]
    #     ),
    #     2,
    # )
    average_count = 0

    print(f"Found: {found_yes}% Average count: {average_count}")
