import sys
import os
import csv

input_file = "/etc/services"
output_file = "services_clean.csv"

if not os.path.exists(input_file):
    raise FileNotFoundError

with open(input_file, "r") as f_in, open(output_file, "w", newline="") as f_out:
    writer = csv.writer(f_out)
    writer.writerow(["service", "port", "protocol", "comment"])

    for line in f_in:
        line = line.strip()
        if not line or line.startswith("#"):
            continue

        if "#" in line:
            parts, comment = line.split("#", 1)
            comment = comment.strip()
        else:
            parts = line
            comment = ""

        tokens = parts.split()
        if len(tokens) < 2:
            continue

        service = tokens[0]
        port_proto = tokens[1]
        port, protocol = port_proto.split("/")

        writer.writerow([service, port, protocol, comment])

sys.exit()
