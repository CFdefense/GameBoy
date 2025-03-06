def parse_trace_line(line):
    """Parse a single line of the trace into its components."""
    try:
        # Split the line into components
        parts = line.strip().split(' - ')
        if len(parts) < 2:
            return None
        
        # Extract PC (4-digit hex code)
        pc = parts[1][:4]
        
        # Extract register values
        registers_part = line.split('A:')[1].strip() if 'A:' in line else None
        if not registers_part:
            return None
            
        return {
            'pc': pc,
            'registers': registers_part,
            'full_line': line.strip()
        }
    except:
        return None

def compare_traces(trace1_lines, trace2_lines):
    """Compare two instruction traces and find mismatches."""
    mismatches = []
    line_num = 0
    
    # Compare lines until we reach the end of either input
    while line_num < len(trace1_lines) and line_num < len(trace2_lines):
        line1 = trace1_lines[line_num]
        line2 = trace2_lines[line_num]
        
        # Skip empty lines
        if not line1.strip() or not line2.strip():
            line_num += 1
            continue
            
        # Parse both lines
        parsed1 = parse_trace_line(line1)
        parsed2 = parse_trace_line(line2)
        
        # Skip lines that couldn't be parsed
        if not parsed1 or not parsed2:
            line_num += 1
            continue
        
        # Compare only PC and register values
        if parsed1['pc'] != parsed2['pc'] or \
           parsed1['registers'] != parsed2['registers']:
            mismatch = {
                'line_number': line_num + 1,
                'trace1': parsed1['full_line'],
                'trace2': parsed2['full_line']
            }
            mismatches.append(mismatch)
            
        line_num += 1
    
    # Check if one input is longer than the other
    remaining_lines = abs(len(trace1_lines) - len(trace2_lines))
    if remaining_lines > 0:
        longer_trace = "Trace 1" if len(trace1_lines) > len(trace2_lines) else "Trace 2"
        print(f"\n{longer_trace} has {remaining_lines} more lines than the other trace.")
    
    return mismatches

def print_results(mismatches):
    """Print the comparison results in a readable format."""
    if not mismatches:
        print("No mismatches found between the traces!")
        return
        
    print(f"\nFound {len(mismatches)} mismatches:")
    print("-" * 80)
    
    for mismatch in mismatches:
        print(f"Line {mismatch['line_number']}:")
        print(f"Trace 1: {mismatch['trace1']}")
        print(f"Trace 2: {mismatch['trace2']}")
        print("-" * 80)

def get_trace_input(trace_number):
    """Get trace input from console until empty line is entered."""
    print(f"\nEnter trace {trace_number} lines (press Enter twice to finish):")
    lines = []
    last_line = None
    
    while True:
        line = input()
        if line == "" and last_line == "":
            break
        lines.append(line)
        last_line = line
    
    return lines

def main():
    # Get traces from console input
    trace1_lines = get_trace_input(1)
    trace2_lines = get_trace_input(2)
    
    try:
        mismatches = compare_traces(trace1_lines, trace2_lines)
        print_results(mismatches)
    except Exception as e:
        print(f"An error occurred: {str(e)}")

if __name__ == "__main__":
    main() 