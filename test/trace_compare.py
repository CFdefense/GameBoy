import re
import argparse

def extract_tick_and_line(line, original_line_num):
    """Extracts the 8-digit hex tick value from a line, or None if not found.
       Handles two formats: 'XXXXXXXX - ...' and 'Ticks:XXXXXXXX A:...' 
       Returns a tuple: (tick_value, original_line_content, original_line_num).
    """
    # Regex to capture ticks from both formats
    # Format 1: 00000004 - ...  (captures 00000004)
    # Format 2: Ticks:00000004 A:... (captures 00000004)
    match = re.match(r'(?:^|Ticks:)([0-9A-Fa-f]{8})(?: -| A:)', line)
    if match:
        # The tick value is always in group 1 due to the non-capturing group for the prefix
        return (match.group(1), line, original_line_num)
    return None

def get_tick_bearing_lines(lines_with_numbers):
    """Filters a list of (line_content, original_line_num) tuples to keep only those with ticks."""
    processed_lines = []
    for i, line_content in enumerate(lines_with_numbers):
        extracted = extract_tick_and_line(line_content, i) # i is the original 0-indexed line number
        if extracted:
            processed_lines.append(extracted) # (tick_value, original_line_content, original_line_num)
    return processed_lines

def main():
    parser = argparse.ArgumentParser(description="Compare two trace files and find the first tick divergence.")
    parser.add_argument("file1", help="Path to the first trace file.")
    parser.add_argument("file2", help="Path to the second trace file.")
    args = parser.parse_args()

    try:
        with open(args.file1, 'r') as f1, open(args.file2, 'r') as f2:
            original_lines1 = f1.readlines()
            original_lines2 = f2.readlines()
    except FileNotFoundError as e:
        print(f"Error: {e}")
        return

    # Get only lines that have ticks, along with their original content and line numbers
    tick_lines1 = get_tick_bearing_lines(original_lines1)
    tick_lines2 = get_tick_bearing_lines(original_lines2)

    divergence_point_orig_idx1 = -1
    divergence_point_orig_idx2 = -1
    found_divergence = False

    min_tick_list_len = min(len(tick_lines1), len(tick_lines2))

    for i in range(min_tick_list_len):
        tick1_val, line1_content, orig_idx1 = tick_lines1[i]
        tick2_val, line2_content, orig_idx2 = tick_lines2[i]

        if tick1_val != tick2_val:
            divergence_point_orig_idx1 = orig_idx1
            divergence_point_orig_idx2 = orig_idx2
            found_divergence = True
            print(f"Divergence detected at content lines:")
            print(f"  File 1 (original line {orig_idx1 + 1}), Tick: {tick1_val} - Line: {line1_content.strip()}")
            print(f"  File 2 (original line {orig_idx2 + 1}), Tick: {tick2_val} - Line: {line2_content.strip()}")
            break
    
    if not found_divergence:
        if len(tick_lines1) != len(tick_lines2):
            print("No tick divergence found in common tick-bearing lines, but files have different numbers of tick-bearing lines.")
            # Point of divergence is the first non-matching line in terms of existence
            if len(tick_lines1) > min_tick_list_len:
                _, _, divergence_point_orig_idx1 = tick_lines1[min_tick_list_len]
                # No corresponding line in file2, so idx2 remains -1 or we can use last known good from file2
                if tick_lines2: # if file2 was not empty of ticks
                     _, _, divergence_point_orig_idx2 = tick_lines2[-1] 
                else: # file2 was empty of ticks
                    divergence_point_orig_idx2 = 0 # Default to beginning if file2 had no ticks
                print(f"  File 1 has more tick lines. Divergence after original line {divergence_point_orig_idx2 + 1} of File 2.")
                print(f"  Next tick line in File 1 is original line {divergence_point_orig_idx1 + 1}: {tick_lines1[min_tick_list_len][1].strip()}")
            else:
                _, _, divergence_point_orig_idx2 = tick_lines2[min_tick_list_len]
                if tick_lines1:
                    _, _, divergence_point_orig_idx1 = tick_lines1[-1]
                else:
                    divergence_point_orig_idx1 = 0
                print(f"  File 2 has more tick lines. Divergence after original line {divergence_point_orig_idx1 + 1} of File 1.")
                print(f"  Next tick line in File 2 is original line {divergence_point_orig_idx2 + 1}: {tick_lines2[min_tick_list_len][1].strip()}")
            found_divergence = True # Treat as found for printing context
        else:
            print("No divergence found. Ticks in both files match for all comparable tick-bearing lines.")
            return
    
    if not found_divergence: # Should only be hit if both files are empty of ticks or identical and empty
        print("No tick-bearing lines found or files are identical.")
        return

    # Use original line indices for context printing
    start_line1 = max(0, divergence_point_orig_idx1 - 5)
    end_line_f1 = min(len(original_lines1), divergence_point_orig_idx1 + 6)
    
    start_line2 = max(0, divergence_point_orig_idx2 - 5)
    end_line_f2 = min(len(original_lines2), divergence_point_orig_idx2 + 6)

    print(f"\n--- Lines from {args.file1} (around divergence at original line {divergence_point_orig_idx1 + 1}) ---")
    for i in range(start_line1, end_line_f1):
        prefix = ">> " if i == divergence_point_orig_idx1 else "   "
        print(f"{prefix}{i+1:05d}: {original_lines1[i].strip()}")

    print(f"\n--- Lines from {args.file2} (around divergence at original line {divergence_point_orig_idx2 + 1}) ---")
    for i in range(start_line2, end_line_f2):
        prefix = ">> " if i == divergence_point_orig_idx2 else "   "
        print(f"{prefix}{i+1:05d}: {original_lines2[i].strip()}")

if __name__ == "__main__":
    main()
