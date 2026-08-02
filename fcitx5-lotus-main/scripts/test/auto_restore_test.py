#!/usr/bin/env python3
import time
import subprocess
import sys
import os

# Test paragraph and expected output from the user's Swift script
test_paragraph = "Chafo cacs banfj, minhf ddang tesst Lotus. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your wonderful support with thiss software."
expected_output = "Chào các bạn, mình đang test Lotus. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. Đắk Lắk, Đắk Nông, Krông Búk. Thanks for your wonderful support with this software."

def run_test():
    print("\n" + "="*50)
    print("        Lotus Auto-Restore Test (Linux)")
    print("="*50)
    print(f"\n[Input (Telex typing)]:\n\"{test_paragraph}\"")
    print(f"\n[Expected Output]:\n\"{expected_output}\"")
    
    print("\n" + "!"*50)
    print(" INSTRUCTIONS:")
    print(" 1. Ensure fcitx5-lotus is active and in Vietnamese mode.")
    print(" 2. Have a text editor (like gedit, VS Code, or a terminal) ready.")
    print(" 3. Press Enter here, then IMMEDIATELY click into your editor.")
    print("!"*50)
    
    input("\nPress Enter to start the 3-second countdown...")
    
    for i in range(3, 0, -1):
        print(f" {i}...")
        time.sleep(1)
    
    print("\n>>> Typing now...")
    
    # Use xdotool to type the string. --delay 60 means 60ms per keystroke (human-like)
    # 50-100ms is usually good for IMs to process without dropping events.
    try:
        subprocess.run(["xdotool", "type", "--delay", "60", test_paragraph], check=True)
    except subprocess.CalledProcessError as e:
        print(f"\nError: xdotool failed: {e}")
        return

    print("\nDone!")
    print("\n" + "="*50)
    print(" VERIFICATION:")
    print(" Check the text in your editor.")
    print(" If it matches the 'Expected Output' above, the test PASSED.")
    print("="*50 + "\n")

if __name__ == "__main__":
    # Check for xdotool
    try:
        subprocess.run(["which", "xdotool"], check=True, capture_output=True)
    except subprocess.CalledProcessError:
        print("\nError: 'xdotool' is not installed.")
        print("Please install it to run this test:")
        print("  sudo apt install xdotool\n")
        sys.exit(1)
        
    run_test()
