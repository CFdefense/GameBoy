import re
import argparse

# Regex to capture the core CPU state parts of a log line.
# It captures: Tick, PC, Opcode (first byte in parens), A, F-flags, BC, DE, HL, IE, IF.
LOG_LINE_REGEX = re.compile(
    r"(?:^|Ticks:)(?P<tick>[0-9A-Fa-f]{8})\s*-\s*" # Tick
    r"(?P<pc>[0-9A-Fa-f]{4}):\s*"                 # PC
    r".*?"                                       # Instruction name (non-greedy)
    r"\((?P<opcode>[0-9A-Fa-f]{2})"              # Opcode (first byte in parens)
    r"(?:\s+[0-9A-Fa-f]{2})*\)\s*"               # Optional other bytes in parens
    r"A:(?P<a>[0-9A-Fa-f]{2})\s*"                  # Register A
    r"F:(?P<f_z>[-Z])(?P<f_n>[-N])(?P<f_h>[-H])(?P<f_c>[-C])\s*" # Flags
    r"BC:(?P<bc>[0-9A-Fa-f]{4})\s*"               # Register BC
    r"DE:(?P<de>[0-9A-Fa-f]{4})\s*"               # Register DE
    r"HL:(?P<hl>[0-9A-Fa-f]{4})"                 # Register HL
    # Optional IE and IF, as they might not always be present or in the same place in all log formats
    r"(?:\s*IE:(?P<ie>[0-9A-Fa-f]{2}))?"
    r"(?:\s*IF:(?P<if>[0-9A-Fa-f]{2}))?"
    # Optional trailing tick value
    r"(?:\s*[0-9A-Fa-f]{8})?$" # End of line anchor
)

def extract_log_data(line_content, original_line_num):
    match = LOG_LINE_REGEX.match(line_content.strip())
    if match:
        data = match.groupdict()
        # Combine F flags for easier comparison if needed, though direct compare is fine
        # data['f_flags'] = f"{data['f_z']}{data['f_n']}{data['f_h']}{data['f_c']}"
        data['original_line_num'] = original_line_num
        data['original_line_content'] = line_content.strip()
        return data
    return None

def get_log_data_lines(original_lines):
    processed_lines = []
    for i, line_content in enumerate(original_lines):
        extracted = extract_log_data(line_content, i)
        if extracted:
            processed_lines.append(extracted)
    return processed_lines

