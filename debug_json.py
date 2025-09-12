#!/usr/bin/env python3
"""Debug JSON structure"""

import json

with open('output/um_BTCUSDT_rangebar_20250901_20250901_0.800pct.json', 'r') as f:
    data = json.load(f)

print("Top-level keys:")
for key in data.keys():
    print(f"  {key}")

print("\nLooking for statistics...")
if 'statistics' in data:
    print("Statistics keys:")
    for key in data['statistics'].keys():
        print(f"  {key}")
else:
    print("No 'statistics' key found")
    
# Let's find total_trades
print("\nSearching for total_trades...")
def find_key(obj, key, path=""):
    if isinstance(obj, dict):
        for k, v in obj.items():
            new_path = f"{path}.{k}" if path else k
            if k == key:
                print(f"Found {key} at: {new_path} = {v}")
            if isinstance(v, (dict, list)):
                find_key(v, key, new_path)
    elif isinstance(obj, list):
        for i, item in enumerate(obj):
            new_path = f"{path}[{i}]"
            if isinstance(item, (dict, list)):
                find_key(item, key, new_path)

find_key(data, "total_trades")