def main():
    parser = argparse.ArgumentParser(description="Compare CPU state in two trace files.")
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

    log_entries1 = get_log_data_lines(original_lines1)
    log_entries2 = get_log_data_lines(original_lines2)

    if not log_entries1:
        print(f"Could not parse any valid log entries from {args.file1}")
    if not log_entries2:
        print(f"Could not parse any valid log entries from {args.file2}")
    if not log_entries1 or not log_entries2:
        return

    divergence_point_orig_idx1 = -1
    divergence_point_orig_idx2 = -1
    found_divergence = False

    min_entries_len = min(len(log_entries1), len(log_entries2))

    fields_to_compare = ['pc', 'opcode', 'a', 'f_z', 'f_n', 'f_h', 'f_c', 'bc', 'de', 'hl']
    # IE and IF are compared separately if primary fields match
    special_hl_opcodes = {'2A', '3A', '22', '32'} # Opcodes for (HL+), (HL-)

    for i in range(min_entries_len):
        entry1 = log_entries1[i]
        entry2 = log_entries2[i]

        if entry1['tick'] != entry2['tick']:
            found_divergence = True
            print(f"Divergence detected due to differing Ticks:")
            print(f"  File 1 (line {entry1['original_line_num'] + 1}): Tick {entry1['tick']} - {entry1['original_line_content']}")
            print(f"  File 2 (line {entry2['original_line_num'] + 1}): Tick {entry2['tick']} - {entry2['original_line_content']}")
            divergence_point_orig_idx1 = entry1['original_line_num']
            divergence_point_orig_idx2 = entry2['original_line_num']
            break

        for field in fields_to_compare:
            val1 = entry1[field]
            val2 = entry2[field]

            if field == 'hl' and entry1.get('opcode') in special_hl_opcodes and entry2.get('opcode') in special_hl_opcodes:
                # Ensure opcodes match for this special handling to be valid (though opcode itself is checked earlier)
                if entry1['opcode'] == entry2['opcode']:
                    try:
                        hl1_val = int(val1, 16)
                        hl2_val = int(val2, 16)
                        if abs(hl1_val - hl2_val) <= 1: # Allow difference of 0 or 1
                            continue # HL matches under special condition
                    except ValueError:
                        # If conversion fails, fall through to direct string comparison
                        pass
            
            if val1 != val2:
                found_divergence = True
                print(f"Divergence detected in CPU state field '{field}' (Ticks: {entry1['tick']} matched):")
                print(f"  File 1 (line {entry1['original_line_num'] + 1}): {field.upper()}:{entry1[field]} - {entry1['original_line_content']}")
                print(f"  File 2 (line {entry2['original_line_num'] + 1}): {field.upper()}:{entry2[field]} - {entry2['original_line_content']}")
                divergence_point_orig_idx1 = entry1['original_line_num']
                divergence_point_orig_idx2 = entry2['original_line_num']
                break
        if found_divergence: break

        # If core CPU state matches, check IE/IF if present
        if entry1.get('ie') != entry2.get('ie') or entry1.get('if') != entry2.get('if'):
            # Ensure both have IE/IF before declaring it an IE/IF mismatch, otherwise it's a format difference
            if entry1.get('ie') is not None and entry1.get('if') is not None and \
               entry2.get('ie') is not None and entry2.get('if') is not None:
                found_divergence = True
                print(f"Divergence detected due to IE/IF mismatch (Ticks: {entry1['tick']} and core CPU state matched):")
                print(f"  File 1 (line {entry1['original_line_num'] + 1}): IE:{entry1.get('ie','N/A')} IF:{entry1.get('if','N/A')} - {entry1['original_line_content']}")
                print(f"  File 2 (line {entry2['original_line_num'] + 1}): IE:{entry2.get('ie','N/A')} IF:{entry2.get('if','N/A')} - {entry2['original_line_content']}")
                divergence_point_orig_idx1 = entry1['original_line_num']
                divergence_point_orig_idx2 = entry2['original_line_num']
                break
            elif not found_divergence: # If not already found a core divergence, and IE/IF differ in presence
                found_divergence = True
                print(f"Log format difference detected in IE/IF presence (Ticks: {entry1['tick']} and core CPU state matched):")
                print(f"  File 1 (line {entry1['original_line_num'] + 1}): {entry1['original_line_content']}")
                print(f"  File 2 (line {entry2['original_line_num'] + 1}): {entry2['original_line_content']}")
                divergence_point_orig_idx1 = entry1['original_line_num']
                divergence_point_orig_idx2 = entry2['original_line_num']
                break
    
    if not found_divergence:
        if len(log_entries1) != len(log_entries2):
            found_divergence = True # Treat as divergence for context printing
            print("No content divergence in common lines, but files have different numbers of parsable log entries.")
            if len(log_entries1) > min_entries_len:
                last_good_idx2 = log_entries2[-1]['original_line_num'] if log_entries2 else -1
                divergence_point_orig_idx1 = log_entries1[min_entries_len]['original_line_num']
                divergence_point_orig_idx2 = last_good_idx2
                print(f"  File 1 has more entries. Divergence after line {last_good_idx2 + 1} of File 2.")
                print(f"  Next entry in File 1 is line {divergence_point_orig_idx1 + 1}: {log_entries1[min_entries_len]['original_line_content']}")
            else:
                last_good_idx1 = log_entries1[-1]['original_line_num'] if log_entries1 else -1
                divergence_point_orig_idx2 = log_entries2[min_entries_len]['original_line_num']
                divergence_point_orig_idx1 = last_good_idx1
                print(f"  File 2 has more entries. Divergence after line {last_good_idx1 + 1} of File 1.")
                print(f"  Next entry in File 2 is line {divergence_point_orig_idx2 + 1}: {log_entries2[min_entries_len]['original_line_content']}")
        else:
            print("No divergence found. All parsed CPU states match.")
            return
    
    if not found_divergence:
        print("No parsable log entries found or files are identical and empty of parsable lines.")
        return

    # Context printing logic (uses original_lines1/2 and divergence_point_orig_idx1/2)
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
